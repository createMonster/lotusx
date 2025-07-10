pub mod account;
pub mod auth;
pub mod codec;
pub mod connector;
pub mod converters;
pub mod market_data;
pub mod trading;
pub mod types;

use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClientBuilder, RestClientConfig, TungsteniteWs};
use auth::BinanceSigner;
use std::sync::Arc;

// Re-export main types for easier importing
pub use codec::{BinanceCodec, BinanceMessage};
pub use connector::BinanceConnector;
pub use types::{
    BinanceAccountInfo, BinanceBalance, BinanceExchangeInfo, BinanceFilter, BinanceKlineData,
    BinanceMarket, BinanceOrderRequest, BinanceOrderResponse, BinanceRestKline,
    BinanceWebSocketKline, BinanceWebSocketOrderBook, BinanceWebSocketTicker,
    BinanceWebSocketTrade,
};

/// Create a Binance connector with REST support only
pub fn create_binance_connector(
    config: ExchangeConfig,
) -> Result<
    BinanceConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BinanceCodec>>,
    ExchangeError,
> {
    create_binance_connector_with_websocket(config, false)
}

/// Create a Binance connector with optional WebSocket support
pub fn create_binance_connector_with_websocket(
    config: ExchangeConfig,
    with_websocket: bool,
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
    let rest_config = RestClientConfig::new(base_url, "binance".to_string());
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

    // Build WebSocket client if requested
    let ws = if with_websocket {
        let ws_url = if config.testnet {
            "wss://testnet.binance.vision/ws".to_string()
        } else {
            "wss://stream.binance.com:443/ws".to_string()
        };

        let codec = BinanceCodec;
        Some(TungsteniteWs::new(ws_url, "binance".to_string(), codec))
    } else {
        None
    };

    Ok(BinanceConnector::new(rest, ws, config))
}

/// Create a Binance connector with REST support only (legacy compatibility)
pub fn create_binance_rest_connector(
    config: ExchangeConfig,
) -> Result<
    BinanceConnector<crate::core::kernel::ReqwestRest, TungsteniteWs<BinanceCodec>>,
    ExchangeError,
> {
    create_binance_connector(config)
}

/// Create a Binance connector with reconnection support
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
    let rest_config = RestClientConfig::new(base_url, "binance".to_string());
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

    // Build WebSocket client with reconnection if requested
    let ws = if with_websocket {
        let ws_url = if config.testnet {
            "wss://testnet.binance.vision/ws".to_string()
        } else {
            "wss://stream.binance.com:443/ws".to_string()
        };

        let codec = BinanceCodec;
        let base_ws = TungsteniteWs::new(ws_url, "binance".to_string(), codec);
        let reconnect_ws = crate::core::kernel::ReconnectWs::new(base_ws)
            .with_max_reconnect_attempts(10)
            .with_reconnect_delay(std::time::Duration::from_secs(2))
            .with_auto_resubscribe(true);
        Some(reconnect_ws)
    } else {
        None
    };

    Ok(BinanceConnector::new(rest, ws, config))
}

/// Helper function to create WebSocket stream identifiers for Binance
pub fn create_binance_stream_identifiers(
    symbols: &[String],
    subscription_types: &[crate::core::types::SubscriptionType],
) -> Vec<String> {
    codec::create_binance_stream_identifiers(symbols, subscription_types)
}
