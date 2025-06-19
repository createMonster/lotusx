use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpApiResponse<T> {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpExchangeInfo {
    pub category: String,
    pub list: Vec<BybitPerpMarket>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpMarket {
    pub symbol: String,
    pub status: String,
    #[serde(rename = "baseCoin")]
    pub base_coin: String,
    #[serde(rename = "quoteCoin")]
    pub quote_coin: String,
    #[serde(rename = "settleCoin")]
    pub settle_coin: String,
    #[serde(rename = "priceScale")]
    pub price_scale: String, // V5 API returns this as string
    #[serde(rename = "lotSizeFilter")]
    pub lot_size_filter: BybitPerpLotSizeFilter,
    #[serde(rename = "priceFilter")]
    pub price_filter: BybitPerpPriceFilter,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpLotSizeFilter {
    #[serde(rename = "minOrderQty")]
    pub min_order_qty: String,
    #[serde(rename = "maxOrderQty")]
    pub max_order_qty: String,
    #[serde(rename = "qtyStep")]
    pub qty_step: String,
    #[serde(rename = "minNotionalValue")]
    pub min_notional_value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpPriceFilter {
    #[serde(rename = "minPrice")]
    pub min_price: String,
    #[serde(rename = "maxPrice")]
    pub max_price: String,
    #[serde(rename = "tickSize")]
    pub tick_size: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpLeverageFilter {
    #[serde(rename = "minLeverage")]
    pub min_leverage: String,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: String,
    #[serde(rename = "leverageStep")]
    pub leverage_step: String,
}

// Account balance response structures
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpCoinBalance {
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
pub struct BybitPerpAccountList {
    #[serde(rename = "accountType")]
    pub account_type: String,
    pub coin: Vec<BybitPerpCoinBalance>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpAccountResult {
    pub list: Vec<BybitPerpAccountList>,
}

// Position response structures
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpPosition {
    pub symbol: String,
    pub side: String,
    pub size: String,
    #[serde(rename = "avgPrice")]
    pub entry_price: String,
    #[serde(rename = "unrealisedPnl")]
    pub unrealised_pnl: String,
    #[serde(rename = "liqPrice")]
    pub liquidation_price: String,
    pub leverage: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpPositionResult {
    pub list: Vec<BybitPerpPosition>,
    pub category: String,
    #[serde(rename = "nextPageCursor")]
    pub next_page_cursor: String,
}

#[derive(Debug, Serialize)]
pub struct BybitPerpOrderRequest {
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
pub struct BybitPerpOrderResponse {
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

// WebSocket Types for Perpetual Futures
#[derive(Debug, Deserialize)]
pub struct BybitPerpWebSocketMessage {
    pub topic: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: serde_json::Value,
    pub ts: i64,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpTickerData {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "prevPrice24h")]
    pub prev_price_24h: String,
    #[serde(rename = "price24hPcnt")]
    pub price_24h_pcnt: String,
    #[serde(rename = "highPrice24h")]
    pub high_price_24h: String,
    #[serde(rename = "lowPrice24h")]
    pub low_price_24h: String,
    #[serde(rename = "volume24h")]
    pub volume_24h: String,
    #[serde(rename = "turnover24h")]
    pub turnover_24h: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpOrderBookData {
    pub symbol: String,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
    pub ts: i64,
    pub u: i64,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpTradeData {
    pub symbol: String,
    pub side: String,
    pub size: String,
    pub price: String,
    #[serde(rename = "tradeTimeMs")]
    pub trade_time_ms: i64,
    #[serde(rename = "tradeId")]
    pub trade_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpKlineData {
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
pub struct BybitPerpMarketsResult {
    pub list: Vec<BybitPerpMarket>,
}

// REST API K-line Types
#[derive(Debug, Deserialize)]
pub struct BybitPerpRestKline {
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

// Bybit Perpetual-specific error types following HFT error handling guidelines
#[derive(Error, Debug)]
pub enum BybitPerpError {
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

    #[error("Contract not found: {symbol}")]
    ContractNotFound { symbol: String },

    #[error("Insufficient margin for position")]
    InsufficientMargin,

    #[error("Position size exceeds limit: max={max}, requested={requested}")]
    PositionSizeExceeded { max: String, requested: String },

    #[error("Leverage out of range: min={min}, max={max}, requested={requested}")]
    InvalidLeverage {
        min: String,
        max: String,
        requested: String,
    },
}

impl BybitPerpError {
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
    pub fn contract_not_found(symbol: String) -> Self {
        Self::ContractNotFound { symbol }
    }

    #[cold]
    #[inline(never)]
    pub fn position_size_exceeded(max: String, requested: String) -> Self {
        Self::PositionSizeExceeded { max, requested }
    }

    #[cold]
    #[inline(never)]
    pub fn invalid_leverage(min: String, max: String, requested: String) -> Self {
        Self::InvalidLeverage {
            min,
            max,
            requested,
        }
    }
}

// Helper trait for adding context to BybitPerp operations
pub trait BybitPerpResultExt<T> {
    fn with_contract_context(self, symbol: &str) -> Result<T, BybitPerpError>;
    fn with_position_context(
        self,
        symbol: &str,
        side: &str,
        quantity: &str,
    ) -> Result<T, BybitPerpError>;
}

impl<T, E> BybitPerpResultExt<T> for Result<T, E>
where
    E: Into<BybitPerpError>,
{
    fn with_contract_context(self, symbol: &str) -> Result<T, BybitPerpError> {
        self.map_err(|e| {
            let error = e.into();
            // Attach lightweight breadcrumb context
            match &error {
                BybitPerpError::NetworkError(req_err) => {
                    tracing::error!(contract = %symbol, error = %req_err, "Network error");
                }
                BybitPerpError::JsonError(json_err) => {
                    tracing::error!(contract = %symbol, error = %json_err, "JSON parsing error");
                }
                _ => {
                    tracing::error!(contract = %symbol, error = %error, "BybitPerp operation failed");
                }
            }
            error
        })
    }

    fn with_position_context(
        self,
        symbol: &str,
        side: &str,
        quantity: &str,
    ) -> Result<T, BybitPerpError> {
        self.map_err(|e| {
            let error = e.into();
            tracing::error!(
                contract = %symbol,
                side = %side,
                quantity = %quantity,
                error = %error,
                "Position operation failed"
            );
            error
        })
    }
}
