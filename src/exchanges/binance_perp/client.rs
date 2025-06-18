use super::types::BinancePerpError;
use crate::core::{config::ExchangeConfig, traits::ExchangeConnector};
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;
use tracing::instrument;

pub struct BinancePerpConnector {
    pub(crate) client: Client,
    pub(crate) config: ExchangeConfig,
    pub(crate) base_url: String,
    pub(crate) max_retries: u32,
    pub(crate) base_delay_ms: u64,
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
            max_retries: 3,
            base_delay_ms: 100,
        }
    }

    #[instrument(skip(self, request_fn), fields(url = %url))]
    pub(crate) async fn request_with_retry<T>(
        &self,
        request_fn: impl Fn() -> reqwest::RequestBuilder,
        url: &str,
    ) -> Result<T, BinancePerpError>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut attempts = 0;

        loop {
            let response = match request_fn().send().await {
                Ok(resp) => resp,
                Err(e) if attempts < self.max_retries && e.is_timeout() => {
                    attempts += 1;
                    let delay = self.base_delay_ms * 2_u64.pow(attempts - 1);
                    tracing::warn!(
                        attempt = attempts,
                        delay_ms = delay,
                        error = %e,
                        "Network timeout, retrying request"
                    );
                    sleep(Duration::from_millis(delay)).await;
                    continue;
                }
                Err(e) => {
                    return Err(BinancePerpError::network_error(format!(
                        "Request failed after {} attempts: {}",
                        attempts, e
                    )));
                }
            };

            if response.status().is_success() {
                return response.json::<T>().await.map_err(|e| {
                    BinancePerpError::parse_error(
                        format!("Failed to parse response: {}", e),
                        Some(url.to_string()),
                    )
                });
            } else if response.status() == 429 && attempts < self.max_retries {
                // Rate limit hit
                attempts += 1;
                let delay = self.base_delay_ms * 2_u64.pow(attempts - 1);
                tracing::warn!(
                    attempt = attempts,
                    delay_ms = delay,
                    status = %response.status(),
                    "Rate limit hit, backing off"
                );
                sleep(Duration::from_millis(delay)).await;
                continue;
            }

            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(BinancePerpError::network_error(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }
    }
}

impl ExchangeConnector for BinancePerpConnector {}
