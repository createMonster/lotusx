use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClientBuilder, RestClientConfig, TungsteniteWs};
use crate::exchanges::paradex::{
    codec::ParadexCodec, connector::ParadexConnector, signer::ParadexSigner,
};
use std::sync::Arc;

/// Create a Paradex connector with REST-only support
pub fn build_connector(
    config: ExchangeConfig,
) -> Result<ParadexConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://api.testnet.paradex.trade".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.paradex.trade".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "paradex".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let signer = Arc::new(ParadexSigner::new(config.secret_key().to_string())?);
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    Ok(ParadexConnector::new_without_ws(rest, config))
}

/// Create a Paradex connector with WebSocket support
pub fn build_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<
    ParadexConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<ParadexCodec>>,
    ExchangeError,
> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://api.testnet.paradex.trade".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.paradex.trade".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "paradex".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let signer = Arc::new(ParadexSigner::new(config.secret_key().to_string())?);
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    // Create WebSocket client
    let ws_url = if config.testnet {
        "wss://ws.testnet.paradex.trade/v1".to_string()
    } else {
        "wss://ws.paradex.trade/v1".to_string()
    };

    let ws = TungsteniteWs::new(ws_url, "paradex".to_string(), ParadexCodec);

    Ok(ParadexConnector::new(rest, ws, config))
}

/// Create a Paradex connector with WebSocket and auto-reconnection support
pub fn build_connector_with_reconnection(
    config: ExchangeConfig,
) -> Result<
    ParadexConnector<
        crate::core::kernel::ReqwestRest,
        crate::core::kernel::ReconnectWs<ParadexCodec, TungsteniteWs<ParadexCodec>>,
    >,
    ExchangeError,
> {
    // Determine base URL
    let base_url = if config.testnet {
        "https://api.testnet.paradex.trade".to_string()
    } else {
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.paradex.trade".to_string())
    };

    // Build REST client
    let rest_config = RestClientConfig::new(base_url, "paradex".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if credentials are provided
    if config.has_credentials() {
        let signer = Arc::new(ParadexSigner::new(config.secret_key().to_string())?);
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;

    // Create WebSocket client with auto-reconnection
    let ws_url = if config.testnet {
        "wss://ws.testnet.paradex.trade/v1".to_string()
    } else {
        "wss://ws.paradex.trade/v1".to_string()
    };

    let base_ws = TungsteniteWs::new(ws_url, "paradex".to_string(), ParadexCodec);
    let reconnect_ws = crate::core::kernel::ReconnectWs::new(base_ws)
        .with_max_reconnect_attempts(10)
        .with_reconnect_delay(std::time::Duration::from_secs(2))
        .with_auto_resubscribe(true);

    Ok(ParadexConnector::new(rest, reconnect_ws, config))
}

/// Legacy function for backward compatibility
pub fn create_paradex_connector(
    config: ExchangeConfig,
) -> Result<
    ParadexConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<ParadexCodec>>,
    ExchangeError,
> {
    build_connector_with_websocket(config)
}

/// Legacy function for backward compatibility
pub fn create_paradex_connector_with_websocket(
    config: ExchangeConfig,
    with_websocket: bool,
) -> Result<
    ParadexConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<ParadexCodec>>,
    ExchangeError,
> {
    // For backward compatibility, return a WebSocket-enabled connector regardless of the flag
    let _ = with_websocket; // Suppress unused variable warning
    build_connector_with_websocket(config)
}

/// Legacy function for backward compatibility
pub fn create_paradex_rest_connector(
    config: ExchangeConfig,
) -> Result<
    ParadexConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<ParadexCodec>>,
    ExchangeError,
> {
    build_connector_with_websocket(config)
}

/// Legacy function for backward compatibility
pub fn create_paradex_connector_with_reconnection(
    config: ExchangeConfig,
    with_websocket: bool,
) -> Result<
    ParadexConnector<
        crate::core::kernel::ReqwestRest,
        crate::core::kernel::ReconnectWs<ParadexCodec, TungsteniteWs<ParadexCodec>>,
    >,
    ExchangeError,
> {
    // For backward compatibility, return a reconnection-enabled connector regardless of the flag
    let _ = with_websocket; // Suppress unused variable warning
    build_connector_with_reconnection(config)
}
