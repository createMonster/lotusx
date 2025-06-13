pub mod account;
pub mod client;
pub mod converters;
pub mod market_data;
pub mod trading;
pub mod types;

// Re-export main types for easier importing
pub use client::BinancePerpConnector;
pub use types::{
    BinancePerpBalance, BinancePerpExchangeInfo, BinancePerpFilter, BinancePerpKlineData,
    BinancePerpMarket, BinancePerpOrderRequest, BinancePerpOrderResponse, BinancePerpPosition,
    BinancePerpRestKline, BinancePerpWebSocketKline, BinancePerpWebSocketOrderBook,
    BinancePerpWebSocketTicker, BinancePerpWebSocketTrade,
};
