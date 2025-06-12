pub mod auth;
pub mod client;
pub mod types;
pub mod converters;
pub mod trading;
pub mod account;
pub mod websocket;
pub mod market_data;

// Re-export main types for easier importing
pub use client::HyperliquidClient;
pub use types::{
    AssetInfo, CancelRequest, Candle, L2Book, LimitOrder, ModifyRequest, OpenOrder, OrderRequest,
    OrderResponse, OrderType, TimeInForce, TriggerOrder, Universe, UserFill, UserState,
};
