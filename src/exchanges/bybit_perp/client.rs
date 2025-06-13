use crate::core::{config::ExchangeConfig, traits::ExchangeConnector};
use reqwest::Client;

pub struct BybitPerpConnector {
    pub(crate) client: Client,
    pub(crate) config: ExchangeConfig,
    pub(crate) base_url: String,
}

impl BybitPerpConnector {
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://api-testnet.bybit.com".to_string()
        } else {
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.bybit.com".to_string())
        };

        Self {
            client: Client::new(),
            config,
            base_url,
        }
    }
}

impl ExchangeConnector for BybitPerpConnector {} 