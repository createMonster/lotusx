pub mod account;
pub mod auth;
pub mod client;
pub mod converters;
pub mod market_data;
pub mod trading;
pub mod types;

// Re-export main types for easier importing
pub use client::BinanceConnector;
pub use types::{
    BinanceAccountInfo, BinanceBalance, BinanceExchangeInfo, BinanceFilter, BinanceKlineData,
    BinanceMarket, BinanceOrderRequest, BinanceOrderResponse, BinanceWebSocketKline,
    BinanceWebSocketOrderBook, BinanceWebSocketTicker, BinanceWebSocketTrade,
};
