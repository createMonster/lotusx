use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::types::KlineInterval;
use crate::exchanges::binance::types::{
    BinanceAccountInfo, BinanceExchangeInfo, BinanceOrderResponse, BinanceRestKline,
};
use serde_json::Value;

/// Thin typed wrapper around `RestClient` for Binance API
pub struct BinanceRestClient<R: RestClient> {
    client: R,
}

impl<R: RestClient> BinanceRestClient<R> {
    pub fn new(client: R) -> Self {
        Self { client }
    }

    /// Get exchange information
    pub async fn get_exchange_info(&self) -> Result<BinanceExchangeInfo, ExchangeError> {
        self.client
            .get_json("/api/v3/exchangeInfo", &[], false)
            .await
    }

    /// Get klines/candlestick data
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<BinanceRestKline>, ExchangeError> {
        let interval_str = interval.to_binance_format();
        let mut params = vec![("symbol", symbol), ("interval", interval_str.as_str())];

        let limit_str;
        let start_time_str;
        let end_time_str;

        if let Some(limit) = limit {
            limit_str = limit.to_string();
            params.push(("limit", limit_str.as_str()));
        }
        if let Some(start_time) = start_time {
            start_time_str = start_time.to_string();
            params.push(("startTime", start_time_str.as_str()));
        }
        if let Some(end_time) = end_time {
            end_time_str = end_time.to_string();
            params.push(("endTime", end_time_str.as_str()));
        }

        self.client.get_json("/api/v3/klines", &params, false).await
    }

    /// Get account information
    pub async fn get_account_info(&self) -> Result<BinanceAccountInfo, ExchangeError> {
        self.client.get_json("/api/v3/account", &[], true).await
    }

    /// Place an order
    pub async fn place_order(&self, order: &Value) -> Result<BinanceOrderResponse, ExchangeError> {
        self.client.post_json("/api/v3/order", order, true).await
    }

    /// Cancel an order
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
    ) -> Result<BinanceOrderResponse, ExchangeError> {
        let mut params = vec![("symbol", symbol)];

        let order_id_str;
        if let Some(order_id) = order_id {
            order_id_str = order_id.to_string();
            params.push(("orderId", order_id_str.as_str()));
        }
        if let Some(orig_client_order_id) = orig_client_order_id {
            params.push(("origClientOrderId", orig_client_order_id));
        }

        self.client
            .delete_json("/api/v3/order", &params, true)
            .await
    }
}

/// Extension trait for `KlineInterval` to support Binance format
pub trait BinanceKlineInterval {
    fn to_binance_format(&self) -> &str;
}

impl BinanceKlineInterval for KlineInterval {
    fn to_binance_format(&self) -> &str {
        match self {
            Self::Seconds1 => "1s",
            Self::Minutes1 => "1m",
            Self::Minutes3 => "3m",
            Self::Minutes5 => "5m",
            Self::Minutes15 => "15m",
            Self::Minutes30 => "30m",
            Self::Hours1 => "1h",
            Self::Hours2 => "2h",
            Self::Hours4 => "4h",
            Self::Hours6 => "6h",
            Self::Hours8 => "8h",
            Self::Hours12 => "12h",
            Self::Days1 => "1d",
            Self::Days3 => "3d",
            Self::Weeks1 => "1w",
            Self::Months1 => "1M",
        }
    }
}
