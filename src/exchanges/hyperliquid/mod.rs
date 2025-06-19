pub mod account;
pub mod auth;
pub mod client;
pub mod converters;
pub mod market_data;
pub mod trading;
pub mod types;
pub mod websocket;

// Re-export main types for easier importing
pub use client::HyperliquidClient;
pub use types::{
    AssetInfo,
    CancelRequest,
    Candle,
    // Export new error types following HFT guidelines
    HyperliquidError,
    HyperliquidResultExt,
    L2Book,
    LimitOrder,
    ModifyRequest,
    OpenOrder,
    OrderRequest,
    OrderResponse,
    OrderType,
    TimeInForce,
    TriggerOrder,
    Universe,
    UserFill,
    UserState,
};
