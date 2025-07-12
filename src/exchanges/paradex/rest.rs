use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::types::KlineInterval;
use crate::exchanges::paradex::types::{
    ParadexBalance, ParadexFundingRate, ParadexFundingRateHistory, ParadexMarket, ParadexOrder,
    ParadexPosition,
};
use serde_json::Value;

/// Thin typed wrapper around `RestClient` for Paradex API
pub struct ParadexRestClient<R: RestClient> {
    client: R,
}

impl<R: RestClient> ParadexRestClient<R> {
    pub fn new(client: R) -> Self {
        Self { client }
    }

    /// Get all available markets
    #[allow(clippy::option_if_let_else)]
    pub async fn get_markets(&self) -> Result<Vec<ParadexMarket>, ExchangeError> {
        let response: serde_json::Value = self.client.get_json("/v1/markets", &[], false).await?;

        // Handle different response formats
        if let Ok(markets_array) = serde_json::from_value::<Vec<ParadexMarket>>(response.clone()) {
            Ok(markets_array)
        } else if let Some(data) = response.get("data") {
            serde_json::from_value::<Vec<ParadexMarket>>(data.clone())
                .map_err(|e| ExchangeError::Other(format!("Failed to parse markets: {}", e)))
        } else {
            Err(ExchangeError::Other(
                "Unexpected markets response format".to_string(),
            ))
        }
    }

    /// Get klines/candlestick data
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Value, ExchangeError> {
        let interval_str = interval.to_paradex_format();
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

        self.client.get_json("/v1/klines", &params, false).await
    }

    /// Get funding rates for symbols
    #[allow(clippy::option_if_let_else)]
    pub async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<ParadexFundingRate>, ExchangeError> {
        let endpoint = symbols.map_or_else(
            || "/v1/funding/rates".to_string(),
            |symbols| format!("/v1/funding/rates?symbols={}", symbols.join(",")),
        );

        let response: serde_json::Value = self.client.get_json(&endpoint, &[], false).await?;

        // Handle different response formats
        if let Ok(rates_array) = serde_json::from_value::<Vec<ParadexFundingRate>>(response.clone())
        {
            Ok(rates_array)
        } else if let Some(data) = response.get("data") {
            serde_json::from_value::<Vec<ParadexFundingRate>>(data.clone())
                .map_err(|e| ExchangeError::Other(format!("Failed to parse funding rates: {}", e)))
        } else {
            Err(ExchangeError::Other(
                "Unexpected funding rates response format".to_string(),
            ))
        }
    }

    /// Get funding rate history for a symbol
    #[allow(clippy::option_if_let_else)]
    pub async fn get_funding_rate_history(
        &self,
        symbol: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<ParadexFundingRateHistory>, ExchangeError> {
        let mut params = vec![("symbol", symbol)];
        let start_time_str;
        let end_time_str;
        let limit_str;

        if let Some(start) = start_time {
            start_time_str = start.to_string();
            params.push(("start_time", &start_time_str));
        }
        if let Some(end) = end_time {
            end_time_str = end.to_string();
            params.push(("end_time", &end_time_str));
        }
        if let Some(limit) = limit {
            limit_str = limit.to_string();
            params.push(("limit", &limit_str));
        }

        let response: serde_json::Value = self
            .client
            .get_json("/v1/funding/history", &params, false)
            .await?;

        // Handle different response formats
        if let Ok(history_array) =
            serde_json::from_value::<Vec<ParadexFundingRateHistory>>(response.clone())
        {
            Ok(history_array)
        } else if let Some(data) = response.get("data") {
            serde_json::from_value::<Vec<ParadexFundingRateHistory>>(data.clone()).map_err(|e| {
                ExchangeError::Other(format!("Failed to parse funding rate history: {}", e))
            })
        } else {
            Err(ExchangeError::Other(
                "Unexpected funding rate history response format".to_string(),
            ))
        }
    }

    /// Place an order
    pub async fn place_order(&self, order: &Value) -> Result<ParadexOrder, ExchangeError> {
        self.client.post_json("/v1/orders", order, true).await
    }

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: &str) -> Result<Value, ExchangeError> {
        let endpoint = format!("/v1/orders/{}", order_id);
        self.client.delete_json(&endpoint, &[], true).await
    }

    /// Get account balances
    #[allow(clippy::option_if_let_else)]
    pub async fn get_account_balances(&self) -> Result<Vec<ParadexBalance>, ExchangeError> {
        let response: serde_json::Value = self
            .client
            .get_json("/v1/account/balances", &[], true)
            .await?;

        // Handle different response formats
        if let Ok(balances_array) = serde_json::from_value::<Vec<ParadexBalance>>(response.clone())
        {
            Ok(balances_array)
        } else if let Some(data) = response.get("data") {
            serde_json::from_value::<Vec<ParadexBalance>>(data.clone())
                .map_err(|e| ExchangeError::Other(format!("Failed to parse balances: {}", e)))
        } else {
            Err(ExchangeError::Other(
                "Unexpected balances response format".to_string(),
            ))
        }
    }

    /// Get account positions
    #[allow(clippy::option_if_let_else)]
    pub async fn get_positions(&self) -> Result<Vec<ParadexPosition>, ExchangeError> {
        let response: serde_json::Value = self
            .client
            .get_json("/v1/account/positions", &[], true)
            .await?;

        // Handle different response formats
        if let Ok(positions_array) =
            serde_json::from_value::<Vec<ParadexPosition>>(response.clone())
        {
            Ok(positions_array)
        } else if let Some(data) = response.get("data") {
            serde_json::from_value::<Vec<ParadexPosition>>(data.clone())
                .map_err(|e| ExchangeError::Other(format!("Failed to parse positions: {}", e)))
        } else {
            Err(ExchangeError::Other(
                "Unexpected positions response format".to_string(),
            ))
        }
    }
}

/// Extension trait for `KlineInterval` to support Paradex format
pub trait ParadexKlineInterval {
    fn to_paradex_format(&self) -> String;
}

impl ParadexKlineInterval for KlineInterval {
    fn to_paradex_format(&self) -> String {
        match self {
            Self::Seconds1 => "1s".to_string(),
            Self::Minutes1 => "1m".to_string(),
            Self::Minutes3 => "3m".to_string(),
            Self::Minutes5 => "5m".to_string(),
            Self::Minutes15 => "15m".to_string(),
            Self::Minutes30 => "30m".to_string(),
            Self::Hours1 => "1h".to_string(),
            Self::Hours2 => "2h".to_string(),
            Self::Hours4 => "4h".to_string(),
            Self::Hours6 => "6h".to_string(),
            Self::Hours8 => "8h".to_string(),
            Self::Hours12 => "12h".to_string(),
            Self::Days1 => "1d".to_string(),
            Self::Days3 => "3d".to_string(),
            Self::Weeks1 => "1w".to_string(),
            Self::Months1 => "1M".to_string(),
        }
    }
}
