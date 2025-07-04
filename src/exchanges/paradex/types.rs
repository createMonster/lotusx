use serde::{Deserialize, Serialize};
use thiserror::Error;

// Paradex-specific error types following HFT error handling guidelines
#[derive(Error, Debug)]
pub enum ParadexError {
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

    #[error("JWT signing failed")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Rate limit exceeded for endpoint: {endpoint}")]
    RateLimit { endpoint: String },

    #[error("Market not found: {symbol}")]
    MarketNotFound { symbol: String },

    #[error("Insufficient balance for operation")]
    InsufficientBalance,

    #[error("WebSocket connection failed: {reason}")]
    WebSocketError { reason: String },

    #[error("Funding rate error: {message}, symbol={symbol:?}")]
    FundingRateError {
        message: String,
        symbol: Option<String>,
    },
}

impl ParadexError {
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
    pub fn market_not_found(symbol: String) -> Self {
        Self::MarketNotFound { symbol }
    }

    #[cold]
    #[inline(never)]
    pub fn websocket_error(reason: String) -> Self {
        Self::WebSocketError { reason }
    }

    #[cold]
    #[inline(never)]
    pub fn funding_rate_error(message: String, symbol: Option<String>) -> Self {
        Self::FundingRateError { message, symbol }
    }

    #[cold]
    #[inline(never)]
    pub fn network_error(message: String) -> Self {
        Self::ApiError { code: 0, message }
    }

    #[cold]
    #[inline(never)]
    pub fn parse_error(message: String, _context: Option<String>) -> Self {
        Self::ApiError { code: 1, message }
    }
}

// Helper trait for adding context to Paradex operations
pub trait ParadexResultExt<T> {
    fn with_symbol_context(self, symbol: &str) -> Result<T, ParadexError>;
    fn with_exchange_context<F>(self, context_fn: F) -> Result<T, ParadexError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ParadexResultExt<T> for Result<T, E>
where
    E: Into<ParadexError>,
{
    fn with_symbol_context(self, symbol: &str) -> Result<T, ParadexError> {
        self.map_err(|e| {
            let base_error = e.into();
            match base_error {
                ParadexError::NetworkError(_) => {
                    ParadexError::api_error(0, format!("Network error for symbol {}", symbol))
                }
                _ => base_error,
            }
        })
    }

    fn with_exchange_context<F>(self, context_fn: F) -> Result<T, ParadexError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            match base_error {
                ParadexError::NetworkError(_) => ParadexError::api_error(0, context_fn()),
                _ => base_error,
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexAsset {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexMarket {
    pub symbol: String,
    pub base_asset: ParadexAsset,
    pub quote_asset: ParadexAsset,
    pub status: String,
    pub state: String,
    pub tick_size: String,
    pub step_size: String,
    pub min_order_size: String,
    pub max_order_size: String,
    pub min_price: String,
    pub max_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexOrder {
    pub id: String,
    pub client_id: String,
    pub market: String,
    pub side: String,
    pub order_type: String,
    pub size: String,
    pub price: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexFill {
    pub id: i64,
    pub market: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub fee: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexPosition {
    pub market: String,
    pub side: String,
    pub average_entry_price: String,
    pub size: String,
    pub unrealized_pnl: String,
    pub liquidation_price: Option<String>,
    pub leverage: String,
}

// API Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ParadexApiError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexApiError {
    pub code: i32,
    pub message: String,
}

// Funding rate types for perpetual trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexFundingRate {
    pub symbol: String,
    pub funding_rate: String,
    pub next_funding_time: i64,
    pub mark_price: String,
    pub index_price: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexFundingRateHistory {
    pub symbol: String,
    pub funding_rate: String,
    pub funding_time: i64,
}

// Balance and account types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexBalance {
    pub asset: String,
    pub available: String,
    pub locked: String,
    pub total: String,
}

// WebSocket types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWebSocketMessage {
    pub channel: String,
    pub data: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWebSocketSubscription {
    pub method: String,
    pub params: Vec<String>,
    pub id: u64,
}
