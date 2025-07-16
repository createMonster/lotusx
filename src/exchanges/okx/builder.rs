use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClientBuilder, RestClientConfig, TungsteniteWs};
use crate::exchanges::okx::{codec::OkxCodec, connector::OkxConnector, signer::OkxSigner};
use std::sync::Arc;

/// Create an OKX connector with REST-only support
pub fn build_connector(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://www.okx.com".to_string() // OKX doesn't have a separate testnet URL
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://www.okx.com".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "okx".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let passphrase = config
            .passphrase()
            .ok_or_else(|| ExchangeError::AuthError("OKX requires passphrase".to_string()))?;

        let signer = Arc::new(OkxSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
            passphrase.to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    Ok(OkxConnector::new_without_ws(rest, config))
}

/// Create an OKX connector with WebSocket support
pub fn build_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<OkxCodec>>, ExchangeError>
{
    // Determine URLs
    let rest_base_url = if config.testnet {
        "https://www.okx.com".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://www.okx.com".to_string())
    };

    let ws_url = if config.testnet {
        "wss://ws.okx.com:8443/ws/v5/public".to_string() // Public channel for testnet
    } else {
        config
            .ws_url
            .clone()
            .unwrap_or_else(|| "wss://ws.okx.com:8443/ws/v5/public".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(rest_base_url, "okx".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let passphrase = config
            .passphrase()
            .ok_or_else(|| ExchangeError::AuthError("OKX requires passphrase".to_string()))?;

        let signer = Arc::new(OkxSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
            passphrase.to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    // Build WebSocket client
    let codec = OkxCodec::new();
    let ws = TungsteniteWs::new(ws_url, codec)?;

    Ok(OkxConnector::new_with_ws(rest, ws, config))
}

/// Create an OKX connector with WebSocket support and reconnection
pub fn build_connector_with_reconnection(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<OkxCodec>>, ExchangeError>
{
    // For now, this is the same as build_connector_with_websocket
    // Future enhancement could add reconnection logic
    build_connector_with_websocket(config)
}

// Legacy compatibility functions

/// Legacy function to create OKX connector (REST only)
pub fn create_okx_connector(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    build_connector(config)
}

/// Legacy function to create OKX REST connector
pub fn create_okx_rest_connector(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    build_connector(config)
}

/// Legacy function to create OKX connector with WebSocket
pub fn create_okx_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<OkxConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<OkxCodec>>, ExchangeError>
{
    build_connector_with_websocket(config)
}

/// Legacy function to create OKX connector with reconnection
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
        let config = ExchangeConfig::new("okx".to_string(), false);
        let result = build_connector(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_okx_connector_missing_passphrase() {
        let mut config = ExchangeConfig::new("okx".to_string(), false);
        config = config.with_credentials("test_key".to_string(), "test_secret".to_string());

        let result = build_connector(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("passphrase"));
    }
}
