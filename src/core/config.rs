use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub api_key: String,
    pub secret_key: String,
    pub testnet: bool,
    pub base_url: Option<String>,
}

impl ExchangeConfig {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
            testnet: false,
            base_url: None,
        }
    }

    pub fn testnet(mut self, testnet: bool) -> Self {
        self.testnet = testnet;
        self
    }

    pub fn base_url(mut self, base_url: String) -> Self {
        self.base_url = Some(base_url);
        self
    }
}
