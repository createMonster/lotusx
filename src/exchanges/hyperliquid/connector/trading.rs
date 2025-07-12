use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse};
use crate::exchanges::hyperliquid::conversions;
use crate::exchanges::hyperliquid::rest::HyperliquidRest;
use async_trait::async_trait;
use tracing::instrument;

/// Trading implementation for Hyperliquid
pub struct Trading<R: RestClient> {
    rest: HyperliquidRest<R>,
}

impl<R: RestClient> Trading<R> {
    pub fn new(rest: HyperliquidRest<R>) -> Self {
        Self { rest }
    }

    pub fn can_sign(&self) -> bool {
        self.rest.can_sign()
    }

    pub fn wallet_address(&self) -> Option<&str> {
        self.rest.wallet_address()
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync> OrderPlacer for Trading<R> {
    /// Place a new order
    #[instrument(skip(self, order), fields(exchange = "hyperliquid"))]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Trading requires authentication".to_string(),
            ));
        }

        // Convert the generic OrderRequest to Hyperliquid's OrderRequest
        let hyperliquid_order = conversions::convert_order_request_to_hyperliquid(&order)?;

        // Place the order
        let response = self.rest.place_order(&hyperliquid_order).await?;

        // Convert the response back to generic OrderResponse
        conversions::convert_hyperliquid_order_response_to_generic(&response, &order)
    }

    /// Cancel an existing order
    #[instrument(skip(self), fields(exchange = "hyperliquid", symbol = %symbol, order_id = %order_id))]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Trading requires authentication".to_string(),
            ));
        }

        // Parse order ID to u64 as required by Hyperliquid
        let oid = order_id.parse::<u64>().map_err(|e| {
            ExchangeError::InvalidParameters(format!("Invalid order ID format: {}", e))
        })?;

        let _response = self.rest.cancel_order(&symbol, oid).await?;
        Ok(())
    }
}

impl<R: RestClient> Trading<R> {
    /// Cancel all open orders (Hyperliquid-specific)
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    pub async fn cancel_all_orders(&self) -> Result<(), ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Trading requires authentication".to_string(),
            ));
        }

        let _response = self.rest.cancel_all_orders().await?;
        Ok(())
    }

    /// Modify an existing order (Hyperliquid-specific)
    #[instrument(skip(self, modify_request), fields(exchange = "hyperliquid"))]
    pub async fn modify_order(
        &self,
        modify_request: &crate::exchanges::hyperliquid::types::ModifyRequest,
    ) -> Result<crate::exchanges::hyperliquid::types::OrderResponse, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Trading requires authentication".to_string(),
            ));
        }

        self.rest.modify_order(modify_request).await
    }

    /// Get open orders for the authenticated user (Hyperliquid-specific)
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    pub async fn get_open_orders(
        &self,
    ) -> Result<Vec<crate::exchanges::hyperliquid::types::OpenOrder>, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Trading requires authentication".to_string(),
            ));
        }

        let wallet_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("No wallet address available".to_string()))?;

        self.rest.get_open_orders(wallet_address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::kernel::rest::ReqwestRest;
    use crate::exchanges::hyperliquid::rest::HyperliquidRest;

    #[test]
    fn test_trading_creation() {
        let rest_client = ReqwestRest::new(
            "https://api.hyperliquid.xyz".to_string(),
            "hyperliquid".to_string(),
            None,
        )
        .unwrap();
        let hyperliquid_rest = HyperliquidRest::new(rest_client, None, false);
        let trading = Trading::new(hyperliquid_rest);

        assert!(!trading.can_sign());
        assert!(trading.wallet_address().is_none());
    }
}
