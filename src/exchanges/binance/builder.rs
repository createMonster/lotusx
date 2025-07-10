use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClientBuilder, RestClientConfig, TungsteniteWs};
use crate::exchanges::binance::{
    codec::BinanceCodec, connector::BinanceConnector, signer::BinanceSigner,
};
use std::sync::Arc;

/// Create a Binance connector with REST-only support
pub fn build_connector(
    config: ExchangeConfig,
) -> Result<BinanceConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://testnet.binance.vision".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.binance.com".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "binance".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let signer = Arc::new(BinanceSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    Ok(BinanceConnector::new_without_ws(rest, config))
}

/// Create a Binance connector with WebSocket support
pub fn build_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<
    BinanceConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BinanceCodec>>,
    ExchangeError,
> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://testnet.binance.vision".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.binance.com".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "binance".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let signer = Arc::new(BinanceSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    // Create WebSocket client
    let ws_url = if config.testnet {
        "wss://testnet.binance.vision/ws".to_string()
    } else {
        "wss://stream.binance.com:443/ws".to_string()
    };

    let ws = TungsteniteWs::new(ws_url, "binance".to_string(), BinanceCodec);

    Ok(BinanceConnector::new(rest, ws, config))
}

/// Create a Binance connector with WebSocket and auto-reconnection support
pub fn build_connector_with_reconnection(
    config: ExchangeConfig,
) -> Result<
    BinanceConnector<
        crate::core::kernel::ReqwestRest,
        crate::core::kernel::ReconnectWs<BinanceCodec, TungsteniteWs<BinanceCodec>>,
    >,
    ExchangeError,
> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://testnet.binance.vision".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.binance.com".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "binance".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let signer = Arc::new(BinanceSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    // Create WebSocket client with auto-reconnection
    let ws_url = if config.testnet {
        "wss://testnet.binance.vision/ws".to_string()
    } else {
        "wss://stream.binance.com:443/ws".to_string()
    };

    let base_ws = TungsteniteWs::new(ws_url, "binance".to_string(), BinanceCodec);
    let reconnect_ws = crate::core::kernel::ReconnectWs::new(base_ws)
        .with_max_reconnect_attempts(10)
        .with_reconnect_delay(std::time::Duration::from_secs(2))
        .with_auto_resubscribe(true);

    Ok(BinanceConnector::new(rest, reconnect_ws, config))
}

/// Legacy function for backward compatibility
pub fn create_binance_connector(
    config: ExchangeConfig,
) -> Result<
    BinanceConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BinanceCodec>>,
    ExchangeError,
> {
    build_connector_with_websocket(config)
}

/// Legacy function for backward compatibility
pub fn create_binance_connector_with_websocket(
    config: ExchangeConfig,
    with_websocket: bool,
) -> Result<
    BinanceConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BinanceCodec>>,
    ExchangeError,
> {
    // For backward compatibility, return a WebSocket-enabled connector regardless of the flag
    let _ = with_websocket; // Suppress unused variable warning
    build_connector_with_websocket(config)
}

/// Legacy function for backward compatibility
pub fn create_binance_rest_connector(
    config: ExchangeConfig,
) -> Result<
    BinanceConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BinanceCodec>>,
    ExchangeError,
> {
    build_connector_with_websocket(config)
}

/// Legacy function for backward compatibility
pub fn create_binance_connector_with_reconnection(
    config: ExchangeConfig,
    with_websocket: bool,
) -> Result<
    BinanceConnector<
        crate::core::kernel::ReqwestRest,
        crate::core::kernel::ReconnectWs<BinanceCodec, TungsteniteWs<BinanceCodec>>,
    >,
    ExchangeError,
> {
    // For backward compatibility, return a reconnection-enabled connector regardless of the flag
    let _ = with_websocket; // Suppress unused variable warning
    build_connector_with_reconnection(config)
}
