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
    build_connector_with_websocket,
    // Legacy compatibility exports
    create_bybit_perp_connector,
};
pub use codec::{create_bybit_perp_stream_identifiers, BybitPerpCodec};
pub use connector::{Account, BybitPerpConnector, MarketData, Trading};

// Helper functions for backward compatibility
pub use types::{
    BybitPerpCoinBalance, BybitPerpError, BybitPerpExchangeInfo, BybitPerpKlineData,
    BybitPerpLotSizeFilter, BybitPerpMarket, BybitPerpOrderRequest, BybitPerpOrderResponse,
    BybitPerpPriceFilter, BybitPerpRestKline, BybitPerpResultExt,
};
