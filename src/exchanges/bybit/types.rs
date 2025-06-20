use serde::{Deserialize, Serialize};
use thiserror::Error;

// Bybit-specific error types following HFT error handling guidelines
#[derive(Error, Debug)]
pub enum BybitError {
    #[error("API error {code}: {message}")]
    ApiError { code: i32, message: String },

    #[error("Authentication failed: {reason}")]
    AuthError { reason: String },

    #[error("Invalid order parameters: {details}")]
    InvalidOrder { details: String },

    #[error("Network request failed")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parsing failed")]
    JsonError(#[from] serde_json::Error),

    #[error("Rate limit exceeded for endpoint: {endpoint}")]
    RateLimit { endpoint: String },

    #[error("Symbol not found: {symbol}")]
    SymbolNotFound { symbol: String },

    #[error("Insufficient balance for operation")]
    InsufficientBalance,
}

impl BybitError {
    /// Mark cold error paths to keep happy path in I-cache
    #[cold]
    #[inline(never)]
    pub fn api_error(code: i32, message: String) -> Self {
        Self::ApiError { code, message }
    }

    #[cold]
    #[inline(never)]
    pub fn auth_error(reason: String) -> Self {
        Self::AuthError { reason }
    }

    #[cold]
    #[inline(never)]
    pub fn invalid_order(details: String) -> Self {
        Self::InvalidOrder { details }
    }

    #[cold]
    #[inline(never)]
    pub fn rate_limit(endpoint: String) -> Self {
        Self::RateLimit { endpoint }
    }

    #[cold]
    #[inline(never)]
    pub fn symbol_not_found(symbol: String) -> Self {
        Self::SymbolNotFound { symbol }
    }
}

// Helper trait for adding context to Bybit operations
pub trait BybitResultExt<T> {
    fn with_symbol_context(self, symbol: &str) -> Result<T, BybitError>;
    fn with_order_context(self, symbol: &str, side: &str) -> Result<T, BybitError>;
}

impl<T, E> BybitResultExt<T> for Result<T, E>
where
    E: Into<BybitError>,
{
    fn with_symbol_context(self, symbol: &str) -> Result<T, BybitError> {
        self.map_err(|e| {
            let error = e.into();
            // Attach lightweight breadcrumb context
            match &error {
                BybitError::NetworkError(req_err) => {
                    tracing::error!(symbol = %symbol, error = %req_err, "Network error");
                }
                BybitError::JsonError(json_err) => {
                    tracing::error!(symbol = %symbol, error = %json_err, "JSON parsing error");
                }
                _ => {
                    tracing::error!(symbol = %symbol, error = %error, "Bybit operation failed");
                }
            }
            error
        })
    }

    fn with_order_context(self, symbol: &str, side: &str) -> Result<T, BybitError> {
        self.map_err(|e| {
            let error = e.into();
            tracing::error!(symbol = %symbol, side = %side, error = %error, "Order operation failed");
            error
        })
    }
}

// API response wrapper for V5 API
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitApiResponse<T> {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: T,
}

// Market data types
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitMarket {
    pub symbol: String,
    pub status: String,
    #[serde(rename = "baseCoin")]
    pub base_coin: String,
    #[serde(rename = "quoteCoin")]
    pub quote_coin: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitMarketsResult {
    pub list: Vec<BybitMarket>,
}

// Account balance types for UNIFIED account
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitCoinBalance {
    pub coin: String,
    #[serde(rename = "walletBalance")]
    pub wallet_balance: String,
    pub locked: String,
    pub equity: String,
    #[serde(rename = "usdValue")]
    pub usd_value: String,
    #[serde(rename = "availableToWithdraw")]
    pub available_to_withdraw: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitAccountList {
    #[serde(rename = "accountType")]
    pub account_type: String,
    pub coin: Vec<BybitCoinBalance>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitAccountResult {
    pub list: Vec<BybitAccountList>,
}

#[derive(Debug, Deserialize)]
pub struct BybitExchangeInfo {
    pub category: String,
    pub list: Vec<BybitMarket>,
}

#[derive(Debug, Deserialize)]
pub struct BybitLotSizeFilter {
    #[serde(rename = "basePrecision")]
    pub base_precision: String,
    #[serde(rename = "quotePrecision")]
    pub quote_precision: String,
    #[serde(rename = "minOrderQty")]
    pub min_order_qty: String,
    #[serde(rename = "maxOrderQty")]
    pub max_order_qty: String,
    #[serde(rename = "minOrderAmt")]
    pub min_order_amt: String,
    #[serde(rename = "maxOrderAmt")]
    pub max_order_amt: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitPriceFilter {
    #[serde(rename = "tickSize")]
    pub tick_size: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitAccountInfo {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: BybitAccountResult,
}

#[derive(Debug, Deserialize)]
pub struct BybitFilter {
    pub filter_type: String,
    pub min_price: Option<String>,
    pub max_price: Option<String>,
    pub min_qty: Option<String>,
    pub max_qty: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BybitV5Result {
    pub category: String,
    pub list: Vec<BybitMarket>,
}

#[derive(Debug, Serialize)]
pub struct BybitOrderRequest {
    pub category: String,
    pub symbol: String,
    pub side: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    pub qty: String,
    pub price: Option<String>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<String>,
    #[serde(rename = "stopPrice")]
    pub stop_price: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BybitOrderResponse {
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "orderLinkId")]
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    pub qty: String,
    pub price: String,
    #[serde(rename = "orderStatus")]
    pub status: String,
    #[serde(rename = "createdTime")]
    pub timestamp: i64,
}

// WebSocket Types
#[derive(Debug, Deserialize)]
pub struct BybitWebSocketTicker {
    pub symbol: String,
    pub price: String,
    pub price_24h_pcnt: String,
    pub price_1h_pcnt: String,
    pub high_price_24h: String,
    pub low_price_24h: String,
    pub turnover_24h: String,
    pub volume_24h: String,
    pub usd_index_price: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitWebSocketOrderBook {
    pub symbol: String,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
    pub timestamp: String,
    pub update_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct BybitWebSocketTrade {
    pub symbol: String,
    pub price: String,
    pub size: String,
    pub side: String,
    pub timestamp: String,
    pub trade_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitWebSocketKline {
    pub symbol: String,
    pub kline: BybitKlineData,
}

#[derive(Debug, Deserialize)]
pub struct BybitKlineData {
    pub start_time: i64,
    pub end_time: i64,
    pub interval: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub close_price: String,
    pub volume: String,
    pub turnover: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitRestKline {
    pub start_time: i64,
    pub end_time: i64,
    pub interval: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub close_price: String,
    pub volume: String,
    pub turnover: String,
}

// Add kline response types for V5 API
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitKlineResult {
    pub symbol: String,
    pub category: String,
    pub list: Vec<Vec<String>>, // Array of arrays containing kline data
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitKlineResponse {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: BybitKlineResult,
}
