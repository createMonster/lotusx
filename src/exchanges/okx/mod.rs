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
    create_okx_connector,
    create_okx_connector_with_reconnection,
    create_okx_connector_with_websocket,
    create_okx_rest_connector,
};
pub use codec::{OkxCodec, OkxMessage};
pub use connector::{Account, MarketData, OkxConnector, Trading};
pub use types::{
    OkxAccountInfo, OkxBalance, OkxKline, OkxMarket, OkxOrder, OkxOrderBook, OkxOrderRequest,
    OkxOrderResponse, OkxResponse, OkxTicker, OkxTrade, OkxWsChannel, OkxWsRequest, OkxWsResponse,
};

/// Helper function to create WebSocket stream identifiers for OKX
pub fn create_okx_stream_identifiers(
    symbols: &[String],
    subscription_types: &[crate::core::types::SubscriptionType],
) -> Vec<String> {
    codec::create_okx_stream_identifiers(symbols, subscription_types)
}
