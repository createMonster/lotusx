use super::auth::generate_nonce;
use super::client::HyperliquidClient;
use super::converters::{convert_to_hyperliquid_order, convert_from_hyperliquid_response};
use super::types::{CancelRequest};
use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse};
use async_trait::async_trait;
use serde_json::json;

#[async_trait]
impl OrderPlacer for HyperliquidClient {
    /// Place an order - this is a critical hot path for HFT
    #[inline]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Private key required for placing orders".to_string(),
            ));
        }

        let hyperliquid_order = convert_to_hyperliquid_order(&order);

        let action = json!({
            "type": "order",
            "orders": [hyperliquid_order]
        });

        let signed_request = self.auth.sign_l1_action(
            action, 
            self.vault_address.clone(), 
            Some(generate_nonce())
        )?;

        let response: super::types::OrderResponse = self
            .post_exchange_request(&signed_request)
            .await?;
            
        Ok(convert_from_hyperliquid_response(&response, &order))
    }

    /// Cancel an order - also critical for HFT
    #[inline]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Private key required for cancelling orders".to_string(),
            ));
        }

        let order_id_parsed = order_id
            .parse::<u64>()
            .map_err(|_| ExchangeError::InvalidParameters("Invalid order ID format".to_string()))?;

        let cancel_request = CancelRequest {
            coin: symbol,
            oid: order_id_parsed,
        };

        let action = json!({
            "type": "cancel",
            "cancels": [cancel_request]
        });

        let signed_request = self.auth.sign_l1_action(
            action,
            self.vault_address.clone(),
            Some(generate_nonce())
        )?;

        let _response: super::types::OrderResponse = self
            .post_exchange_request(&signed_request)
            .await?;
            
        Ok(())
    }
} 