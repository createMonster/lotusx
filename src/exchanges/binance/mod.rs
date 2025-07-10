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
    create_binance_connector,
    create_binance_connector_with_reconnection,
    create_binance_connector_with_websocket,
    create_binance_rest_connector,
};
pub use codec::{BinanceCodec, BinanceMessage};
pub use connector::{Account, BinanceConnector, MarketData, Trading};
pub use types::{
    BinanceAccountInfo, BinanceBalance, BinanceExchangeInfo, BinanceFilter, BinanceKlineData,
    BinanceMarket, BinanceOrderRequest, BinanceOrderResponse, BinanceRestKline,
    BinanceWebSocketKline, BinanceWebSocketOrderBook, BinanceWebSocketTicker,
    BinanceWebSocketTrade,
};

/// Helper function to create WebSocket stream identifiers for Binance
pub fn create_binance_stream_identifiers(
    symbols: &[String],
    subscription_types: &[crate::core::types::SubscriptionType],
) -> Vec<String> {
    codec::create_binance_stream_identifiers(symbols, subscription_types)
}
