use anyhow::Error;
use ethabi::Token;
use ethers::{
    contract::abigen,
    core::k256::ecdsa::SigningKey,
    prelude::{LocalWallet, Provider, SignerMiddleware, Wallet},
    providers::{Middleware, Ws},
    types::{Address, Filter, H160, H256, U256, U64},
    utils::format_bytes32_string,
};
use std::{str::FromStr, sync::Arc};

type WalletType = Wallet<SigningKey>;
type Client = SignerMiddleware<Provider<Ws>, WalletType>;

pub const REQUIRED_CONFIRMATIONS: usize = 10;
pub const DID_ETH_REGISTRY: &str = "0xd1D374DDE031075157fDb64536eF5cC13Ae75000";
pub const DATA_LIFETIME: u64 = 86400 * 365; // 1 year

// Generate rust bindings for the DIDRegistry contract
abigen!(
    DIDRegistry,
    "../abi/DIDRegistry.json",
    derives(serde::Deserialize, serde::Serialize)
);

pub struct DidEthRegistry {
    contract: DIDRegistry<Client>,
    signer: Arc<Client>,
}

fn wallet_from_key(wallet_key: &str) -> Result<WalletType, Error> {
    let wallet = wallet_key.parse::<LocalWallet>()?;
    Ok(wallet)
}

impl DidEthRegistry {
    pub async fn new(rpc_url: String, wallet_signer: String) -> Result<DidEthRegistry, Error> {
        // this could be better, but shows how we may accept environment variables from the outside
        let registry_address = DID_ETH_REGISTRY;

        let provider = Provider::<Ws>::connect(rpc_url).await?;
        let chain_id = provider.get_chainid().await?;
        tracing::info!("Connected to chain: {chain_id}");

        // wallet/signer info
        let wallet_result = wallet_from_key(&wallet_signer);
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
            Err(err)
        }
    }

    pub async fn owner(&self, id: String) -> Result<String, Error> {
        let id_as_address = H160::from_str(&id).unwrap();
        let owner = self.contract.identity_owner(id_as_address).call().await?;
        tracing::info!("Owner: {owner}");
        Ok(format!("{owner}"))
    }

    pub async fn set_attribute(&self, name: String, value: String) -> Result<String, Error> {
        let name_b32 = format_bytes32_string(&name).unwrap();
        let tx = self.contract.set_attribute(
            self.signer.address(),
            name_b32,
            value.as_bytes().to_vec().into(),
            U256::from(DATA_LIFETIME),
        );
        let receipt = tx
            .send()
            .await?
            .confirmations(REQUIRED_CONFIRMATIONS)
            .await?;
        Ok(format!("{receipt:?}"))
    }

    pub async fn attribute(&self, id: String) -> Result<Vec<(String, String)>, Error> {
        // TODO handle revocation
        let id_as_address = H160::from_str(&id).unwrap();
        let mut result_vec = Vec::new();
        let mut prev_change_result: Result<U256, _> =
            self.contract.changed(id_as_address).call().await;
        let current_block = self.signer.get_block_number().await?;
        let block = self.signer.get_block(current_block).await.unwrap();
        let mut block_timestamp = U256::zero();
        if let Some(block) = block {
            block_timestamp = block.timestamp;
            tracing::info!("block_timestamp: {block_timestamp}");
        }
        while let Ok(prev_change) = prev_change_result {
            let prev_change = U64::from(prev_change.as_u64());
            if prev_change == U64::zero() {
                tracing::debug!("last change");
                break;
            }
            tracing::info!("prev_change: {prev_change}");
            let id_topic = [H256::from(id_as_address)];
            let contract_addr = DID_ETH_REGISTRY.parse::<Address>().unwrap();
            let filter = Filter::new()
                .from_block(prev_change)
                .to_block(prev_change)
                .event("DIDAttributeChanged(address,bytes32,bytes,uint256,uint256)")
                .address(vec![contract_addr])
                .topic1(id_topic.to_vec());

            tracing::debug!("filter: {:?}", filter);
            let logs = self.signer.get_logs(&filter).await;
            if let Ok(logs) = logs {
                if logs.len() == 0 {
                    tracing::debug!("no logs");
                    break;
                }
                for log in logs.iter() {
                    // topic 0 is the event signature
                    tracing::debug!("log: {:?}", log);
                    let owner_id = Address::from(log.topics[0]);
                    tracing::debug!("owner: {:?}", owner_id);

                    let param_result = self.decode_did_attribute_changed_param(log.data.to_vec());
                    if let Ok(param) = param_result {
                        tracing::debug!("param: {:?}", param);
                        let name_fixed = param[0].clone().into_fixed_bytes().unwrap();
                        let attribute_name = String::from_utf8(name_fixed).unwrap();
                        tracing::info!("attribute name: {attribute_name}");
                        let attribute_value = param[1].clone().into_string().unwrap();
                        tracing::info!("attribute value: {attribute_value}");
                        let validity = param[2].clone().into_uint().unwrap();
                        tracing::info!("valid until: {validity}");
                        let log_prev_change = param[3].clone().into_uint().unwrap();
                        prev_change_result = Ok(log_prev_change);
                        if validity < block_timestamp {
                            result_vec.push((attribute_name.clone(), attribute_value));
                        }
                    } else {
                        tracing::error!("Error decoding param: {:?}", param_result);
                        break;
                    }
                }
            } else {
                tracing::error!("Error getting logs: {:?}", logs);
                break;
            }
        }
        Ok(result_vec)
    }

    pub fn wallet_address(&self) -> String {
        self.signer.address().to_string()
    }

    fn decode_did_attribute_changed_param(&self, data: Vec<u8>) -> Result<Vec<Token>, Error> {
        let param = [
            ethabi::ParamType::FixedBytes(32),
            ethabi::ParamType::String,
            ethabi::ParamType::Uint(256),
            ethabi::ParamType::Uint(256),
        ];
        let decoded = ethabi::decode(&param, &data)?;
        Ok(decoded)
    }
}
