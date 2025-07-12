// Core modules - one responsibility per file
pub mod codec; // impl WsCodec (WebSocket dialect)
pub mod conversions; // String ↔︎ Decimal, Symbol, etc.
pub mod rest; // thin typed wrapper around RestClient
pub mod signer; // Hmac / Ed25519 / JWT authentication
pub mod types; // serde structs ← raw JSON

// Sub-trait implementations organized by responsibility
pub mod builder;
pub mod connector; // compose sub-traits // fluent builder → concrete connector

// Re-export main types for easier importing
pub use builder::{
    build_connector,
    build_connector_with_reconnection,
    build_connector_with_websocket,
    // Legacy exports for backward compatibility
    create_binance_perp_connector,
    create_binance_perp_connector_with_reconnection,
    create_binance_perp_connector_with_websocket,
    create_binance_perp_rest_connector,
};
pub use codec::{BinancePerpCodec, BinancePerpMessage};
pub use connector::BinancePerpConnector;
pub use conversions::*;
pub use rest::BinancePerpRestClient;
pub use signer::BinancePerpSigner;
pub use types::*;
