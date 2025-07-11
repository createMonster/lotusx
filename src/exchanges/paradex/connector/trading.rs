use crate::core::errors::ExchangeError;
use crate::core::kernel::rest::RestClient;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse, OrderSide, OrderType};
use crate::exchanges::paradex::rest::ParadexRestClient;
use async_trait::async_trait;
use serde_json::{json, Value};
use tracing::instrument;

/// Trading implementation for Paradex
pub struct Trading<R: RestClient> {
    rest: ParadexRestClient<R>,
}

impl<R: RestClient> Trading<R> {
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: ParadexRestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync> OrderPlacer for Trading<R> {
    #[instrument(
        skip(self),
        fields(
            exchange = "paradex",
            symbol = %order.symbol,
            side = ?order.side,
            order_type = ?order.order_type,
            quantity = %order.quantity
        )
    )]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        // Convert order to Paradex format
        let paradex_order = convert_order_request(&order)?;

        // Place the order using the REST client
        let response = self.rest.place_order(&paradex_order).await?;

        // Convert the response back to OrderResponse
        Ok(OrderResponse {
            order_id: response.id,
            client_order_id: response.client_id,
            symbol: order.symbol,
            side: order.side,
            order_type: order.order_type,
            quantity: order.quantity,
            price: order.price,
            status: response.status,
            timestamp: chrono::DateTime::parse_from_rfc3339(&response.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .timestamp_millis(),
        })
    }

    #[instrument(
        skip(self),
        fields(
            exchange = "paradex",
            symbol = %symbol,
            order_id = %order_id
        )
    )]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        // Cancel the order using the REST client
        let _response = self.rest.cancel_order(&order_id).await?;

        // Log success
        tracing::info!(
            symbol = %symbol,
            order_id = %order_id,
            "Order cancelled successfully"
        );

        Ok(())
    }
}

/// Convert OrderRequest to Paradex JSON format
fn convert_order_request(order: &OrderRequest) -> Result<Value, ExchangeError> {
    let side = match order.side {
        OrderSide::Buy => "BUY",
        OrderSide::Sell => "SELL",
    };

    let order_type = match order.order_type {
        OrderType::Market => "MARKET",
        OrderType::Limit => "LIMIT",
        OrderType::StopLoss => "STOP_MARKET",
        OrderType::StopLossLimit => "STOP_LIMIT",
        OrderType::TakeProfit => "TAKE_PROFIT_MARKET",
        OrderType::TakeProfitLimit => "TAKE_PROFIT_LIMIT",
    };

    let mut paradex_order = json!({
        "market": order.symbol.to_string(),
        "side": side,
        "type": order_type,
        "size": order.quantity.to_string(),
    });

    // Add price for limit orders
    if let Some(price) = order.price {
        paradex_order["price"] = json!(price.to_string());
    }

    // Add stop price for stop orders
    if let Some(stop_price) = order.stop_price {
        paradex_order["stop_price"] = json!(stop_price.to_string());
    }

    // Add time in force if provided
    if let Some(time_in_force) = &order.time_in_force {
        paradex_order["time_in_force"] = json!(time_in_force.to_string());
    }

    Ok(paradex_order)
}
