use super::signer::HyperliquidSigner;
use super::types::*;
use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use serde_json::Value;
use tracing::instrument;

/// Thin typed wrapper around `RestClient` for Hyperliquid API
#[derive(Clone)]
pub struct HyperliquidRest<R: RestClient> {
    client: R,
    signer: Option<HyperliquidSigner>,
    vault_address: Option<String>,
    is_testnet: bool,
}

impl<R: RestClient> HyperliquidRest<R> {
    pub fn new(client: R, signer: Option<HyperliquidSigner>, is_testnet: bool) -> Self {
        Self {
            client,
            signer,
            vault_address: None,
            is_testnet,
        }
    }

    pub fn with_vault_address(mut self, vault_address: String) -> Self {
        self.vault_address = Some(vault_address);
        self
    }

    pub fn wallet_address(&self) -> Option<&str> {
        self.signer.as_ref().and_then(|s| s.wallet_address())
    }

    pub fn can_sign(&self) -> bool {
        self.signer.as_ref().map_or(false, |s| s.can_sign())
    }

    /// Get all available markets/trading pairs
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    pub async fn get_markets(&self) -> Result<Vec<AssetInfo>, ExchangeError> {
        let request = InfoRequest::Meta;
        let request_value = serde_json::to_value(&request).map_err(ExchangeError::JsonError)?;

        let response: Value = self
            .client
            .post_json("/info", &request_value, false)
            .await?;

        // Extract universe from the response
        if let Some(universe) = response.get("universe") {
            let universe: Universe =
                serde_json::from_value(universe.clone()).map_err(ExchangeError::JsonError)?;
            Ok(universe.universe)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all token mids (ticker prices)
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    pub async fn get_all_mids(&self) -> Result<serde_json::Map<String, Value>, ExchangeError> {
        let request = InfoRequest::AllMids;
        let request_value = serde_json::to_value(&request).map_err(ExchangeError::JsonError)?;

        let response: Value = self
            .client
            .post_json("/info", &request_value, false)
            .await?;

        if let Some(mids) = response.as_object() {
            Ok(mids.clone())
        } else {
            Ok(serde_json::Map::new())
        }
    }

    /// Get level 2 order book for a specific coin
    #[instrument(skip(self), fields(exchange = "hyperliquid", coin = %coin))]
    pub async fn get_l2_book(&self, coin: &str) -> Result<L2Book, ExchangeError> {
        let request = InfoRequest::L2Book {
            coin: coin.to_string(),
        };
        let request_value = serde_json::to_value(&request).map_err(ExchangeError::JsonError)?;

        self.client.post_json("/info", &request_value, false).await
    }

    /// Get recent trades for a specific coin
    #[instrument(skip(self), fields(exchange = "hyperliquid", coin = %coin))]
    pub async fn get_recent_trades(&self, coin: &str) -> Result<Vec<Value>, ExchangeError> {
        // Note: Hyperliquid doesn't have a direct recentTrades endpoint in InfoRequest
        // This would need to be implemented differently or removed
        Err(ExchangeError::Other(
            "Recent trades not supported via InfoRequest".to_string(),
        ))
    }

    /// Get candlestick data for a specific coin
    #[instrument(skip(self), fields(exchange = "hyperliquid", coin = %coin, interval = %interval))]
    pub async fn get_candlestick_snapshot(
        &self,
        coin: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Candle>, ExchangeError> {
        let request = InfoRequest::CandleSnapshot {
            coin: coin.to_string(),
            interval: interval.to_string(),
            start_time: start_time.unwrap_or(0) as u64,
            end_time: end_time.unwrap_or(0) as u64,
        };
        let request_value = serde_json::to_value(&request).map_err(ExchangeError::JsonError)?;

        self.client.post_json("/info", &request_value, false).await
    }

    /// Get user state (requires authentication)
    #[instrument(skip(self), fields(exchange = "hyperliquid", user = %user))]
    pub async fn get_user_state(&self, user: &str) -> Result<UserState, ExchangeError> {
        let request = InfoRequest::UserState {
            user: user.to_string(),
        };
        let request_value = serde_json::to_value(&request).map_err(ExchangeError::JsonError)?;

        self.client.post_json("/info", &request_value, false).await
    }

    /// Get user fills (requires authentication)
    #[instrument(skip(self), fields(exchange = "hyperliquid", user = %user))]
    pub async fn get_user_fills(&self, user: &str) -> Result<Vec<UserFill>, ExchangeError> {
        let request = InfoRequest::UserFills {
            user: user.to_string(),
        };
        let request_value = serde_json::to_value(&request).map_err(ExchangeError::JsonError)?;

        self.client.post_json("/info", &request_value, false).await
    }

    /// Get open orders (requires authentication)
    #[instrument(skip(self), fields(exchange = "hyperliquid", user = %user))]
    pub async fn get_open_orders(&self, user: &str) -> Result<Vec<OpenOrder>, ExchangeError> {
        let request = InfoRequest::OpenOrders {
            user: user.to_string(),
        };
        let request_value = serde_json::to_value(&request).map_err(ExchangeError::JsonError)?;

        self.client.post_json("/info", &request_value, false).await
    }

    /// Place an order (requires authentication)
    #[instrument(skip(self, order), fields(exchange = "hyperliquid"))]
    pub async fn place_order(&self, order: &OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let signer = self.signer.as_ref().ok_or_else(|| {
            ExchangeError::AuthError("No signer available for placing orders".to_string())
        })?;

        let action = serde_json::json!({
            "type": "order",
            "orders": [order]
        });

        let exchange_request = signer.sign_l1_action(action, self.vault_address.clone(), None)?;
        let request_value =
            serde_json::to_value(&exchange_request).map_err(ExchangeError::JsonError)?;

        self.client
            .post_json("/exchange", &request_value, false)
            .await
    }

    /// Cancel an order (requires authentication)
    #[instrument(skip(self), fields(exchange = "hyperliquid", coin = %coin, oid = %oid))]
    pub async fn cancel_order(&self, coin: &str, oid: u64) -> Result<OrderResponse, ExchangeError> {
        let signer = self.signer.as_ref().ok_or_else(|| {
            ExchangeError::AuthError("No signer available for canceling orders".to_string())
        })?;

        let action = serde_json::json!({
            "type": "cancel",
            "cancels": [{
                "coin": coin,
                "oid": oid
            }]
        });

        let exchange_request = signer.sign_l1_action(action, self.vault_address.clone(), None)?;
        let request_value =
            serde_json::to_value(&exchange_request).map_err(ExchangeError::JsonError)?;

        self.client
            .post_json("/exchange", &request_value, false)
            .await
    }

    /// Cancel all orders (requires authentication)
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    pub async fn cancel_all_orders(&self) -> Result<OrderResponse, ExchangeError> {
        let signer = self.signer.as_ref().ok_or_else(|| {
            ExchangeError::AuthError("No signer available for canceling orders".to_string())
        })?;

        let action = serde_json::json!({
            "type": "cancelByCloid",
            "cancels": []
        });

        let exchange_request = signer.sign_l1_action(action, self.vault_address.clone(), None)?;
        let request_value =
            serde_json::to_value(&exchange_request).map_err(ExchangeError::JsonError)?;

        self.client
            .post_json("/exchange", &request_value, false)
            .await
    }

    /// Modify an order (requires authentication)
    #[instrument(skip(self, modify_request), fields(exchange = "hyperliquid"))]
    pub async fn modify_order(
        &self,
        modify_request: &ModifyRequest,
    ) -> Result<OrderResponse, ExchangeError> {
        let signer = self.signer.as_ref().ok_or_else(|| {
            ExchangeError::AuthError("No signer available for modifying orders".to_string())
        })?;

        let action = serde_json::json!({
            "type": "modify",
            "modifies": [modify_request]
        });

        let exchange_request = signer.sign_l1_action(action, self.vault_address.clone(), None)?;
        let request_value =
            serde_json::to_value(&exchange_request).map_err(ExchangeError::JsonError)?;

        self.client
            .post_json("/exchange", &request_value, false)
            .await
    }

    /// Get WebSocket URL for this client
    pub fn get_websocket_url(&self) -> String {
        if self.is_testnet {
            "wss://api.hyperliquid-testnet.xyz/ws".to_string()
        } else {
            "wss://api.hyperliquid.xyz/ws".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::kernel::rest::ReqwestRest;

    #[test]
    fn test_rest_client_creation() {
        let rest_client = ReqwestRest::new(
            "https://api.hyperliquid.xyz".to_string(),
            "hyperliquid".to_string(),
            None,
        )
        .unwrap();
        let hyperliquid_rest = HyperliquidRest::new(rest_client, None, false);

        assert!(!hyperliquid_rest.can_sign());
        assert!(hyperliquid_rest.wallet_address().is_none());
    }
}
