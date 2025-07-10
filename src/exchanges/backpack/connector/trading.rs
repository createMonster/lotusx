use crate::core::{
    errors::ExchangeError,
    kernel::RestClient,
    traits::OrderPlacer,
    types::{OrderRequest, OrderResponse},
};
use crate::exchanges::backpack::rest::BackpackRestClient;
use async_trait::async_trait;
use serde_json::json;
use tracing::instrument;

/// Trading implementation for Backpack
pub struct Trading<R: RestClient> {
    rest: BackpackRestClient<R>,
}

impl<R: RestClient> Trading<R> {
    /// Create a new trading engine
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: BackpackRestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient> OrderPlacer for Trading<R> {
    #[instrument(skip(self), fields(exchange = "backpack"))]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        // Convert OrderRequest to Backpack API format
        let order_json = json!({
            "symbol": order.symbol.as_str(),
            "side": order.side,
            "type": order.order_type,
            "quantity": order.quantity.to_string(),
            "price": order.price.map(|p| p.to_string()),
            "timeInForce": order.time_in_force,
        });

        let response = self.rest.place_order(&order_json).await?;

        // Convert Backpack response to core OrderResponse
        Ok(OrderResponse {
            order_id: response.order_id.to_string(),
            client_order_id: response.client_order_id.unwrap_or_default(),
            symbol: crate::core::types::conversion::string_to_symbol(&response.symbol),
            side: order.side,
            order_type: order.order_type,
            quantity: order.quantity,
            price: order.price,
            status: response.status,
            timestamp: response.timestamp,
        })
    }

    #[instrument(skip(self), fields(exchange = "backpack", symbol = %symbol, order_id = %order_id))]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let order_id_i64: i64 = order_id
            .parse()
            .map_err(|_| ExchangeError::Other(format!("Invalid order ID format: {}", order_id)))?;
        self.rest
            .cancel_order(&symbol, Some(order_id_i64), None)
            .await?;
        Ok(())
    }
}
