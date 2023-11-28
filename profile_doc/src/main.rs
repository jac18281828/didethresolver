use dideth::DidEthRegistry;
use ethenv::{environment, init, printenv};

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
    println!("owner: {}", owner_id.clone());
    let owner_attribute = contract.attribute(owner_id.clone()).await.unwrap();
    for (key, value) in owner_attribute.iter() {
        println!("attribute - key: {}, value: {}", key, value);
    }
}
