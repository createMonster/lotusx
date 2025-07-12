// Core kernel-compatible modules
pub mod codec;
pub mod conversions;
pub mod rest;
pub mod signer;
pub mod types;

// Connector modules
pub mod connector;

// Builder and factory
pub mod builder;

// Re-export main types for easier importing
pub use builder::{
    build_hyperliquid_connector, build_hyperliquid_connector_with_websocket,
    create_hyperliquid_client, HyperliquidBuilder,
};
pub use connector::HyperliquidConnector;
pub use rest::HyperliquidRest;
pub use signer::HyperliquidSigner;
pub use types::{
    AssetInfo, CancelRequest, Candle, HyperliquidError, HyperliquidResultExt, L2Book, LimitOrder,
    ModifyRequest, OpenOrder, OrderRequest, OrderResponse, OrderType, TimeInForce, TriggerOrder,
    Universe, UserFill, UserState,
};

// Export codec for WebSocket usage
pub use codec::{HyperliquidCodec, HyperliquidWsMessage};

// Export conversions
pub use conversions::*;
