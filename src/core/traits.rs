use crate::core::{errors::ExchangeError, types::*};
use async_trait::async_trait;

#[async_trait]
pub trait ExchangeConnector {
    /// Get all available markets/trading pairs
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError>;
    
    /// Place a new order
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError>;
} 