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
    create_paradex_connector,
    create_paradex_connector_with_reconnection,
    create_paradex_connector_with_websocket,
    create_paradex_rest_connector,
};
pub use codec::{create_subscription_channel, ParadexCodec};
pub use connector::{Account, MarketData, ParadexConnector, Trading};
pub use signer::ParadexSigner;

// Helper functions for creating stream identifiers
pub fn create_paradex_stream_identifiers(
    symbols: &[String],
    subscription_types: &[crate::core::types::SubscriptionType],
) -> Vec<String> {
    let mut streams = Vec::new();
    for symbol in symbols {
        for sub_type in subscription_types {
            streams.push(create_subscription_channel(symbol, sub_type));
        }
    }
    streams
}
