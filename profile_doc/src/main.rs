use dideth::DidEthRegistry;
use ethenv::{environment, init, printenv};

use serde_json::json;

fn fmt_as_did(public_key: String) -> String {
    format!("did:ethr:{}", public_key)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> serde_json::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    init();
    let env = environment();
    printenv(&env);

    let public_key = env.public_key.clone();
    let contract = DidEthRegistry::new(env.rpc_url, env.private_key)
        .await
        .unwrap();
    let owner_id = contract.owner(public_key.clone()).await.unwrap();

    let owner_attribute_vec = contract.attribute(owner_id.clone()).await.unwrap();
    let did = fmt_as_did(public_key.clone());

    let mut xmtp_service = std::collections::HashMap::<String, String>::new();
    xmtp_service.insert("id".to_string(), did.clone());
    xmtp_service.insert("type".to_string(), "MessagingService".to_string());
    xmtp_service.insert("recipientKeys".to_string(), public_key.clone());
    xmtp_service.insert(
        "serviceEndpoint".to_string(),
        "https://xmtp.com/resolver".to_string(),
    );

    owner_attribute_vec.iter().for_each(|(key, value)| {
        xmtp_service.insert(key.clone(), value.clone());
    });

    let did_document = json!({
        "@context": [
            "https://www.w3.org/ns/did/v1",
            "https://w3id.org/security/suites/ed25519-2020/v1"
        ],
        "controller": format!("{}", did.clone()),
        "id": format!("{}", did.clone()),
        "verificationMethod": [
            {
                "id": format!("{}", did.clone()),
                "type": "Ed25519VerificationKey2020",
                "controller": format!("{}", did.clone()),
                "publicKeyMultibase": format!("{}", public_key.clone()),
            }
        ],
        "service": [ xmtp_service ],
    });
    serde_json::to_writer_pretty(std::io::stdout(), &did_document)?;
    Ok(())
}
