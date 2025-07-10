use crate::core::{
    config::ExchangeConfig,
    errors::ExchangeError,
    kernel::{Ed25519Signer, RestClientBuilder, RestClientConfig, TungsteniteWs},
};
use crate::exchanges::backpack::{codec::BackpackCodec, connector::BackpackConnector};
use std::sync::Arc;

/// Create a Backpack connector with REST-only support
pub fn build_connector(
    config: ExchangeConfig,
) -> Result<BackpackConnector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    // Create REST client with Backpack configuration
    let rest_config = RestClientConfig::new(
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.backpack.exchange".to_string()),
        "backpack".to_string(),
    )
    .with_timeout(30)
    .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if available
    if !config.api_key().is_empty() && !config.secret_key().is_empty() {
        let signer = Ed25519Signer::new(config.secret_key())?;
        rest_builder = rest_builder.with_signer(Arc::new(signer));
    }

    let rest = rest_builder.build()?;

    Ok(BackpackConnector::new_without_ws(rest, config))
}

/// Create a Backpack connector with WebSocket support
pub fn build_connector_with_websocket(
    config: ExchangeConfig,
) -> Result<
    BackpackConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BackpackCodec>>,
    ExchangeError,
> {
    // Create REST client with Backpack configuration
    let rest_config = RestClientConfig::new(
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.backpack.exchange".to_string()),
        "backpack".to_string(),
    )
    .with_timeout(30)
    .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if available
    if !config.api_key().is_empty() && !config.secret_key().is_empty() {
        let signer = Ed25519Signer::new(config.secret_key())?;
        rest_builder = rest_builder.with_signer(Arc::new(signer));
    }

    let rest = rest_builder.build()?;

    // Create WebSocket client
    let ws_url = "wss://ws.backpack.exchange".to_string();
    let codec = BackpackCodec::new();
    let ws = TungsteniteWs::new(ws_url, "backpack".to_string(), codec);

    Ok(BackpackConnector::new(rest, ws, config))
}

/// Create a Backpack connector with WebSocket and auto-reconnection support
pub fn build_connector_with_reconnection(
    config: ExchangeConfig,
) -> Result<
    BackpackConnector<
        crate::core::kernel::ReqwestRest,
        crate::core::kernel::ReconnectWs<BackpackCodec, TungsteniteWs<BackpackCodec>>,
    >,
    ExchangeError,
> {
    // Create REST client with Backpack configuration
    let rest_config = RestClientConfig::new(
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.backpack.exchange".to_string()),
        "backpack".to_string(),
    )
    .with_timeout(30)
    .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if available
    if !config.api_key().is_empty() && !config.secret_key().is_empty() {
        let signer = Ed25519Signer::new(config.secret_key())?;
        rest_builder = rest_builder.with_signer(Arc::new(signer));
    }

    let rest = rest_builder.build()?;

    // Create WebSocket client with auto-reconnection
    let ws_url = "wss://ws.backpack.exchange".to_string();
    let codec = BackpackCodec::new();
    let base_ws = TungsteniteWs::new(ws_url, "backpack".to_string(), codec);
    let reconnect_ws = crate::core::kernel::ReconnectWs::new(base_ws)
        .with_max_reconnect_attempts(10)
        .with_reconnect_delay(std::time::Duration::from_secs(2))
        .with_auto_resubscribe(true);

    Ok(BackpackConnector::new(rest, reconnect_ws, config))
}

/// Legacy function for backward compatibility
pub fn create_backpack_connector(
    config: ExchangeConfig,
    with_websocket: bool,
) -> Result<
    BackpackConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BackpackCodec>>,
    ExchangeError,
> {
    // For backward compatibility, return a WebSocket-enabled connector regardless of the flag
    let _ = with_websocket; // Suppress unused variable warning
    build_connector_with_websocket(config)
}

/// Legacy function for backward compatibility
pub fn create_backpack_connector_with_reconnection(
    config: ExchangeConfig,
    with_websocket: bool,
) -> Result<
    BackpackConnector<
        crate::core::kernel::ReqwestRest,
        crate::core::kernel::ReconnectWs<BackpackCodec, TungsteniteWs<BackpackCodec>>,
    >,
    ExchangeError,
> {
    // For backward compatibility, return a reconnection-enabled connector regardless of the flag
    let _ = with_websocket; // Suppress unused variable warning
    build_connector_with_reconnection(config)
}
