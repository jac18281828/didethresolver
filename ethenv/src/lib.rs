use std::env;

pub struct Environment {
    pub rpc_url: String,
    pub public_key: String,
    pub private_key: String,
    pub attribute: Vec<(String, String)>,
}

pub fn init() {
    dotenv::dotenv().ok();
}

pub fn environment() -> Environment {
    let attribute = env::var("ATTRIBUTE").expect("ATTRIBUTE must be set");
    let attribute_vec = attribute.split(",").collect::<Vec<&str>>();
    let attribute_pair = attribute_vec
        .iter()
        .map(|x| {
            let pair = x.split("=").collect::<Vec<&str>>();
            (pair[0].trim().to_string(), pair[1].trim().to_string())
        })
        .collect::<Vec<(String, String)>>();
    Environment {
        rpc_url: env::var("RPC_URL").expect("RPC_URL must be set"),
        public_key: env::var("PUBLIC_KEY").expect("PUBLIC_KEY must be set"),
        private_key: env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set"),
        attribute: attribute_pair,
    }
}

pub fn printenv(env: &Environment) {
    println!("rpc_url: {}", env.rpc_url.split("v2").next().unwrap());
    println!("private_key: {}", scram(env.private_key.clone()));
    for (key, value) in env.attribute.iter() {
        println!("attribute: {}={}", key, value);
    }
}

pub fn scram(value: String) -> String {
    let mut scrambled = String::new();
    for _ in 0..value.len().min(10) {
        scrambled.push('*');
    }
    scrambled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment() {
        std::env::set_var("RPC_URL", "https://example.com");
        std::env::set_var("PUBLIC_KEY", "my_public_key");
        std::env::set_var("PRIVATE_KEY", "my_private_key");
        std::env::set_var("ATTRIBUTE", "key=value, key2=value2");

        let env = environment();

        assert_eq!(env.rpc_url, "https://example.com");
        assert_eq!(env.public_key, "my_public_key");
        assert_eq!(env.private_key, "my_private_key");
        assert_eq!(env.attribute.len(), 2);
        assert_eq!(env.attribute[0].0, "key");
        assert_eq!(env.attribute[0].1, "value");
        assert_eq!(env.attribute[1].0, "key2");
        assert_eq!(env.attribute[1].1, "value2");
    }

    #[test]
    #[should_panic]
    fn test_environment_missing_rpc_url() {
        std::env::remove_var("RPC_URL");
        std::env::set_var("PUBLIC_KEY", "my_public_key");        
        std::env::set_var("PRIVATE_KEY", "my_private_key");
        std::env::set_var("ATTRIBUTE", "key=value");

        environment();
    }

    #[test]
    #[should_panic]
    fn test_environment_missing_private_key() {
        std::env::set_var("PUBLIC_KEY", "my_public_key");        
        std::env::set_var("RPC_URL", "https://example.com");
        std::env::remove_var("PRIVATE_KEY");
        std::env::set_var("ATTRIBUTE", "key=value");        

        environment();
    }

    #[test]
    #[should_panic]
    fn test_environment_missing_attribute() {
        std::env::set_var("PUBLIC_KEY", "my_public_key");
        std::env::set_var("RPC_URL", "https://example.com");
        std::env::set_var("PRIVATE_KEY", "my_private_key");
        std::env::remove_var("ATTRIBUTE");

        environment();
    }

    #[test]
    #[should_panic]
    fn test_environment_missing_public_key() {
        std::env::set_var("RPC_URL", "https://example.com");
        std::env::set_var("PRIVATE_KEY", "my_private_key");
        std::env::set_var("ATTRIBUTE", "key=value");
        std::env::remove_var("PUBLIC_KEY");

        environment();
    }

    #[test]
    fn test_scram() {
        assert_eq!(scram("12345678901".to_string()), "**********");
        assert_eq!(scram("1234567890".to_string()), "**********");
        assert_eq!(scram("123456789".to_string()), "*********");
        assert_eq!(scram("12345678".to_string()), "********");
        assert_eq!(scram("1234567".to_string()), "*******");
        assert_eq!(scram("123456".to_string()), "******");
        assert_eq!(scram("12345".to_string()), "*****");
        assert_eq!(scram("1234".to_string()), "****");
        assert_eq!(scram("123".to_string()), "***");
        assert_eq!(scram("12".to_string()), "**");
        assert_eq!(scram("1".to_string()), "*");
        assert_eq!(scram("".to_string()), "");
    }
}
