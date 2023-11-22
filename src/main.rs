use ethers::{
    providers::{Http, Middleware, Provider},
    types::{Address, Filter, H160, H256, U64},
};
use log::{debug, info};
use std::sync::Arc;
use wasm_bindgen::{JsCast, JsError};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew::{Html, Properties};

#[derive(Properties, Clone, PartialEq)]
pub struct FormProps {
    pub label: String,
    pub value: String,
    pub callback: Callback<String>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct AttributeProps {
    pub label: String,
    pub name: String,
    pub value: String,
}

#[derive(Properties, Clone, PartialEq)]
pub struct DidDocumentProps {
    #[prop_or_default]
    pub public_key: String,
    #[prop_or_default]
    pub private_key: String,
    #[prop_or_default]
    pub rpc_url: String,
    #[prop_or_default]
    pub did: String,
    #[prop_or_default]
    pub owner: String,
    #[prop_or_default]
    pub attributes: Vec<AttributeProps>,
}

fn scram(value: String) -> String {
    let mut scrambled = String::new();
    for _ in 0..value.len().min(10) {
        scrambled.push('*');
    }
    scrambled
}

#[function_component(SetAttributeComponent)]
pub fn set_attribute_component(props: &AttributeProps) -> Html {
    let AttributeProps { label, name, value } = props.clone();

    let set_attribute = Callback::from(move |_| {
        info!("setting attribute");
    });
    html! {
        <div class="setAttribute">
            <label>{ label }</label>
            <input type="text" name={"name"} value={name} />
            <input type="text" name={"value"} value={value} />
            <button label="Set" onclick={set_attribute} >{"Set"}</button>
        </div>
    }
}

#[function_component(FormInputComponent)]
pub fn form_input_component(props: &FormProps) -> Html {
    let FormProps {
        label,
        value,
        callback,
    } = props.clone();
    let is_private = label == "Private Key";
    let set_value = Callback::from(move |input_event: Event| {
        let input_event_target = input_event.target().unwrap();
        let current_input_text = input_event_target.unchecked_into::<web_sys::HtmlInputElement>();
        info!(
            "current_input_text: {:?}",
            if is_private {
                scram(current_input_text.value())
            } else {
                current_input_text.value()
            }
        );
        callback.emit(current_input_text.value());
    });
    html! {
        <div>
            <label>{ label }</label>
            <input type={ if is_private {"password"} else {"text"} } name={props.label.clone()} onchange={set_value} value={ if is_private {scram(value)} else {value} } />
        </div>
    }
}

#[function_component(DIDDocument)]
fn did_document(did_doc: &DidDocumentProps) -> Html {
    let DidDocumentProps {
        public_key: _,
        private_key: _,
        rpc_url: _,
        did,
        owner,
        attributes,
    } = did_doc.clone();
    html! {
        <div>
            <div>{format!("did: {}", did) }</div>
            <div>{format!("owner: {}", owner) }</div>
            <div>
                { for attributes.iter().map(|attribute| html!{ <SetAttributeComponent label={attribute.label.clone()} name={attribute.name.clone()} value={attribute.value.clone()} /> }) }
            </div>
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    let did_prop = use_state(|| DidDocumentProps {
        public_key: "".to_string(),
        private_key: "".to_string(),
        rpc_url: "https://eth-sepolia.g.alchemy.com/v2".to_string(),
        did: "did:eth:0x1234567890".to_string(),
        owner: "0x1234567890".to_string(),
        attributes: vec![
            AttributeProps {
                label: "name".to_string(),
                name: "name".to_string(),
                value: "value".to_string(),
            },
            AttributeProps {
                label: "name".to_string(),
                name: "name".to_string(),
                value: "value".to_string(),
            },
        ],
    });

    let on_public_key = {
        let did_prop = did_prop.clone();
        Callback::from(move |value: String| {
            did_prop.set(DidDocumentProps {
                public_key: value.clone(),
                ..(*did_prop).clone()
            });
            info!("public key: {:?}", value);
        })
    };

    let on_private_key = {
        let did_prop = did_prop.clone();
        Callback::from(move |value: String| {
            did_prop.set(DidDocumentProps {
                private_key: value.clone(),
                ..(*did_prop).clone()
            });
            info!("private key: {:?}", scram(value));
        })
    };

    let on_rpc_url = {
        let did_prop = did_prop.clone();
        Callback::from(move |value: String| {
            did_prop.set(DidDocumentProps {
                rpc_url: value.clone(),
                ..(*did_prop).clone()
            });
            info!("rpc url: {:?}", value);
        })
    };

    let on_search = {
        let did_prop = did_prop.clone();
        Callback::from(move |_| {
            let did_prop = did_prop.clone();
            let rpc_url = did_prop.rpc_url.clone();
            let private_key = did_prop.private_key.clone();
            info!("searching");
            spawn_local(async move {
                let provider = Provider::<Http>::try_from(&rpc_url.clone()).unwrap();
                let client = Arc::new(provider);
                let registry_result =
                    didethresolver::DidEthRegistry::new(rpc_url, private_key).await;
                if let Ok(registry) = registry_result {
                    let public_key = registry.signer_address().to_string();
                    debug!("public key: {:?}", public_key);
                    let owner_result = registry.owner(public_key.clone()).await;
                    if let Ok(owner) = owner_result {
                        info!("owner: {:?}", owner);
                        let mut attribute: Vec<AttributeProps> = vec![];
                        let mut prev_change_result = registry.changed(owner.clone()).await;
                        while let Ok(prev_change_u64) = prev_change_result {
                            let prev_change = U64::from(prev_change_u64);
                            if prev_change.as_u64() == 0 {
                                break;
                            }
                            let token_topics = [H256::from(owner.parse::<H160>().unwrap())];
                            let contract_addr =
                                registry.contract_address().parse::<Address>().unwrap();
                            let filter = Filter::new()
                                .address(contract_addr)
                                .event("DIDAttributeChanged(address,bytes32,bytes,uint256,uint256")
                                .topic1(token_topics.to_vec())
                                .topic2(token_topics.to_vec())
                                .from_block(prev_change)
                                .to_block(prev_change);
                            let logs = client.get_logs(&filter).await;
                            if let Ok(logs) = logs {
                                for log in logs.iter() {
                                    debug!("log: {:?}", log);
                                    let owner_id = Address::from(log.topics[0]);
                                    let attribute_name =
                                        String::from_utf8(log.topics[1].as_fixed_bytes().to_vec())
                                            .unwrap();
                                    let attribute_value =
                                        String::from_utf8(log.topics[2].as_bytes().to_vec())
                                            .unwrap();
                                    let log_prev_change =
                                        U64::from_big_endian(log.topics[3].as_bytes());
                                    debug!("owner: {:?}", owner_id);
                                    attribute.push(AttributeProps {
                                        label: attribute_name.clone(),
                                        name: attribute_name.clone(),
                                        value: attribute_value.clone(),
                                    });
                                    prev_change_result = Ok(log_prev_change.as_u64());
                                }
                            } else {
                                attribute.push(AttributeProps {
                                    label: "New Attribute".to_string(),
                                    name: "".to_string(),
                                    value: "".to_string(),
                                });
                                prev_change_result = Err(JsError::new("Unable to get logs"));
                            }
                        }

                        did_prop.set(DidDocumentProps {
                            owner: owner.clone(),
                            attributes: attribute.clone(),
                            ..(*did_prop).clone()
                        });
                    } else {
                        info!("owner error.");
                    }
                } else {
                    info!("registry error.");
                }
            });
        })
    };

    html! {
        <html lang="en">
        <head>
        <meta charset="UTF-8" />
        <meta http-equiv="X-UA-Compatible" content="IE=edge" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>{ "did:eth registry" }</title>
        </head>
        <body>
        <div class="formInput">
        <FormInputComponent label="Public Key" value={did_prop.public_key.clone()} callback={on_public_key}/>
        <FormInputComponent label="Private Key" value={scram(did_prop.private_key.clone())} callback={on_private_key} />
        <FormInputComponent label="Rpc Url" value={did_prop.rpc_url.clone()} callback={on_rpc_url} />
        <button onclick={on_search}>{ "Search" }</button>
        </div>
        <div>
        <DIDDocument ..(*did_prop).clone() />
        </div>
        </body>
        </html>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    didethresolver::set_logger();
    info!("Starting up");
    yew::Renderer::<App>::new().render();
}
