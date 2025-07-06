use super::auth::ParadexAuth;
use super::client::ParadexConnector;
use super::types::ParadexOrder;
use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use tracing::{error, instrument};

#[async_trait]
impl OrderPlacer for ParadexConnector {
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
        if !self.can_trade() {
            return Err(ExchangeError::AuthError(
                "Missing API credentials for trading".to_string(),
            ));
        }

        let auth = ParadexAuth::with_private_key(self.config.secret_key.expose_secret().as_str())
            .map_err(|e| {
            error!(error = %e, "Failed to create auth");
            ExchangeError::AuthError(format!("Authentication setup failed: {}", e))
        })?;

        let token = auth.sign_jwt().map_err(|e| {
            error!(error = %e, "Failed to sign JWT");
            ExchangeError::AuthError(format!("JWT signing failed: {}", e))
        })?;

        let url = format!("{}/v1/orders", self.base_url);

        // Convert order to Paradex format
        let paradex_order = convert_order_request(&order);

        let response = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(&paradex_order)
            .send()
            .await
            .map_err(|e| {
                error!(
                    symbol = %order.symbol,
                    error = %e,
                    "Failed to send order request"
                );
                ExchangeError::NetworkError(format!("Order request failed: {}", e))
            })?;

        self.handle_order_response(response, &order).await
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
        if !self.can_trade() {
            return Err(ExchangeError::AuthError(
                "Missing API credentials for trading".to_string(),
            ));
        }

        let auth = ParadexAuth::with_private_key(self.config.secret_key.expose_secret().as_str())
            .map_err(|e| {
            error!(error = %e, "Failed to create auth");
            ExchangeError::AuthError(format!("Authentication setup failed: {}", e))
        })?;

        let token = auth.sign_jwt().map_err(|e| {
            error!(error = %e, "Failed to sign JWT");
            ExchangeError::AuthError(format!("JWT signing failed: {}", e))
        })?;

        let url = format!("{}/v1/orders/{}", self.base_url, order_id);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| {
                error!(
                    symbol = %symbol,
                    order_id = %order_id,
                    error = %e,
                    "Failed to send cancel request"
                );
                ExchangeError::NetworkError(format!("Cancel request failed: {}", e))
            })?;

        self.handle_cancel_response(response, &symbol, &order_id)
            .await
    }
}

impl ParadexConnector {
    #[cold]
    #[inline(never)]
    async fn handle_order_response(
        &self,
        response: reqwest::Response,
        order: &OrderRequest,
    ) -> Result<OrderResponse, ExchangeError> {
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!(
                symbol = %order.symbol,
                status = %status,
                error_text = %error_text,
                "Order placement failed"
            );

            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Order placement failed: {}", error_text),
            });
        }

        let paradex_response: ParadexOrder = response.json().await.map_err(|e| {
            error!(
                symbol = %order.symbol,
                error = %e,
                "Failed to parse order response"
            );
            ExchangeError::Other(format!("Failed to parse order response: {}", e))
        })?;

        Ok(OrderResponse {
            order_id: paradex_response.id,
            client_order_id: paradex_response.client_id,
            symbol: crate::core::types::conversion::string_to_symbol(&paradex_response.market),
            side: order.side.clone(),
            order_type: order.order_type.clone(),
            quantity: crate::core::types::conversion::string_to_quantity(&paradex_response.size),
            price: Some(crate::core::types::conversion::string_to_price(&paradex_response.price)),
            status: paradex_response.status,
            timestamp: chrono::DateTime::parse_from_rfc3339(&paradex_response.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .timestamp_millis(),
        })
    }

    #[cold]
    #[inline(never)]
    async fn handle_cancel_response(
        &self,
        response: reqwest::Response,
        symbol: &str,
        order_id: &str,
    ) -> Result<(), ExchangeError> {
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!(
                symbol = %symbol,
                order_id = %order_id,
                status = %status,
                error_text = %error_text,
                "Order cancellation failed"
            );

            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Order cancellation failed: {}", error_text),
            });
        }

        Ok(())
    }
}

/// Convert core `OrderRequest` to Paradex order format
fn convert_order_request(order: &OrderRequest) -> serde_json::Value {
    let side = match order.side {
        crate::core::types::OrderSide::Buy => "BUY",
        crate::core::types::OrderSide::Sell => "SELL",
    };

    let order_type = match order.order_type {
        crate::core::types::OrderType::Market => "MARKET",
        crate::core::types::OrderType::Limit => "LIMIT",
        crate::core::types::OrderType::StopLoss => "STOP_MARKET",
        crate::core::types::OrderType::StopLossLimit => "STOP_LIMIT",
        crate::core::types::OrderType::TakeProfit => "TAKE_PROFIT_MARKET",
        crate::core::types::OrderType::TakeProfitLimit => "TAKE_PROFIT_LIMIT",
    };

    let mut paradex_order = serde_json::json!({
        "market": order.symbol.to_string(),
        "side": side,
        "order_type": order_type,
        "size": order.quantity.to_string(),
    });

    // Add price for limit orders
    if let Some(price) = &order.price {
        if matches!(
            order.order_type,
            crate::core::types::OrderType::Limit
                | crate::core::types::OrderType::StopLossLimit
                | crate::core::types::OrderType::TakeProfitLimit
        ) {
            paradex_order["price"] = serde_json::Value::String(price.to_string());
        }
    }

    // Add stop price for stop orders
    if let Some(stop_price) = &order.stop_price {
        if matches!(
            order.order_type,
            crate::core::types::OrderType::StopLoss
                | crate::core::types::OrderType::StopLossLimit
                | crate::core::types::OrderType::TakeProfit
                | crate::core::types::OrderType::TakeProfitLimit
        ) {
            paradex_order["stop_price"] = serde_json::Value::String(stop_price.to_string());
        }
    }

    // Add time in force for limit orders
    if let Some(tif) = &order.time_in_force {
        let time_in_force = match tif {
            crate::core::types::TimeInForce::GTC => "GTC",
            crate::core::types::TimeInForce::IOC => "IOC",
            crate::core::types::TimeInForce::FOK => "FOK",
        };
        paradex_order["time_in_force"] = serde_json::Value::String(time_in_force.to_string());
    } else if matches!(order.order_type, crate::core::types::OrderType::Limit) {
        // Default to GTC for limit orders
        paradex_order["time_in_force"] = serde_json::Value::String("GTC".to_string());
    }

    paradex_order
}
