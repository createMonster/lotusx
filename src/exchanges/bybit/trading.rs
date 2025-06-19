use super::auth;
use super::client::BybitConnector;
use super::converters::{convert_order_side, convert_order_type, convert_time_in_force};
use super::types::{self as bybit_types, BybitError, BybitResultExt};
use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse, OrderType};
use async_trait::async_trait;
use tracing::{instrument, error};

/// Helper to handle API response errors for orders
#[cold]
#[inline(never)]
fn handle_order_api_error(ret_code: i32, ret_msg: String, symbol: &str) -> BybitError {
    error!(symbol = %symbol, code = ret_code, message = %ret_msg, "Order API error");
    BybitError::api_error(ret_code, ret_msg)
}

/// Helper to handle order parsing errors
#[cold]
#[inline(never)]
fn handle_order_parse_error(err: serde_json::Error, response_text: &str, symbol: &str) -> BybitError {
    error!(symbol = %symbol, response = %response_text, "Failed to parse order response");
    BybitError::JsonError(err)
}

#[async_trait]
impl OrderPlacer for BybitConnector {
    #[instrument(skip(self), fields(exchange = "bybit", symbol = %order.symbol, side = ?order.side, order_type = ?order.order_type))]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let url = format!("{}/v5/order/create", self.base_url);
        let timestamp = auth::get_timestamp();

        // Build the request body for V5 API
        let mut request_body = bybit_types::BybitOrderRequest {
            category: "spot".to_string(),
            symbol: order.symbol.clone(),
            side: convert_order_side(&order.side),
            order_type: convert_order_type(&order.order_type),
            qty: order.quantity.clone(),
            price: None,
            time_in_force: None,
            stop_price: None,
        };

        // Add price for limit orders
        if matches!(order.order_type, OrderType::Limit) {
            request_body.price = order.price.clone();
            request_body.time_in_force = Some(
                order.time_in_force.as_ref()
                    .map_or_else(|| "GTC".to_string(), convert_time_in_force)
            );
        }

        // Add stop price for stop orders
        if let Some(stop_price) = &order.stop_price {
            request_body.stop_price = Some(stop_price.clone());
        }

        let body = serde_json::to_string(&request_body)
            .with_order_context(&order.symbol, &order.side.to_string())?;

        // V5 API signature
        let signature = auth::sign_v5_request(&body, self.config.secret_key(), self.config.api_key(), timestamp)
            .with_order_context(&order.symbol, &order.side.to_string())?;

        let response = self
            .client
            .post(&url)
            .header("X-BAPI-API-KEY", self.config.api_key())
            .header("X-BAPI-TIMESTAMP", timestamp.to_string())
            .header("X-BAPI-RECV-WINDOW", "5000")
            .header("X-BAPI-SIGN", &signature)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .with_order_context(&order.symbol, &order.side.to_string())?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .with_order_context(&order.symbol, &order.side.to_string())?;
            return Err(ExchangeError::Other(format!(
                "Order placement failed for {}: {}",
                order.symbol, error_text
            )));
        }

        let response_text = response.text().await
            .with_order_context(&order.symbol, &order.side.to_string())?;
            
        let api_response: bybit_types::BybitApiResponse<bybit_types::BybitOrderResponse> = 
            serde_json::from_str(&response_text)
                .map_err(|e| handle_order_parse_error(e, &response_text, &order.symbol))?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::Other(
                handle_order_api_error(api_response.ret_code, api_response.ret_msg, &order.symbol).to_string()
            ));
        }

        let bybit_response = api_response.result;
        let order_id = bybit_response.order_id.clone();
        Ok(OrderResponse {
            order_id,
            client_order_id: bybit_response.client_order_id,
            symbol: bybit_response.symbol,
            side: order.side,
            order_type: order.order_type,
            quantity: bybit_response.qty,
            price: Some(bybit_response.price),
            status: bybit_response.status,
            timestamp: bybit_response.timestamp,
        })
    }

    #[instrument(skip(self), fields(exchange = "bybit", symbol = %symbol, order_id = %order_id))]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let url = format!("{}/v5/order/cancel", self.base_url);
        let timestamp = auth::get_timestamp();

        let request_body = serde_json::json!({
            "category": "spot",
            "symbol": symbol,
            "orderId": order_id
        });

        let body = request_body.to_string();
        let signature = auth::sign_v5_request(&body, self.config.secret_key(), self.config.api_key(), timestamp)
            .with_symbol_context(&symbol)?;

        let response = self
            .client
            .post(&url)
            .header("X-BAPI-API-KEY", self.config.api_key())
            .header("X-BAPI-TIMESTAMP", timestamp.to_string())
            .header("X-BAPI-RECV-WINDOW", "5000")
            .header("X-BAPI-SIGN", &signature)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .with_symbol_context(&symbol)?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .with_symbol_context(&symbol)?;
            return Err(ExchangeError::Other(format!(
                "Order cancellation failed for {}: {}",
                symbol, error_text
            )));
        }

        let response_text = response.text().await
            .with_symbol_context(&symbol)?;
            
        let api_response: bybit_types::BybitApiResponse<serde_json::Value> = 
            serde_json::from_str(&response_text)
                .map_err(|e| handle_order_parse_error(e, &response_text, &symbol))?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::Other(
                handle_order_api_error(api_response.ret_code, api_response.ret_msg, &symbol).to_string()
            ));
        }

        Ok(())
    }
} 