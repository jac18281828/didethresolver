use anyhow::Error;
use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    prelude::{LocalWallet, Provider, SignerMiddleware, Wallet},
    providers::{Http, Middleware},
    types::{H160, U256},
};
use std::{str::FromStr, sync::Arc};
use tracing_wasm::WASMLayerConfigBuilder;
use wasm_bindgen::prelude::*;

// 2. Add client type
type WalletType = Wallet<SigningKey>;
type Client = SignerMiddleware<Provider<Http>, WalletType>;

pub const DID_ETH_REGISTRY: &str = "0xd1D374DDE031075157fDb64536eF5cC13Ae75000";
pub const DATA_LIFETIME: u64 = 86400 * 365; // 1 year
pub const MAX_SAFE_INTEGER: u64 = 9007199254740991; // javascript limit on number size

// Generate rust bindings for the DIDRegistry contract
abigen!(
    DIDRegistry,
    "./abi/DIDRegistry.json",
    derives(serde::Deserialize, serde::Serialize)
);

#[wasm_bindgen]
pub fn set_logger() {
    let tracing_config = WASMLayerConfigBuilder::new()
        .set_max_level(tracing::Level::INFO)
        .build();
    tracing_wasm::set_as_global_default_with_config(tracing_config);
}

#[wasm_bindgen]
pub struct DidEthRegistry {
    contract: DIDRegistry<Client>,
    signer: Arc<Client>,
}

fn wallet_from_key(wallet_key: &str) -> Result<WalletType, Error> {
    let wallet = wallet_key.parse::<LocalWallet>()?;
    Ok(wallet)
}

#[wasm_bindgen]
impl DidEthRegistry {
    #[wasm_bindgen(constructor)]
    pub async fn new(rpc_url: String, wallet_key: String) -> Result<DidEthRegistry, JsError> {
        // this could be better, but shows how we may accept environment variables from the outside
        let registry_address = DID_ETH_REGISTRY;
        let provider = Provider::<Http>::try_from(rpc_url.as_str()).unwrap();
        let chain_id = provider.get_chainid().await?;
        tracing::info!("Connected to chain: {chain_id}");

        // wallet/signer info
        let wallet_result = wallet_from_key(&wallet_key);
        if let Ok(wallet) = wallet_result {
            tracing::info!("Wallet: {:?}", wallet);
            let signer = Arc::new(SignerMiddleware::new(provider, wallet));
            tracing::info!("Registry Contract address: {registry_address}");
            let registry_address = H160::from_str(registry_address).unwrap();
            let contract = DIDRegistry::new(registry_address, signer.clone());

            Ok(Self { contract, signer })
        } else {
            let err = wallet_result.unwrap_err();
            tracing::error!("Wallet error: {:?}", err);
            Err(JsError::new("Unable to construct wallet"))
        }
    }

    pub async fn owner(&self, id: String) -> Result<String, JsError> {
        let id_as_address = H160::from_str(&id).unwrap();
        tracing::debug!("ID: {}", id_as_address);
        let owner = self.contract.identity_owner(id_as_address).call().await?;
        tracing::info!("Owner: {owner}");
        Ok(format!("{owner}"))
    }

    pub async fn set_attribute(&self, attribute: String) -> Result<String, JsError> {
        let tx = self.contract.set_attribute(
            self.signer.address(),
            *b"did:eth some attribute0000000000",
            attribute.as_bytes().to_vec().into(),
            U256::from(DATA_LIFETIME),
        );
        let receipt = tx.send().await?.await?;
        tracing::info!("Receipt: {receipt:?}");
        Ok(format!("{receipt:?}"))
    }

    pub async fn changed(&self, owner: String) -> Result<u64, JsError> {
        let owner_as_address = H160::from_str(&owner).unwrap();
        tracing::debug!("Owner: {}", owner_as_address);
        let changed = self.contract.changed(owner_as_address).call().await?;
        tracing::info!("Changed: {changed}");
        if changed <= U256::from(MAX_SAFE_INTEGER) {
            tracing::info!("Changed: {changed}");
            Ok(changed.as_u64())
        } else {
            tracing::warn!("Changed: {changed} exceeds MAX_SAFE_INTEGER");
            Err(JsError::new(
                "Unable to convert changed into JS MAX_SAFE_INTEGER",
            ))
        }
    }

    pub fn signer_address(&self) -> String {
        self.signer.address().to_string()
    }

    pub fn contract_address(&self) -> String {
        self.contract.address().to_string()
    }
}
