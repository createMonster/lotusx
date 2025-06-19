use super::auth::HyperliquidAuth;
#[allow(clippy::wildcard_imports)]
use super::types::*;
use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::traits::ExchangeConnector;
use async_trait::async_trait;
use reqwest::Client;
use tracing::{error, instrument};

const MAINNET_API_URL: &str = "https://api.hyperliquid.xyz";
const TESTNET_API_URL: &str = "https://api.hyperliquid-testnet.xyz";

/// Helper to handle API response errors
#[cold]
#[inline(never)]
fn handle_api_error(status: u16, body: String) -> HyperliquidError {
    error!(status = status, body = %body, "API request failed");
    HyperliquidError::api_error(format!("HTTP {} error: {}", status, body))
}

pub struct HyperliquidClient {
    pub(crate) client: Client,
    pub(crate) base_url: String,
    pub(crate) auth: HyperliquidAuth,
    pub(crate) vault_address: Option<String>,
    pub(crate) is_testnet: bool,
}

impl HyperliquidClient {
    /// Create a new client with configuration
    pub fn new(config: ExchangeConfig) -> Self {
        let is_testnet = config.testnet;
        let has_credentials = config.has_credentials();
        let api_key = if has_credentials {
            Some(config.api_key().to_string())
        } else {
            None
        };
        let base_url_option = config.base_url;

        let base_url = if is_testnet {
            TESTNET_API_URL.to_string()
        } else {
            base_url_option.unwrap_or_else(|| MAINNET_API_URL.to_string())
        };

        let auth = api_key.map_or_else(HyperliquidAuth::new, |key| {
            HyperliquidAuth::with_private_key(&key).unwrap_or_else(|_| HyperliquidAuth::new())
        });

        Self {
            client: Client::new(),
            base_url,
            auth,
            vault_address: None,
            is_testnet,
        }
    }

    /// Create a new client with private key for signing
    pub fn with_private_key(private_key: &str, testnet: bool) -> Result<Self, ExchangeError> {
        let base_url = if testnet {
            TESTNET_API_URL.to_string()
        } else {
            MAINNET_API_URL.to_string()
        };

        let auth = HyperliquidAuth::with_private_key(private_key)?;

        Ok(Self {
            client: Client::new(),
            base_url,
            auth,
            vault_address: None,
            is_testnet: testnet,
        })
    }

    /// Create a read-only client without signing capabilities
    pub fn read_only(testnet: bool) -> Self {
        let base_url = if testnet {
            TESTNET_API_URL.to_string()
        } else {
            MAINNET_API_URL.to_string()
        };

        Self {
            client: Client::new(),
            base_url,
            auth: HyperliquidAuth::new(),
            vault_address: None,
            is_testnet: testnet,
        }
    }

    /// Set vault address for trading
    pub fn with_vault_address(mut self, vault_address: String) -> Self {
        self.vault_address = Some(vault_address);
        self
    }

    /// Get wallet address
    pub fn wallet_address(&self) -> Option<&str> {
        self.auth.wallet_address()
    }

    /// Check if client can sign transactions
    pub fn can_sign(&self) -> bool {
        self.auth.can_sign()
    }

    /// Check if client is in testnet mode
    pub fn is_testnet(&self) -> bool {
        self.is_testnet
    }

    /// Get WebSocket URL for this client
    pub fn get_websocket_url(&self) -> String {
        if self.is_testnet {
            "wss://api.hyperliquid-testnet.xyz/ws".to_string()
        } else {
            "wss://api.hyperliquid.xyz/ws".to_string()
        }
    }

    // Internal helper methods for HTTP requests
    #[instrument(
        skip(self, request),
        fields(exchange = "hyperliquid", request_type = "info")
    )]
    pub(crate) async fn post_info_request<T>(
        &self,
        request: &InfoRequest,
    ) -> Result<T, ExchangeError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}/info", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(request)
            .send()
            .await
            .with_symbol_context("*")?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await.with_symbol_context("*")?;
            return Err(ExchangeError::Other(
                handle_api_error(status, error_text).to_string(),
            ));
        }

        let result: T = response.json().await.with_symbol_context("*")?;
        Ok(result)
    }

    #[instrument(skip(self, request), fields(exchange = "hyperliquid", request_type = "exchange", vault = ?self.vault_address))]
    pub(crate) async fn post_exchange_request<T>(
        &self,
        request: &ExchangeRequest,
    ) -> Result<T, ExchangeError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}/exchange", self.base_url);

        let response = self.client.post(&url).json(request).send().await;

        let response = if let Some(vault_address) = &self.vault_address {
            response.with_vault_context(vault_address)?
        } else {
            response.with_symbol_context("*")?
        };

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response.text().await;

            let error_text = if let Some(vault_address) = &self.vault_address {
                error_text.with_vault_context(vault_address)?
            } else {
                error_text.with_symbol_context("*")?
            };

            return Err(ExchangeError::Other(
                handle_api_error(status, error_text).to_string(),
            ));
        }

        let result: T = if let Some(vault_address) = &self.vault_address {
            response.json().await.with_vault_context(vault_address)?
        } else {
            response.json().await.with_symbol_context("*")?
        };

        Ok(result)
    }
}

#[async_trait]
impl ExchangeConnector for HyperliquidClient {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = HyperliquidClient::read_only(true);
        assert!(!client.can_sign());
        assert!(client.is_testnet);
    }
}
