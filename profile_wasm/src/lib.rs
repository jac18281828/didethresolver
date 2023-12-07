use anyhow::Error;
use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    prelude::{LocalWallet, Provider, SignerMiddleware, Wallet},
    providers::{Middleware, Ws},
    types::H160,
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

        let provider = Provider::<Ws>::connect(rpc_url).await?;
        let chain_id = provider.get_chainid().await?;
        tracing::info!("Connected to chain: {chain_id}");

        // wallet/signer info
        let wallet_result = wallet_from_key(&wallet_key);
        if let Ok(wallet) = wallet_result {
            tracing::info!("Wallet: {:?}", wallet);
            let middleware = SignerMiddleware::new_with_provider_chain(provider, wallet)
                .await
                .unwrap();
            let signer = Arc::new(middleware);
            tracing::info!("Registry Contract Connected: {registry_address}");
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
        let owner = self.contract.identity_owner(id_as_address).call().await?;
        tracing::info!("Owner: {owner}");
        Ok(format!("{owner}"))
    }

    pub fn wallet_address(&self) -> String {
        self.signer.address().to_string()
    }
}
