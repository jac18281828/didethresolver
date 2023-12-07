use dideth::DidEthRegistry;
use ethenv::{environment, init, printenv};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();
    init();
    let env = environment();
    printenv(&env);

    let public_key = env.public_key.clone();
    let contract = DidEthRegistry::new(env.rpc_url, env.private_key)
        .await
        .unwrap();
    let sender = contract.wallet_address();
    println!("sender: {}", sender);
    let owner_id = contract.owner(public_key.to_string()).await.unwrap();
    println!("owner: {}", owner_id);
    for (key, value) in env.attribute.iter() {
        println!("revoke_attribute - key: {}, value: {}", key, value);
        let tx_receipt = contract
            .revoke_attribute(key.clone(), value.clone())
            .await
            .unwrap();
        println!("tx: {:?}", tx_receipt);
    }
}
