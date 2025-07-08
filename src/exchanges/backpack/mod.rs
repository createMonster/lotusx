pub mod account;
pub mod auth;
pub mod client;
pub mod converters;
pub mod market_data;
pub mod trading;
pub mod types;

// Re-export main types for easier importing
pub use auth::*;
pub use client::BackpackConnector;
pub use converters::*;
pub use types::{
    BackpackBalance, BackpackExchangeInfo, BackpackKlineData, BackpackMarket, BackpackOrderRequest,
    BackpackOrderResponse, BackpackPosition, BackpackRestKline, BackpackWebSocketKline,
    BackpackWebSocketLiquidation, BackpackWebSocketMarkPrice, BackpackWebSocketOpenInterest,
    BackpackWebSocketOrderBook, BackpackWebSocketRFQ, BackpackWebSocketRFQUpdate,
    BackpackWebSocketTicker, BackpackWebSocketTrade,
};
