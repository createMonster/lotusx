use super::client::BybitPerpConnector;
use super::converters::{convert_order_side, convert_order_type, convert_time_in_force};
use super::types as bybit_perp_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse, OrderType};
use crate::exchanges::bybit::auth; // Reuse auth from spot Bybit
use async_trait::async_trait;

#[async_trait]
impl OrderPlacer for BybitPerpConnector {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let url = format!("{}/v5/order/create", self.base_url);
        let timestamp = auth::get_timestamp();

        // Build the request body for V5 API
        let mut request_body = bybit_perp_types::BybitPerpOrderRequest {
            category: "linear".to_string(), // Use linear for perpetual futures
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
                order
                    .time_in_force
                    .as_ref()
                    .map_or_else(|| "GTC".to_string(), convert_time_in_force),
            );
        }

        // Add stop price for stop orders
        if let Some(stop_price) = &order.stop_price {
            request_body.stop_price = Some(stop_price.clone());
        }

        let body = serde_json::to_string(&request_body).map_err(|e| {
            ExchangeError::NetworkError(format!("Failed to serialize request: {}", e))
        })?;

        // V5 API signature
        let signature = auth::sign_v5_request(
            &body,
            self.config.secret_key(),
            self.config.api_key(),
            timestamp,
        )?;

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
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Order placement failed: {}",
                error_text
            )));
        }

        let response_text = response.text().await?;
        let api_response: bybit_perp_types::BybitPerpApiResponse<
            bybit_perp_types::BybitPerpOrderResponse,
        > = serde_json::from_str(&response_text).map_err(|e| {
            ExchangeError::NetworkError(format!(
                "Failed to parse Bybit response: {}. Response was: {}",
                e, response_text
            ))
        })?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::NetworkError(format!(
                "Bybit API error ({}): {}",
                api_response.ret_code, api_response.ret_msg
            )));
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

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let url = format!("{}/v5/order/cancel", self.base_url);
        let timestamp = auth::get_timestamp();

        let request_body = serde_json::json!({
            "category": "linear",
            "symbol": symbol,
            "orderId": order_id
        });

        let body = request_body.to_string();
        let signature = auth::sign_v5_request(
            &body,
            self.config.secret_key(),
            self.config.api_key(),
            timestamp,
        )?;

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
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Order cancellation failed: {}",
                error_text
            )));
        }

        let response_text = response.text().await?;
        let api_response: bybit_perp_types::BybitPerpApiResponse<serde_json::Value> =
            serde_json::from_str(&response_text).map_err(|e| {
                ExchangeError::NetworkError(format!(
                    "Failed to parse Bybit response: {}. Response was: {}",
                    e, response_text
                ))
            })?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::NetworkError(format!(
                "Bybit API error ({}): {}",
                api_response.ret_code, api_response.ret_msg
            )));
        }

        Ok(())
    }
}
