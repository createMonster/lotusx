use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{ReqwestRest, RestClientBuilder, RestClientConfig, TungsteniteWs};
use crate::exchanges::hyperliquid::codec::HyperliquidCodec;
use crate::exchanges::hyperliquid::connector::HyperliquidConnector;
use crate::exchanges::hyperliquid::rest::HyperliquidRest;
use crate::exchanges::hyperliquid::signer::HyperliquidSigner;
use std::sync::Arc;

const MAINNET_API_URL: &str = "https://api.hyperliquid.xyz";
const TESTNET_API_URL: &str = "https://api.hyperliquid-testnet.xyz";
const MAINNET_WS_URL: &str = "wss://api.hyperliquid.xyz/ws";
const TESTNET_WS_URL: &str = "wss://api.hyperliquid-testnet.xyz/ws";

/// Builder for creating Hyperliquid connectors
pub struct HyperliquidBuilder {
    config: ExchangeConfig,
    enable_websocket: bool,
    vault_address: Option<String>,
}

impl HyperliquidBuilder {
    /// Create a new builder with the provided config
    pub fn new(config: ExchangeConfig) -> Self {
        Self {
            config,
            enable_websocket: false,
            vault_address: None,
        }
    }

    /// Enable WebSocket support
    pub fn with_websocket(mut self) -> Self {
        self.enable_websocket = true;
        self
    }

    /// Set vault address for trading (optional)
    pub fn with_vault_address(mut self, vault_address: String) -> Self {
        self.vault_address = Some(vault_address);
        self
    }

    /// Build a REST-only connector
    pub fn build_rest_only(self) -> Result<HyperliquidConnector<ReqwestRest, ()>, ExchangeError> {
        let rest_client = self.build_rest_client()?;
        let hyperliquid_rest = self.build_hyperliquid_rest(rest_client)?;
        Ok(HyperliquidConnector::new(hyperliquid_rest))
    }

    /// Build a connector with WebSocket support
    pub fn build_with_websocket(
        self,
    ) -> Result<HyperliquidConnector<ReqwestRest, TungsteniteWs<HyperliquidCodec>>, ExchangeError>
    {
        let rest_client = self.build_rest_client()?;
        let hyperliquid_rest = self.build_hyperliquid_rest(rest_client)?;
        let ws_client = self.build_websocket_client();
        Ok(HyperliquidConnector::new_with_ws(
            hyperliquid_rest,
            ws_client,
        ))
    }

    /// Build a connector (auto-detects WebSocket requirement)
    pub fn build(self) -> Result<HyperliquidConnector<ReqwestRest, ()>, ExchangeError> {
        // For now, we'll return the REST-only version since WebSocket with type erasure is complex
        self.build_rest_only()
    }

    fn build_rest_client(&self) -> Result<ReqwestRest, ExchangeError> {
        let base_url = if self.config.testnet {
            TESTNET_API_URL
        } else {
            self.config.base_url.as_deref().unwrap_or(MAINNET_API_URL)
        };

        let rest_config = RestClientConfig::new(base_url.to_string(), "hyperliquid".to_string());
        let mut rest_builder = RestClientBuilder::new(rest_config);

        // Add signer if credentials are available
        if self.config.has_credentials() {
            let private_key = self.config.secret_key();
            let signer = if private_key.is_empty() {
                Arc::new(HyperliquidSigner::new())
            } else {
                Arc::new(HyperliquidSigner::with_private_key(private_key)?)
            };
            rest_builder = rest_builder.with_signer(signer);
        }

        rest_builder.build()
    }

    fn build_hyperliquid_rest(
        &self,
        rest_client: ReqwestRest,
    ) -> Result<HyperliquidRest<ReqwestRest>, ExchangeError> {
        let signer = if self.config.has_credentials() {
            let private_key = self.config.secret_key();
            if private_key.is_empty() {
                Some(HyperliquidSigner::new())
            } else {
                Some(HyperliquidSigner::with_private_key(private_key)?)
            }
        } else {
            None
        };

        let mut hyperliquid_rest = HyperliquidRest::new(rest_client, signer, self.config.testnet);

        if let Some(vault_address) = &self.vault_address {
            hyperliquid_rest = hyperliquid_rest.with_vault_address(vault_address.clone());
        }

        Ok(hyperliquid_rest)
    }

    fn build_websocket_client(&self) -> TungsteniteWs<HyperliquidCodec> {
        let ws_url = if self.config.testnet {
            TESTNET_WS_URL
        } else {
            MAINNET_WS_URL
        };

        let codec = HyperliquidCodec::new();
        TungsteniteWs::new(ws_url.to_string(), "hyperliquid".to_string(), codec)
    }
}

/// Convenience function to build a Hyperliquid connector
pub fn build_hyperliquid_connector(
    config: ExchangeConfig,
) -> Result<HyperliquidConnector<ReqwestRest, ()>, ExchangeError> {
    HyperliquidBuilder::new(config).build()
}

/// Convenience function to build a Hyperliquid connector with WebSocket support
pub fn build_hyperliquid_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<HyperliquidConnector<ReqwestRest, TungsteniteWs<HyperliquidCodec>>, ExchangeError> {
    HyperliquidBuilder::new(config)
        .with_websocket()
        .build_with_websocket()
}

/// Legacy compatibility function - create a connector from `ExchangeConfig`
pub fn create_hyperliquid_client(
    config: ExchangeConfig,
) -> Result<HyperliquidConnector<ReqwestRest, ()>, ExchangeError> {
    build_hyperliquid_connector(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let config = ExchangeConfig::new("test_key".to_string(), "test_secret".to_string());
        let builder = HyperliquidBuilder::new(config);

        // Test that we can create a builder
        assert!(!builder.enable_websocket);
        assert!(builder.vault_address.is_none());
    }

    #[test]
    fn test_builder_with_websocket() {
        let config = ExchangeConfig::new("test_key".to_string(), "test_secret".to_string());
        let builder = HyperliquidBuilder::new(config).with_websocket();

        assert!(builder.enable_websocket);
    }

    #[test]
    fn test_builder_with_vault() {
        let config = ExchangeConfig::new("test_key".to_string(), "test_secret".to_string());
        let builder = HyperliquidBuilder::new(config).with_vault_address("0x123".to_string());

        assert_eq!(builder.vault_address, Some("0x123".to_string()));
    }

    #[test]
    fn test_convenience_functions() {
        // Use read-only config for testing builder functionality
        let config = ExchangeConfig::read_only();

        // Test build_hyperliquid_connector
        let result = build_hyperliquid_connector(config);
        assert!(result.is_ok());
    }
}
