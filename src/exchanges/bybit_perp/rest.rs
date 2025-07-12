use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::exchanges::bybit_perp::types::{
    BybitPerpAccountResult, BybitPerpApiResponse, BybitPerpExchangeInfo,
    BybitPerpFundingRateResponse, BybitPerpKlineResponse, BybitPerpOrderRequest,
    BybitPerpOrderResponse, BybitPerpPositionResult, BybitPerpTickerResponse,
};
use serde_json::Value;

/// Thin typed wrapper around `RestClient` for Bybit Perpetual API
pub struct BybitPerpRestClient<R: RestClient> {
    client: R,
}

impl<R: RestClient> BybitPerpRestClient<R> {
    pub fn new(client: R) -> Self {
        Self { client }
    }

    /// Get all perpetual markets
    pub async fn get_markets(
        &self,
    ) -> Result<BybitPerpApiResponse<BybitPerpExchangeInfo>, ExchangeError> {
        let params = [("category", "linear")];
        self.client
            .get_json("/v5/market/instruments-info", &params, false)
            .await
    }

    /// Get klines for a symbol
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<BybitPerpKlineResponse, ExchangeError> {
        let mut params = vec![
            ("category", "linear"),
            ("symbol", symbol),
            ("interval", interval),
        ];

        let limit_str;
        if let Some(limit_val) = limit {
            limit_str = limit_val.to_string();
            params.push(("limit", &limit_str));
        }

        let start_str;
        if let Some(start) = start_time {
            start_str = start.to_string();
            params.push(("start", &start_str));
        }

        let end_str;
        if let Some(end) = end_time {
            end_str = end.to_string();
            params.push(("end", &end_str));
        }

        self.client
            .get_json("/v5/market/kline", &params, false)
            .await
    }

    /// Get ticker information for symbols
    pub async fn get_tickers(
        &self,
        symbol: Option<&str>,
    ) -> Result<BybitPerpTickerResponse, ExchangeError> {
        let mut params = vec![("category", "linear")];

        if let Some(sym) = symbol {
            params.push(("symbol", sym));
        }

        self.client
            .get_json("/v5/market/tickers", &params, false)
            .await
    }

    /// Get funding rate information
    pub async fn get_funding_rate(
        &self,
        symbol: &str,
    ) -> Result<BybitPerpFundingRateResponse, ExchangeError> {
        let params = [("category", "linear"), ("symbol", symbol)];
        self.client
            .get_json("/v5/market/funding/history", &params, false)
            .await
    }

    /// Get all funding rates
    pub async fn get_all_funding_rates(
        &self,
    ) -> Result<BybitPerpFundingRateResponse, ExchangeError> {
        let params = [("category", "linear")];
        self.client
            .get_json("/v5/market/funding/history", &params, false)
            .await
    }

    /// Get account balance
    pub async fn get_account_balance(
        &self,
    ) -> Result<BybitPerpApiResponse<BybitPerpAccountResult>, ExchangeError> {
        let params = [("accountType", "UNIFIED")];
        self.client
            .get_json("/v5/account/wallet-balance", &params, true)
            .await
    }

    /// Get positions
    pub async fn get_positions(
        &self,
        settle_coin: Option<&str>,
    ) -> Result<BybitPerpApiResponse<BybitPerpPositionResult>, ExchangeError> {
        let mut params = vec![("category", "linear")];

        if let Some(coin) = settle_coin {
            params.push(("settleCoin", coin));
        } else {
            params.push(("settleCoin", "USDT"));
        }

        self.client
            .get_json("/v5/position/list", &params, true)
            .await
    }

    /// Place an order
    pub async fn place_order(
        &self,
        order: &BybitPerpOrderRequest,
    ) -> Result<BybitPerpApiResponse<BybitPerpOrderResponse>, ExchangeError> {
        let body = serde_json::to_value(order)?;
        self.client.post_json("/v5/order/create", &body, true).await
    }

    /// Cancel an order
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> Result<BybitPerpApiResponse<Value>, ExchangeError> {
        let request_body = serde_json::json!({
            "category": "linear",
            "symbol": symbol,
            "orderId": order_id
        });

        self.client
            .post_json("/v5/order/cancel", &request_body, true)
            .await
    }

    /// Get order history
    pub async fn get_order_history(
        &self,
        symbol: Option<&str>,
        limit: Option<u32>,
    ) -> Result<BybitPerpApiResponse<Value>, ExchangeError> {
        let mut params = vec![("category", "linear")];

        if let Some(sym) = symbol {
            params.push(("symbol", sym));
        }

        let limit_str;
        if let Some(limit_val) = limit {
            limit_str = limit_val.to_string();
            params.push(("limit", &limit_str));
        }

        self.client
            .get_json("/v5/order/history", &params, true)
            .await
    }

    /// Get order book
    pub async fn get_order_book(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<BybitPerpApiResponse<Value>, ExchangeError> {
        let mut params = vec![("category", "linear"), ("symbol", symbol)];

        let limit_str;
        if let Some(limit_val) = limit {
            limit_str = limit_val.to_string();
            params.push(("limit", &limit_str));
        }

        self.client
            .get_json("/v5/market/orderbook", &params, false)
            .await
    }

    /// Get recent trades
    pub async fn get_recent_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<BybitPerpApiResponse<Value>, ExchangeError> {
        let mut params = vec![("category", "linear"), ("symbol", symbol)];

        let limit_str;
        if let Some(limit_val) = limit {
            limit_str = limit_val.to_string();
            params.push(("limit", &limit_str));
        }

        self.client
            .get_json("/v5/market/recent-trade", &params, false)
            .await
    }
}
