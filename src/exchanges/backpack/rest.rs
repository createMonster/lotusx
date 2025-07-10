use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::exchanges::backpack::types::{
    BackpackBalanceMap, BackpackDepthResponse, BackpackFill, BackpackFundingRate,
    BackpackKlineResponse, BackpackMarketResponse, BackpackOrder, BackpackOrderResponse,
    BackpackPositionResponse, BackpackTickerResponse, BackpackTradeResponse,
};
use serde_json::Value;

/// Thin typed wrapper around `RestClient` for Backpack API
pub struct BackpackRestClient<R: RestClient> {
    client: R,
}

impl<R: RestClient> BackpackRestClient<R> {
    pub fn new(client: R) -> Self {
        Self { client }
    }

    /// Get all markets
    pub async fn get_markets(&self) -> Result<Vec<BackpackMarketResponse>, ExchangeError> {
        self.client.get_json("/api/v1/markets", &[], false).await
    }

    /// Get ticker for a symbol
    pub async fn get_ticker(&self, symbol: &str) -> Result<BackpackTickerResponse, ExchangeError> {
        let params = [("symbol", symbol)];
        self.client.get_json("/api/v1/ticker", &params, false).await
    }

    /// Get order book depth
    pub async fn get_order_book(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<BackpackDepthResponse, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.client.get_json("/api/v1/depth", &params, false).await
    }

    /// Get recent trades
    pub async fn get_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackTradeResponse>, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.client.get_json("/api/v1/trades", &params, false).await
    }

    /// Get klines/candlestick data
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackKlineResponse>, ExchangeError> {
        let start_str = start_time.map(|t| t.to_string());
        let end_str = end_time.map(|t| t.to_string());
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol), ("interval", interval)];

        if let Some(ref start) = start_str {
            params.push(("startTime", start.as_str()));
        }
        if let Some(ref end) = end_str {
            params.push(("endTime", end.as_str()));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.client.get_json("/api/v1/klines", &params, false).await
    }

    /// Get funding rates
    pub async fn get_funding_rates(&self) -> Result<Vec<BackpackFundingRate>, ExchangeError> {
        self.client
            .get_json("/api/v1/funding/rates", &[], false)
            .await
    }

    /// Get funding rate history
    pub async fn get_funding_rate_history(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackFundingRate>, ExchangeError> {
        let start_str = start_time.map(|t| t.to_string());
        let end_str = end_time.map(|t| t.to_string());
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref start) = start_str {
            params.push(("startTime", start.as_str()));
        }
        if let Some(ref end) = end_str {
            params.push(("endTime", end.as_str()));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.client
            .get_json("/api/v1/funding/rates/history", &params, false)
            .await
    }

    /// Get account balances (requires authentication)
    pub async fn get_balances(&self) -> Result<BackpackBalanceMap, ExchangeError> {
        self.client.get_json("/api/v1/balances", &[], true).await
    }

    /// Get account positions (requires authentication)
    pub async fn get_positions(&self) -> Result<Vec<BackpackPositionResponse>, ExchangeError> {
        self.client.get_json("/api/v1/positions", &[], true).await
    }

    /// Get order history (requires authentication)
    pub async fn get_order_history(
        &self,
        symbol: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackOrder>, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![];

        if let Some(symbol) = symbol {
            params.push(("symbol", symbol));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.client.get_json("/api/v1/orders", &params, true).await
    }

    /// Place an order (requires authentication)
    pub async fn place_order(&self, order: &Value) -> Result<BackpackOrderResponse, ExchangeError> {
        self.client.post_json("/api/v1/order", order, true).await
    }

    /// Cancel an order (requires authentication)
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<i64>,
        client_order_id: Option<&str>,
    ) -> Result<BackpackOrderResponse, ExchangeError> {
        let order_id_str = order_id.map(|id| id.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref order_id) = order_id_str {
            params.push(("orderId", order_id.as_str()));
        }
        if let Some(client_order_id) = client_order_id {
            params.push(("clientOrderId", client_order_id));
        }

        self.client
            .delete_json("/api/v1/order", &params, true)
            .await
    }

    /// Get fills (requires authentication)
    pub async fn get_fills(
        &self,
        symbol: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<BackpackFill>, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![];

        if let Some(symbol) = symbol {
            params.push(("symbol", symbol));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.client.get_json("/api/v1/fills", &params, true).await
    }
}
