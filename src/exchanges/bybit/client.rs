use crate::core::config::ExchangeConfig;
use reqwest::Client;

pub struct BybitConnector {
    pub client: Client,
    pub config: ExchangeConfig,
    pub base_url: String,
}

impl BybitConnector {
    pub fn new(config: ExchangeConfig) -> Self {
        let client = Client::new();
        let base_url = if config.testnet {
            "https://api-testnet.bybit.com".to_string()
        } else {
            "https://api.bybit.com".to_string()
        };

        Self {
            client,
            config,
            base_url,
        }
    }

    pub fn get_config(&self) -> &ExchangeConfig {
        &self.config
    }
}
