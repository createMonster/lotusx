use crate::core::{config::ExchangeConfig, traits::ExchangeConnector};
use reqwest::Client;

pub struct BinancePerpConnector {
    pub(crate) client: Client,
    pub(crate) config: ExchangeConfig,
    pub(crate) base_url: String,
}

impl BinancePerpConnector {
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://testnet.binancefuture.com".to_string()
        } else {
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://fapi.binance.com".to_string())
        };

        Self {
            client: Client::new(),
            config,
            base_url,
        }
    }
}

impl ExchangeConnector for BinancePerpConnector {}
