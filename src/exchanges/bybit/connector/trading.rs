use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse, Symbol};
use crate::exchanges::bybit::conversions::{
    convert_order_side, convert_order_type, convert_time_in_force,
};
use crate::exchanges::bybit::rest::BybitRestClient;
use crate::exchanges::bybit::types::{BybitOrderRequest, BybitOrderResponse};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Trading implementation for Bybit
pub struct Trading<R: RestClient> {
    rest: BybitRestClient<R>,
}

impl<R: RestClient> Trading<R> {
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: BybitRestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient + Send + Sync> OrderPlacer for Trading<R> {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        // Convert unified order to Bybit format
        let bybit_order = BybitOrderRequest {
            category: "spot".to_string(),
            symbol: order.symbol.to_string(),
            side: convert_order_side(&order.side),
            order_type: convert_order_type(&order.order_type),
            qty: order.quantity.to_string(),
            price: order.price.map(|p| p.to_string()),
            time_in_force: order.time_in_force.as_ref().map(convert_time_in_force),
            stop_price: order.stop_price.map(|p| p.to_string()),
        };

        // Validate required fields
        if bybit_order.order_type == "Limit" && bybit_order.price.is_none() {
            return Err(ExchangeError::InvalidParameters(
                "Price is required for limit orders".to_string(),
            ));
        }

        // Validate quantity
        let _quantity = Decimal::from_str(&bybit_order.qty)
            .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid quantity: {}", e)))?;

        // Validate price if provided
        if let Some(ref price_str) = bybit_order.price {
            let _price = Decimal::from_str(price_str)
                .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid price: {}", e)))?;
        }

        let bybit_response: BybitOrderResponse = self.rest.place_order(&bybit_order).await?;

        // Convert Bybit response to unified response
        Ok(OrderResponse {
            order_id: bybit_response.order_id.clone(),
            client_order_id: bybit_response.client_order_id.clone(),
            symbol: Symbol::from_string(&bybit_response.symbol)
                .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid symbol: {}", e)))?,
            side: order.side,
            order_type: order.order_type,
            quantity: order.quantity,
            price: order.price,
            status: bybit_response.status,
            timestamp: bybit_response.timestamp,
        })
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        self.rest.cancel_order(&symbol, &order_id).await?;
        Ok(())
    }
}
