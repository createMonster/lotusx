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
                crate::core::types::SubscriptionType::OrderBook { depth } => {
                    let depth = depth.unwrap_or(1);
                    streams.push(format!("orderbook.{}.{}", depth, symbol));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{KlineInterval, SubscriptionType};

    #[test]
    fn stream_identifiers_match_bybit_v5_topics() {
        let symbols = vec!["BTCUSDT".to_string()];
        let subscription_types = vec![
            SubscriptionType::Ticker,
            SubscriptionType::Trades,
            SubscriptionType::OrderBook { depth: Some(50) },
            SubscriptionType::Klines {
                interval: KlineInterval::Minutes1,
            },
        ];

        let streams = create_bybit_stream_identifiers(&symbols, &subscription_types);

        assert_eq!(
            streams,
            vec![
                "tickers.BTCUSDT",
                "publicTrade.BTCUSDT",
                "orderbook.50.BTCUSDT",
                "kline.1.BTCUSDT"
            ]
        );
    }

    #[test]
    fn orderbook_without_depth_uses_level_one_topic() {
        let symbols = vec!["BTCUSDT".to_string()];
        let subscription_types = vec![SubscriptionType::OrderBook { depth: None }];

        let streams = create_bybit_stream_identifiers(&symbols, &subscription_types);

        assert_eq!(streams, vec!["orderbook.1.BTCUSDT"]);
    }
}
