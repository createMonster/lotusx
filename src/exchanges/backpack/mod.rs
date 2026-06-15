pub mod codec;
pub mod conversions;
pub mod signer;
pub mod types;

pub mod builder;
pub mod connector;
pub mod rest;

use crate::exchanges::backpack::conversions::kline_interval_to_backpack_string;

// Re-export main components
pub use builder::{
    build_connector,
    build_connector_with_reconnection,
    build_connector_with_websocket,
    // Legacy compatibility exports
    create_backpack_connector,
    create_backpack_connector_with_reconnection,
};
pub use codec::BackpackCodec;
pub use connector::{Account, BackpackConnector, MarketData, Trading};
pub use types::{
    BackpackBalance, BackpackExchangeInfo, BackpackKlineData, BackpackMarket, BackpackOrderRequest,
    BackpackOrderResponse, BackpackPosition, BackpackRestKline, BackpackWebSocketKline,
    BackpackWebSocketLiquidation, BackpackWebSocketMarkPrice, BackpackWebSocketOpenInterest,
    BackpackWebSocketOrderBook, BackpackWebSocketRFQ, BackpackWebSocketRFQUpdate,
    BackpackWebSocketTicker, BackpackWebSocketTrade,
};

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
                        kline_interval_to_backpack_string(*interval),
                        symbol
                    ));
                }
            }
        }
    }

    streams
}
