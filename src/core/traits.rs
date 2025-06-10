use crate::core::{
    errors::ExchangeError,
    types::{
        Market, MarketDataType, OrderRequest, OrderResponse, SubscriptionType, WebSocketConfig,
    },
};
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
pub trait ExchangeConnector {
    /// Get all available markets/trading pairs
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError>;
    /// Place a new order
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError>;

    /// Subscribe to market data via WebSocket
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError>;

    /// Get WebSocket endpoint URL for market data
    fn get_websocket_url(&self) -> String;
}
