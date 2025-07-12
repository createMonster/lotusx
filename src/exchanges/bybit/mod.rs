pub mod codec;
pub mod conversions;
pub mod signer;
pub mod types;

pub mod builder;
pub mod connector;
pub mod rest;

// Re-export main components
pub use builder::{
    build_connector,
    build_connector_with_reconnection,
    build_connector_with_websocket,
    // Legacy compatibility exports
    create_bybit_connector,
    create_bybit_connector_with_reconnection,
};
pub use codec::BybitCodec;
pub use connector::{Account, BybitConnector, MarketData, Trading};
pub use types::{
    BybitAccountInfo, BybitCoinBalance, BybitError, BybitExchangeInfo, BybitFilter, BybitKlineData,
    BybitLotSizeFilter, BybitMarket, BybitPriceFilter, BybitResultExt,
};

// Helper functions for stream identifiers
pub fn create_bybit_stream_identifiers(
    symbols: &[String],
    subscription_types: &[crate::core::types::SubscriptionType],
) -> Vec<String> {
    let mut streams = Vec::new();

    for symbol in symbols {
        for sub_type in subscription_types {
            match sub_type {
                crate::core::types::SubscriptionType::Ticker => {
                    streams.push(format!("tickers.{}", symbol));
                }
                crate::core::types::SubscriptionType::Trades => {
                    streams.push(format!("publicTrade.{}", symbol));
                }
                crate::core::types::SubscriptionType::OrderBook { depth: _ } => {
                    streams.push(format!("orderbook.{}.200ms", symbol));
                }
                crate::core::types::SubscriptionType::Klines { interval } => {
                    let interval_str =
                        crate::exchanges::bybit::conversions::kline_interval_to_bybit_string(
                            *interval,
                        );
                    streams.push(format!("kline.{}.{}", interval_str, symbol));
                }
            }
        }
    }

    streams
}
