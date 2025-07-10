use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::types::KlineInterval;
use crate::exchanges::binance_perp::types::{
    BinancePerpBalance, BinancePerpExchangeInfo, BinancePerpFundingRate, BinancePerpOrderResponse,
    BinancePerpPosition, BinancePerpPremiumIndex, BinancePerpRestKline,
    BinancePerpWebSocketOrderBook, BinancePerpWebSocketTicker, BinancePerpWebSocketTrade,
};
use serde_json::Value;
use tracing::instrument;

/// REST API operations for Binance Perpetual
pub struct BinancePerpRestClient<R: RestClient> {
    rest: R,
}

impl<R: RestClient> BinancePerpRestClient<R> {
    /// Create a new REST client wrapper
    pub fn new(rest: R) -> Self {
        Self { rest }
    }

    /// Get exchange information
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    pub async fn get_exchange_info(&self) -> Result<BinancePerpExchangeInfo, ExchangeError> {
        self.rest
            .get_json("/fapi/v1/exchangeInfo", &[], false)
            .await
    }

    /// Get ticker for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol))]
    pub async fn get_ticker(
        &self,
        symbol: &str,
    ) -> Result<BinancePerpWebSocketTicker, ExchangeError> {
        let params = [("symbol", symbol)];
        self.rest
            .get_json("/fapi/v1/ticker/24hr", &params, false)
            .await
    }

    /// Get order book for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol))]
    pub async fn get_order_book(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<BinancePerpWebSocketOrderBook, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json("/fapi/v1/depth", &params, false).await
    }

    /// Get recent trades for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol))]
    pub async fn get_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<Vec<BinancePerpWebSocketTrade>, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest
            .get_json("/fapi/v1/aggTrades", &params, false)
            .await
    }

    /// Get klines for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol, interval = %interval))]
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<BinancePerpRestKline>, ExchangeError> {
        let interval_str = interval.to_binance_format();
        let limit_str = limit.map(|l| l.to_string());
        let start_time_str = start_time.map(|t| t.to_string());
        let end_time_str = end_time.map(|t| t.to_string());

        let mut params = vec![("symbol", symbol), ("interval", &interval_str)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }
        if let Some(ref start_time) = start_time_str {
            params.push(("startTime", start_time.as_str()));
        }
        if let Some(ref end_time) = end_time_str {
            params.push(("endTime", end_time.as_str()));
        }

        self.rest.get_json("/fapi/v1/klines", &params, false).await
    }

    /// Get funding rate for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol))]
    pub async fn get_funding_rate(
        &self,
        symbol: &str,
    ) -> Result<BinancePerpFundingRate, ExchangeError> {
        let params = [("symbol", symbol)];
        self.rest
            .get_json("/fapi/v1/fundingRate", &params, false)
            .await
    }

    /// Get all funding rates
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    pub async fn get_all_funding_rates(
        &self,
    ) -> Result<Vec<BinancePerpFundingRate>, ExchangeError> {
        self.rest.get_json("/fapi/v1/fundingRate", &[], false).await
    }

    /// Get premium index for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol))]
    pub async fn get_premium_index(
        &self,
        symbol: &str,
    ) -> Result<BinancePerpPremiumIndex, ExchangeError> {
        let params = [("symbol", symbol)];
        self.rest
            .get_json("/fapi/v1/premiumIndex", &params, false)
            .await
    }

    /// Get account information (authenticated)
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    pub async fn get_account_info(
        &self,
    ) -> Result<crate::exchanges::binance_perp::types::BinancePerpAccountInfo, ExchangeError> {
        self.rest.get_json("/fapi/v2/account", &[], true).await
    }

    /// Get account balance (authenticated)
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    pub async fn get_balance(&self) -> Result<Vec<BinancePerpBalance>, ExchangeError> {
        self.rest.get_json("/fapi/v2/balance", &[], true).await
    }

    /// Get account positions (authenticated)
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    pub async fn get_positions(&self) -> Result<Vec<BinancePerpPosition>, ExchangeError> {
        self.rest.get_json("/fapi/v2/positionRisk", &[], true).await
    }

    /// Place a new order (authenticated)
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    pub async fn place_order(
        &self,
        body: &Value,
    ) -> Result<BinancePerpOrderResponse, ExchangeError> {
        self.rest.post_json("/fapi/v1/order", body, true).await
    }

    /// Cancel an order (authenticated)
    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol))]
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
    ) -> Result<BinancePerpOrderResponse, ExchangeError> {
        let order_id_str = order_id.map(|id| id.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref order_id) = order_id_str {
            params.push(("orderId", order_id.as_str()));
        }
        if let Some(orig_client_order_id) = orig_client_order_id {
            params.push(("origClientOrderId", orig_client_order_id));
        }

        self.rest.delete_json("/fapi/v1/order", &params, true).await
    }

    /// Get historical funding rates for a symbol
    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol))]
    pub async fn get_funding_rate_history(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<BinancePerpFundingRate>, ExchangeError> {
        let start_time_str = start_time.map(|t| t.to_string());
        let end_time_str = end_time.map(|t| t.to_string());
        let limit_str = limit.map(|l| l.to_string());

        let mut params = vec![("symbol", symbol)];

        if let Some(ref start_time) = start_time_str {
            params.push(("startTime", start_time.as_str()));
        }
        if let Some(ref end_time) = end_time_str {
            params.push(("endTime", end_time.as_str()));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest
            .get_json("/fapi/v1/fundingRate", &params, false)
            .await
    }
}
