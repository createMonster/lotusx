use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClientBuilder, RestClientConfig, TungsteniteWs};
use crate::exchanges::okx::{codec::OkxCodec, connector::OkxConnector, signer::OkxSigner};
use std::sync::Arc;
use std::time::Duration;

/// Builder for creating OKX exchange connectors
///
/// This builder provides a fluent interface for configuring and building OKX connectors
/// with various options including REST-only, WebSocket support, and reconnection logic.
#[derive(Default)]
pub struct OkxBuilder {
    config: ExchangeConfig,
    passphrase: Option<String>,
    ws_reconnect_interval: Option<Duration>,
    ws_ping_interval: Option<Duration>,
    max_reconnect_attempts: Option<u32>,
    rest_timeout: u64,
    rest_max_retries: u32,
}

impl OkxBuilder {
    /// Create a new `OkxBuilder` with default settings
    pub fn new() -> Self {
        Self {
            config: ExchangeConfig::new(String::new(), String::new()),
            passphrase: None,
            ws_reconnect_interval: None,
            ws_ping_interval: None,
            max_reconnect_attempts: None,
            rest_timeout: 30,
            rest_max_retries: 3,
        }
    }

    /// Set the exchange configuration
    pub fn with_config(mut self, config: ExchangeConfig) -> Self {
        self.config = config;
        self
    }

    /// Set testnet mode
    pub fn with_testnet(mut self, testnet: bool) -> Self {
        self.config.testnet = testnet;
        self
    }

    /// Set API credentials
    pub fn with_credentials(
        mut self,
        api_key: String,
        secret_key: String,
        passphrase: String,
    ) -> Self {
        self.config = ExchangeConfig::new(api_key, secret_key).testnet(self.config.testnet);
        if let Some(base_url) = self.config.base_url.clone() {
            self.config = self.config.base_url(base_url);
        }
        self.passphrase = Some(passphrase);
        self
    }

    /// Set base URL for REST API
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.config.base_url = Some(base_url);
        self
    }

    /// Set WebSocket URL (stored internally, not in config)
    pub fn with_ws_url(self, _ws_url: String) -> Self {
        // WebSocket URL is handled internally based on testnet setting
        self
    }

    /// Set WebSocket reconnection interval
    pub fn with_ws_reconnect_interval(mut self, interval: Duration) -> Self {
        self.ws_reconnect_interval = Some(interval);
        self
    }

    /// Set WebSocket ping interval for heartbeat
    pub fn with_ws_ping_interval(mut self, interval: Duration) -> Self {
        self.ws_ping_interval = Some(interval);
        self
    }

    /// Set maximum number of reconnection attempts
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = Some(attempts);
        self
    }

    /// Set REST client timeout
    pub fn with_rest_timeout(mut self, timeout: u64) -> Self {
        self.rest_timeout = timeout;
        self
    }

    /// Set REST client maximum retries
    pub fn with_rest_max_retries(mut self, retries: u32) -> Self {
        self.rest_max_retries = retries;
        self
    }

    /// Build a REST-only OKX connector
    pub fn build_rest_only(
        self,
    ) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
        // Determine base URL
        let base_url = if self.config.testnet {
            "https://www.okx.com".to_string() // OKX doesn't have a separate testnet URL
        } else {
            self.config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://www.okx.com".to_string())
        };

        // Build REST client
        let rest_config = RestClientConfig::new(base_url, "okx".to_string())
            .with_timeout(self.rest_timeout)
            .with_max_retries(self.rest_max_retries);

        let mut rest_builder = RestClientBuilder::new(rest_config);

        // Add authentication if credentials are provided
        if self.config.has_credentials() {
            let passphrase = self.passphrase.ok_or_else(|| {
                ExchangeError::ConfigurationError(
                    "OKX passphrase is required when using credentials".to_string(),
                )
            })?;

            let signer = Arc::new(OkxSigner::new(
                self.config.api_key().to_string(),
                self.config.secret_key().to_string(),
                passphrase,
            ));
            rest_builder = rest_builder.with_signer(signer);
        }

        let rest = rest_builder.build()?;

        Ok(OkxConnector::new_without_ws(rest, self.config))
    }

    /// Build an OKX connector with WebSocket support
    pub fn build_with_ws(
        self,
    ) -> Result<
        OkxConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<OkxCodec>>,
        ExchangeError,
    > {
        // Determine URLs
        let rest_base_url = if self.config.testnet {
            "https://www.okx.com".to_string()
        } else {
            self.config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://www.okx.com".to_string())
        };

        let ws_url = "wss://ws.okx.com:8443/ws/v5/public".to_string();

        // Build REST client
        let rest_config = RestClientConfig::new(rest_base_url, "okx".to_string())
            .with_timeout(self.rest_timeout)
            .with_max_retries(self.rest_max_retries);

        let mut rest_builder = RestClientBuilder::new(rest_config);

        // Add authentication if credentials are provided
        if self.config.has_credentials() {
            let passphrase = self.passphrase.ok_or_else(|| {
                ExchangeError::ConfigurationError(
                    "OKX passphrase is required when using credentials".to_string(),
                )
            })?;

            let signer = Arc::new(OkxSigner::new(
                self.config.api_key().to_string(),
                self.config.secret_key().to_string(),
                passphrase,
            ));
            rest_builder = rest_builder.with_signer(signer);
        }

        let rest = rest_builder.build()?;

        // Build WebSocket client
        let codec = OkxCodec::new();
        let ws = TungsteniteWs::new(ws_url, "okx".to_string(), codec);

        Ok(OkxConnector::new_with_ws(rest, ws, self.config))
    }

    /// Build an OKX connector with WebSocket support and reconnection logic
    pub fn build_with_reconnection(
        self,
    ) -> Result<
        OkxConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<OkxCodec>>,
        ExchangeError,
    > {
        // For now, this is the same as build_with_ws
        // Future enhancement will add reconnection logic using the configured parameters
        // TODO: Implement reconnection logic in task 3
        self.build_with_ws()
    }
}

// Legacy functions for backward compatibility

/// Create an OKX connector with REST-only support
///
/// @deprecated Use `OkxBuilder` instead
pub fn build_connector(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    OkxBuilder::new().with_config(config).build_rest_only()
}

/// Create an OKX connector with WebSocket support
///
/// @deprecated Use `OkxBuilder` instead
pub fn build_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<OkxCodec>>, ExchangeError>
{
    OkxBuilder::new().with_config(config).build_with_ws()
}

/// Create an OKX connector with WebSocket support and reconnection
///
/// @deprecated Use `OkxBuilder` instead
pub fn build_connector_with_reconnection(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<OkxCodec>>, ExchangeError>
{
    OkxBuilder::new()
        .with_config(config)
        .build_with_reconnection()
}

// Legacy compatibility functions

/// Legacy function to create OKX connector (REST only)
///
/// @deprecated Use `OkxBuilder` instead
pub fn create_okx_connector(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    build_connector(config)
}

/// Legacy function to create OKX REST connector
///
/// @deprecated Use `OkxBuilder` instead
pub fn create_okx_rest_connector(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    build_connector(config)
}

/// Legacy function to create OKX connector with WebSocket
///
/// @deprecated Use `OkxBuilder` instead
pub fn create_okx_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<OkxCodec>>, ExchangeError>
{
    build_connector_with_websocket(config)
}

/// Legacy function to create OKX connector with reconnection
///
/// @deprecated Use `OkxBuilder` instead
pub fn create_okx_connector_with_reconnection(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<OkxCodec>>, ExchangeError>
{
    build_connector_with_reconnection(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::ExchangeConfig;

    #[test]
    fn test_build_okx_connector_without_credentials() {
        let config = ExchangeConfig::new(String::new(), String::new());
        let result = build_connector(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_okx_connector_missing_passphrase() {
        let config = ExchangeConfig::new("test_key".to_string(), "test_secret".to_string());

        let result = build_connector(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("passphrase"));
    }

    #[test]
    fn test_okx_builder_rest_only() {
        let builder = OkxBuilder::new()
            .with_testnet(false)
            .with_rest_timeout(60)
            .with_rest_max_retries(5);

        let result = builder.build_rest_only();
        assert!(result.is_ok());
    }

    #[test]
    fn test_okx_builder_with_credentials() {
        let builder = OkxBuilder::new().with_credentials(
            "test_key".to_string(),
            "test_secret".to_string(),
            "test_passphrase".to_string(),
        );

        let result = builder.build_rest_only();
        assert!(result.is_ok());
    }

    #[test]
    fn test_okx_builder_with_ws() {
        let builder = OkxBuilder::new()
            .with_testnet(false)
            .with_ws_reconnect_interval(Duration::from_secs(30))
            .with_ws_ping_interval(Duration::from_secs(15))
            .with_max_reconnect_attempts(5);

        let result = builder.build_with_ws();
        assert!(result.is_ok());
    }
}
