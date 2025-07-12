use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClientBuilder, RestClientConfig, TungsteniteWs};
use crate::exchanges::binance_perp::{
    codec::BinancePerpCodec, connector::BinancePerpConnector, signer::BinancePerpSigner,
};
use std::sync::Arc;

/// Create a Binance Perpetual connector with REST-only support
pub fn build_connector(
    config: ExchangeConfig,
) -> Result<BinancePerpConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://testnet.binancefuture.com".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://fapi.binance.com".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "binance_perp".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let signer = Arc::new(BinancePerpSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    Ok(BinancePerpConnector::new_without_ws(rest, config))
}

/// Create a Binance Perpetual connector with WebSocket support
pub fn build_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<
    BinancePerpConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BinancePerpCodec>>,
    ExchangeError,
> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://testnet.binancefuture.com".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://fapi.binance.com".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "binance_perp".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let signer = Arc::new(BinancePerpSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    // Create WebSocket client
    let ws_url = if config.testnet {
        "wss://stream.binancefuture.com/ws".to_string()
    } else {
        "wss://fstream.binance.com/ws".to_string()
    };

    let ws = TungsteniteWs::new(ws_url, "binance_perp".to_string(), BinancePerpCodec);

    Ok(BinancePerpConnector::new(rest, ws, config))
}

/// Create a Binance Perpetual connector with WebSocket and auto-reconnection support
pub fn build_connector_with_reconnection(
    config: ExchangeConfig,
) -> Result<
    BinancePerpConnector<
        crate::core::kernel::ReqwestRest,
        crate::core::kernel::ReconnectWs<BinancePerpCodec, TungsteniteWs<BinancePerpCodec>>,
    >,
    ExchangeError,
> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://testnet.binancefuture.com".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://fapi.binance.com".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "binance_perp".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let signer = Arc::new(BinancePerpSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    // Create WebSocket client with auto-reconnection
    let ws_url = if config.testnet {
        "wss://stream.binancefuture.com/ws".to_string()
    } else {
        "wss://fstream.binance.com/ws".to_string()
    };

    let base_ws = TungsteniteWs::new(ws_url, "binance_perp".to_string(), BinancePerpCodec);
    let reconnect_ws = crate::core::kernel::ReconnectWs::new(base_ws)
        .with_max_reconnect_attempts(10)
        .with_reconnect_delay(std::time::Duration::from_secs(2))
        .with_auto_resubscribe(true);

    Ok(BinancePerpConnector::new(rest, reconnect_ws, config))
}

/// Legacy function for backward compatibility
pub fn create_binance_perp_connector(
    config: ExchangeConfig,
) -> Result<BinancePerpConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    build_connector(config)
}

/// Legacy function for backward compatibility
pub fn create_binance_perp_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<
    BinancePerpConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BinancePerpCodec>>,
    ExchangeError,
> {
    build_connector_with_websocket(config)
}

/// Legacy function for backward compatibility
pub fn create_binance_perp_connector_with_reconnection(
    config: ExchangeConfig,
) -> Result<
    BinancePerpConnector<
        crate::core::kernel::ReqwestRest,
        crate::core::kernel::ReconnectWs<BinancePerpCodec, TungsteniteWs<BinancePerpCodec>>,
    >,
    ExchangeError,
> {
    build_connector_with_reconnection(config)
}

/// Legacy function for backward compatibility
pub fn create_binance_perp_rest_connector(
    config: ExchangeConfig,
) -> Result<BinancePerpConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    build_connector(config)
}
