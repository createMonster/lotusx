use super::auth::generate_nonce;
use super::client::HyperliquidClient;
use super::converters::{convert_from_hyperliquid_response, convert_to_hyperliquid_order};
use super::types::{CancelRequest, HyperliquidError};
use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse};
use async_trait::async_trait;
use serde_json::json;
use tracing::{error, instrument};

/// Helper to handle authentication errors
#[cold]
#[inline(never)]
fn handle_auth_error(operation: &str) -> HyperliquidError {
    error!(operation = %operation, "Authentication required for operation");
    HyperliquidError::auth_error(format!("Private key required for {}", operation))
}

/// Helper to handle invalid order ID parsing
#[cold]
#[inline(never)]
fn handle_invalid_order_id(order_id: &str) -> HyperliquidError {
    error!(order_id = %order_id, "Invalid order ID format");
    HyperliquidError::invalid_order("Invalid order ID format - must be a number".to_string())
}

#[async_trait]
impl OrderPlacer for HyperliquidClient {
    /// Place an order - this is a critical hot path for HFT
    #[instrument(skip(self), fields(exchange = "hyperliquid", symbol = %order.symbol, side = ?order.side, order_type = ?order.order_type))]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                handle_auth_error("placing orders").to_string(),
            ));
        }

        let hyperliquid_order = convert_to_hyperliquid_order(&order);

        let action = json!({
            "type": "order",
            "orders": [hyperliquid_order]
        });

        let signed_request =
            self.auth
                .sign_l1_action(action, self.vault_address.clone(), Some(generate_nonce()))?;

        let response: super::types::OrderResponse =
            self.post_exchange_request(&signed_request).await?;

        Ok(convert_from_hyperliquid_response(&response, &order))
    }

    /// Cancel an order - also critical for HFT
    #[instrument(skip(self), fields(exchange = "hyperliquid", symbol = %symbol, order_id = %order_id))]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                handle_auth_error("cancelling orders").to_string(),
            ));
        }

        let order_id_parsed = order_id.parse::<u64>().map_err(|_| {
            ExchangeError::InvalidParameters(handle_invalid_order_id(&order_id).to_string())
        })?;

        let cancel_request = CancelRequest {
            coin: symbol,
            oid: order_id_parsed,
        };

        let action = json!({
            "type": "cancel",
            "cancels": [cancel_request]
        });

        let signed_request =
            self.auth
                .sign_l1_action(action, self.vault_address.clone(), Some(generate_nonce()))?;

        let _response: super::types::OrderResponse =
            self.post_exchange_request(&signed_request).await?;

        Ok(())
    }
}
