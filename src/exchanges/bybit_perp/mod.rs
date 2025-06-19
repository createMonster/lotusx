pub mod account;
pub mod client;
pub mod converters;
pub mod market_data;
pub mod trading;
pub mod types;

// Re-export main types for easier importing
pub use client::BybitPerpConnector;
pub use types::{
    BybitPerpCoinBalance,
    // Export new error types following HFT guidelines
    BybitPerpError,
    BybitPerpExchangeInfo,
    BybitPerpKlineData,
    BybitPerpLotSizeFilter,
    BybitPerpMarket,
    BybitPerpOrderRequest,
    BybitPerpOrderResponse,
    BybitPerpPriceFilter,
    BybitPerpRestKline,
    BybitPerpResultExt,
};
