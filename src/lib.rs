use anyhow::Error;
use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    prelude::{LocalWallet, Provider, SignerMiddleware, Wallet},
    providers::{Middleware, Ws},
    types::{H160, U256},
};

use std::{str::FromStr, sync::Arc};
use tracing_wasm::WASMLayerConfigBuilder;
use wasm_bindgen::prelude::*;

// 2. Add client type
type WalletType = Wallet<SigningKey>;
type Client = SignerMiddleware<Provider<Ws>, WalletType>;

pub const DID_ETH_REGISTRY: &str = "0xd1D374DDE031075157fDb64536eF5cC13Ae75000";
pub const DATA_LIFETIME: u64 = 86400 * 365; // 1 year

// Generate rust bindings for the DIDRegistry contract
abigen!(
    DIDRegistry,
    "./abi/EIP1056Registry.json",
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
    pub async fn new(rpc_url: &str, wallet_key: &str) -> Result<DidEthRegistry, JsError> {
        // this could be better, but shows how we may accept environment variables from the outside
        let registry_address = DID_ETH_REGISTRY;

        let provider = Provider::<Ws>::connect(rpc_url).await?;
        let chain_id = provider.get_chainid().await?;
        tracing::info!("Connected to chain: {chain_id}");

        // wallet/signer info
        let wallet_result = wallet_from_key(wallet_key);
        if let Ok(wallet) = wallet_result {
            tracing::info!("{:?}", wallet);
            let signer = Arc::new(SignerMiddleware::new(provider, wallet));
            tracing::info!("Registry Contract address: {registry_address}");
            let registry_address = H160::from_str(DID_ETH_REGISTRY).unwrap();
            let contract = DIDRegistry::new(registry_address, signer.clone());

            Ok(Self { contract, signer })
        } else {
            let err = wallet_result.unwrap_err();
            tracing::error!("Wallet error: {:?}", err);
            Err(JsError::new("Unable to construct wallet"))
        }
    }

    pub async fn owner(&self, id: &str) -> Result<String, JsError> {
        //let hex_range = 2..id.len();
        //let id = &id[hex_range];
        tracing::info!("identity: {}", id);
        let h160_result = H160::from_str(id);
        match h160_result {
            Ok(identity) => {
                tracing::info!("H160: {:?}", identity);
                let owner = self.contract.identity_owner(identity).call().await?;
                let owner_str = H160::to_string(&owner);
                tracing::info!("Owner: {}", owner_str);
                Ok(owner_str)
            }
            Err(e) => {
                tracing::error!("Error: {:?}", e);
                Err(JsError::new("Unable to parse identity"))
            }
        }
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

    pub async fn public_key(&self) -> Result<String, JsError> {
        let public_key = self.signer.address();
        let public_key_str = H160::to_string(&public_key);
        tracing::info!("Public Key: {public_key_str}");
        Ok(public_key_str)
    }
}
