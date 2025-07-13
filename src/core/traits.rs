use crate::core::{
    errors::ExchangeError,
    types::{
        Balance, FundingRate, Kline, KlineInterval, Market, MarketDataType, OrderRequest,
        OrderResponse, Position, SubscriptionType, WebSocketConfig,
    },
};
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
pub trait MarketDataSource {
    /// Get all available markets/trading pairs
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError>;

    /// Subscribe to market data via WebSocket
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError>;

    /// Get WebSocket endpoint URL for market data
    fn get_websocket_url(&self) -> String;

    /// Get historical k-lines/candlestick data
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError>;
}

#[async_trait]
pub trait OrderPlacer {
    /// Place a new order
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError>;

    /// Cancel an existing order
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError>;

    /// Modify an existing order
    async fn modify_order(
        &self,
        _order_id: String,
        _order: OrderRequest,
    ) -> Result<OrderResponse, ExchangeError> {
        // Default implementation returns an error, so existing connectors
        // don't break.
        Err(ExchangeError::Other(
            "Order modification not supported".to_string(),
        ))
    }
}

#[async_trait]
pub trait AccountInfo {
    // Account-related methods can be added here as needed

    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError>;
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError>;
}

/// Trait for funding rate operations (PERPETUAL EXCHANGES ONLY)
#[async_trait]
pub trait FundingRateSource {
    /// Get current funding rates for one or more symbols
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError>;

    /// Get all available funding rates from the exchange
    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError>;

    /// Get historical funding rates for a symbol
    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError>;
}

// BACKWARD-COMPATIBLE trait composition (NON-BREAKING APPROACH)
#[async_trait]
pub trait FundingRateConnector: MarketDataSource + FundingRateSource {}

// Optional: Enhanced connector for perpetual exchanges
#[async_trait]
pub trait PerpetualExchangeConnector: ExchangeConnector + FundingRateSource {}

// Optional: Keep a composite trait for convenience when you need all functionality
#[async_trait]
pub trait ExchangeConnector: MarketDataSource + OrderPlacer + AccountInfo {}
