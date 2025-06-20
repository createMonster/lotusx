use crate::core::{
    errors::ExchangeError,
    types::{
        Balance, Kline, Market, MarketDataType, OrderRequest, OrderResponse, Position,
        SubscriptionType, WebSocketConfig,
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
        interval: String,
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
}

#[async_trait]
pub trait AccountInfo {
    // Account-related methods can be added here as needed

    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError>;
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError>;
}

// Optional: Keep a composite trait for convenience when you need all functionality
#[async_trait]
pub trait ExchangeConnector: MarketDataSource + OrderPlacer + AccountInfo {}
