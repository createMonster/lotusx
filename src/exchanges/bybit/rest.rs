use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::types::KlineInterval;
use crate::exchanges::bybit::conversions::kline_interval_to_bybit_string;
use crate::exchanges::bybit::types::{
    BybitAccountInfo, BybitKlineResult, BybitMarketsResult, BybitOrderRequest, BybitOrderResponse,
    BybitTicker,
};
use async_trait::async_trait;
use reqwest::Method;
use serde_json::Value;

/// Thin typed wrapper around `RestClient` for Bybit API
pub struct BybitRestClient<R: RestClient> {
    client: R,
}

impl<R: RestClient> BybitRestClient<R> {
    pub fn new(client: R) -> Self {
        Self { client }
    }

    /// Get all tradable markets
    pub async fn get_markets(&self) -> Result<BybitMarketsResult, ExchangeError> {
        let params = [("category", "spot")];
        self.client
            .get_json("/v5/market/instruments-info", &params, false)
            .await
    }

    /// Get ticker for a symbol
    pub async fn get_ticker(&self, symbol: &str) -> Result<BybitTicker, ExchangeError> {
        let params = [("category", "spot"), ("symbol", symbol)];
        self.client
            .get_json("/v5/market/tickers", &params, false)
            .await
    }

    /// Get klines for a symbol
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<BybitKlineResult, ExchangeError> {
        let interval_str = kline_interval_to_bybit_string(interval);
        let limit_str = limit.unwrap_or(200).to_string();

        let mut params = vec![
            ("category", "spot"),
            ("symbol", symbol),
            ("interval", interval_str),
            ("limit", &limit_str),
        ];

        let start_time_str;
        let end_time_str;

        if let Some(start) = start_time {
            start_time_str = start.to_string();
            params.push(("start", &start_time_str));
        }

        if let Some(end) = end_time {
            end_time_str = end.to_string();
            params.push(("end", &end_time_str));
        }

        self.client
            .get_json("/v5/market/kline", &params, false)
            .await
    }

    /// Get account balances (requires authentication)
    pub async fn get_balances(&self) -> Result<BybitAccountInfo, ExchangeError> {
        let params = [("accountType", "UNIFIED")];
        self.client
            .get_json("/v5/account/wallet-balance", &params, true)
            .await
    }

    /// Place a new order (requires authentication)
    pub async fn place_order(
        &self,
        order: &BybitOrderRequest,
    ) -> Result<BybitOrderResponse, ExchangeError> {
        let body = serde_json::to_value(order).map_err(|e| {
            ExchangeError::SerializationError(format!("Failed to serialize order: {}", e))
        })?;

        self.client.post_json("/v5/order/create", &body, true).await
    }

    /// Cancel an existing order (requires authentication)
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> Result<serde_json::Value, ExchangeError> {
        let body = serde_json::json!({
            "category": "spot",
            "symbol": symbol,
            "orderId": order_id
        });

        self.client.post_json("/v5/order/cancel", &body, true).await
    }

    /// Get order history (requires authentication)
    pub async fn get_orders(&self, symbol: &str) -> Result<serde_json::Value, ExchangeError> {
        let params = [("category", "spot"), ("symbol", symbol)];
        self.client
            .get_json("/v5/order/history", &params, true)
            .await
    }

    /// Get trading fees (requires authentication)
    pub async fn get_trading_fees(&self) -> Result<serde_json::Value, ExchangeError> {
        let params = [("category", "spot")];
        self.client
            .get_json("/v5/account/fee-rate", &params, true)
            .await
    }
}

// Implement RestClient trait to delegate to inner client
#[async_trait]
impl<R: RestClient> RestClient for BybitRestClient<R> {
    async fn get(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<Value, ExchangeError> {
        self.client.get(endpoint, query_params, authenticated).await
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
        signed: bool,
    ) -> Result<T, ExchangeError> {
        self.client.get_json(endpoint, params, signed).await
    }

    async fn post(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<Value, ExchangeError> {
        self.client.post(endpoint, body, authenticated).await
    }

    async fn post_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Value,
        signed: bool,
    ) -> Result<T, ExchangeError> {
        self.client.post_json(endpoint, body, signed).await
    }

    async fn put(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<Value, ExchangeError> {
        self.client.put(endpoint, body, authenticated).await
    }

    async fn put_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &Value,
        authenticated: bool,
    ) -> Result<T, ExchangeError> {
        self.client.put_json(endpoint, body, authenticated).await
    }

    async fn delete(
        &self,
        endpoint: &str,
        query_params: &[(&str, &str)],
        authenticated: bool,
    ) -> Result<Value, ExchangeError> {
        self.client
            .delete(endpoint, query_params, authenticated)
            .await
    }

    async fn delete_json<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
        signed: bool,
    ) -> Result<T, ExchangeError> {
        self.client.delete_json(endpoint, params, signed).await
    }

    async fn signed_request(
        &self,
        method: Method,
        endpoint: &str,
        query_params: &[(&str, &str)],
        body: &[u8],
    ) -> Result<Value, ExchangeError> {
        self.client
            .signed_request(method, endpoint, query_params, body)
            .await
    }

    async fn signed_request_json<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        query_params: &[(&str, &str)],
        body: &[u8],
    ) -> Result<T, ExchangeError> {
        self.client
            .signed_request_json(method, endpoint, query_params, body)
            .await
    }
}
