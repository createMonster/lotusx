pub mod auth;
pub mod codec;
pub mod connector;
pub mod converters;
pub mod market_data;
pub mod types;

use crate::core::{
    config::ExchangeConfig,
    errors::ExchangeError,
    kernel::{Ed25519Signer, ReqwestRest, RestClientBuilder, RestClientConfig, TungsteniteWs},
};
use codec::BackpackCodec;
use std::sync::Arc;

// Re-export main types for easier importing
pub use auth::*;
pub use connector::BackpackConnector;
pub use converters::*;
pub use types::{
    BackpackBalance, BackpackExchangeInfo, BackpackKlineData, BackpackMarket, BackpackOrderRequest,
    BackpackOrderResponse, BackpackPosition, BackpackRestKline, BackpackWebSocketKline,
    BackpackWebSocketLiquidation, BackpackWebSocketMarkPrice, BackpackWebSocketOpenInterest,
    BackpackWebSocketOrderBook, BackpackWebSocketRFQ, BackpackWebSocketRFQUpdate,
    BackpackWebSocketTicker, BackpackWebSocketTrade,
};

/// Factory function to create a Backpack connector with kernel dependencies
pub fn create_backpack_connector(
    config: ExchangeConfig,
    with_websocket: bool,
) -> Result<connector::BackpackConnector<ReqwestRest, TungsteniteWs<BackpackCodec>>, ExchangeError>
{
    // Create REST client with Backpack configuration
    let rest_config = RestClientConfig::new(
        config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.backpack.exchange".to_string()),
        "backpack".to_string(),
    );

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if available
    if !config.api_key().is_empty() && !config.secret_key().is_empty() {
        let signer = Ed25519Signer::new(config.secret_key())?;
        rest_builder = rest_builder.with_signer(Arc::new(signer));
    }

    let rest = rest_builder.build()?;

    // Create WebSocket session if requested
    let ws = if with_websocket {
        let ws_url = "wss://ws.backpack.exchange".to_string();
        let codec = BackpackCodec::new();
        Some(TungsteniteWs::new(ws_url, "backpack".to_string(), codec))
    } else {
        None
    };

    Ok(connector::BackpackConnector::new(rest, ws, config))
}

/// Factory function to create a Backpack connector with reconnection support
pub fn create_backpack_connector_with_reconnection(
    config: ExchangeConfig,
    with_websocket: bool,
) -> Result<
    connector::BackpackConnector<
        ReqwestRest,
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
    );

    let mut rest_builder = RestClientBuilder::new(rest_config);

    // Add authentication if available
    if !config.api_key().is_empty() && !config.secret_key().is_empty() {
        let signer = Ed25519Signer::new(config.secret_key())?;
        rest_builder = rest_builder.with_signer(Arc::new(signer));
    }

    let rest = rest_builder.build()?;

    // Create WebSocket session with reconnection if requested
    let ws = if with_websocket {
        let ws_url = "wss://ws.backpack.exchange".to_string();
        let codec = BackpackCodec::new();
        let base_ws = TungsteniteWs::new(ws_url, "backpack".to_string(), codec);
        let reconnect_ws = crate::core::kernel::ReconnectWs::new(base_ws)
            .with_max_reconnect_attempts(10)
            .with_reconnect_delay(std::time::Duration::from_secs(2))
            .with_auto_resubscribe(true);
        Some(reconnect_ws)
    } else {
        None
    };

    Ok(connector::BackpackConnector::new(rest, ws, config))
}

/// Helper function to create WebSocket stream identifiers for Backpack
pub fn create_backpack_stream_identifiers(
    symbols: &[String],
    subscription_types: &[crate::core::types::SubscriptionType],
) -> Vec<String> {
    let mut streams = Vec::new();

    for symbol in symbols {
        for sub_type in subscription_types {
            match sub_type {
                crate::core::types::SubscriptionType::Ticker => {
                    streams.push(format!("ticker.{}", symbol));
                }
                crate::core::types::SubscriptionType::OrderBook { depth: _ } => {
                    streams.push(format!("depth.{}", symbol));
                }
                crate::core::types::SubscriptionType::Trades => {
                    streams.push(format!("trade.{}", symbol));
                }
                crate::core::types::SubscriptionType::Klines { interval } => {
                    streams.push(format!(
                        "kline.{}.{}",
                        interval.to_backpack_format(),
                        symbol
                    ));
                }
            }
        }
    }

    streams
}

/// Helper extension trait for `KlineInterval` to support Backpack format
pub trait BackpackKlineInterval {
    fn to_backpack_format(&self) -> &str;
}

impl BackpackKlineInterval for crate::core::types::KlineInterval {
    fn to_backpack_format(&self) -> &str {
        match self {
            Self::Minutes1 => "1m",
            Self::Minutes3 => "3m",
            Self::Minutes5 => "5m",
            Self::Minutes15 => "15m",
            Self::Minutes30 => "30m",
            Self::Hours1 => "1h",
            Self::Hours2 => "2h",
            Self::Hours4 => "4h",
            Self::Hours6 => "6h",
            Self::Hours8 => "8h",
            Self::Hours12 => "12h",
            Self::Days1 => "1d",
            Self::Days3 => "3d",
            Self::Weeks1 => "1w",
            Self::Months1 => "1M",
            Self::Seconds1 => "1s", // Backpack may not support seconds
        }
    }
}
