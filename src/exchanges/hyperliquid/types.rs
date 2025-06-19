use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// Hyperliquid-specific error types following HFT error handling guidelines
#[derive(Error, Debug)]
pub enum HyperliquidError {
    #[error("API error: {message}")]
    ApiError { message: String },

    #[error("Authentication failed: {reason}")]
    AuthError { reason: String },

    #[error("Invalid order parameters: {details}")]
    InvalidOrder { details: String },

    #[error("Network request failed")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parsing failed")]
    JsonError(#[from] serde_json::Error),

    #[error("Rate limit exceeded for operation: {operation}")]
    RateLimit { operation: String },

    #[error("Asset not found: {symbol}")]
    AssetNotFound { symbol: String },

    #[error("Insufficient margin for position")]
    InsufficientMargin,

    #[error("Position size exceeds limit: max={max}, requested={requested}")]
    PositionSizeExceeded { max: String, requested: String },

    #[error("Invalid signature or nonce")]
    SignatureError,

    #[error("Vault operation not supported: {operation}")]
    VaultError { operation: String },

    #[error("WebSocket connection failed: {reason}")]
    WebSocketError { reason: String },
}

impl HyperliquidError {
    /// Mark cold error paths to keep happy path in I-cache
    #[cold]
    #[inline(never)]
    pub fn api_error(message: String) -> Self {
        Self::ApiError { message }
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
    pub fn rate_limit(operation: String) -> Self {
        Self::RateLimit { operation }
    }

    #[cold]
    #[inline(never)]
    pub fn asset_not_found(symbol: String) -> Self {
        Self::AssetNotFound { symbol }
    }

    #[cold]
    #[inline(never)]
    pub fn position_size_exceeded(max: String, requested: String) -> Self {
        Self::PositionSizeExceeded { max, requested }
    }

    #[cold]
    #[inline(never)]
    pub fn vault_error(operation: String) -> Self {
        Self::VaultError { operation }
    }

    #[cold]
    #[inline(never)]
    pub fn websocket_error(reason: String) -> Self {
        Self::WebSocketError { reason }
    }
}

// Helper trait for adding context to Hyperliquid operations
pub trait HyperliquidResultExt<T> {
    fn with_symbol_context(self, symbol: &str) -> Result<T, HyperliquidError>;
    fn with_order_context(self, symbol: &str, side: &str) -> Result<T, HyperliquidError>;
    fn with_vault_context(self, vault_address: &str) -> Result<T, HyperliquidError>;
}

impl<T, E> HyperliquidResultExt<T> for Result<T, E>
where
    E: Into<HyperliquidError>,
{
    fn with_symbol_context(self, symbol: &str) -> Result<T, HyperliquidError> {
        self.map_err(|e| {
            let error = e.into();
            // Attach lightweight breadcrumb context
            match &error {
                HyperliquidError::NetworkError(req_err) => {
                    tracing::error!(symbol = %symbol, error = %req_err, "Network error");
                }
                HyperliquidError::JsonError(json_err) => {
                    tracing::error!(symbol = %symbol, error = %json_err, "JSON parsing error");
                }
                _ => {
                    tracing::error!(symbol = %symbol, error = %error, "Hyperliquid operation failed");
                }
            }
            error
        })
    }

    fn with_order_context(self, symbol: &str, side: &str) -> Result<T, HyperliquidError> {
        self.map_err(|e| {
            let error = e.into();
            tracing::error!(symbol = %symbol, side = %side, error = %error, "Order operation failed");
            error
        })
    }

    fn with_vault_context(self, vault_address: &str) -> Result<T, HyperliquidError> {
        self.map_err(|e| {
            let error = e.into();
            tracing::error!(vault = %vault_address, error = %error, "Vault operation failed");
            error
        })
    }
}

// Common types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub name: String,
    #[serde(rename = "szDecimals")]
    pub sz_decimals: u32,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: u32,
    #[serde(rename = "onlyIsolated")]
    pub only_isolated: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Universe {
    pub universe: Vec<AssetInfo>,
}

// Price and quantity types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size(pub String);

// Info endpoint request types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InfoRequest {
    #[serde(rename = "meta")]
    Meta,
    #[serde(rename = "universe")]
    Universe,
    #[serde(rename = "allMids")]
    AllMids,
    #[serde(rename = "userState")]
    UserState { user: String },
    #[serde(rename = "openOrders")]
    OpenOrders { user: String },
    #[serde(rename = "userFills")]
    UserFills { user: String },
    #[serde(rename = "l2Book")]
    L2Book { coin: String },
    #[serde(rename = "candleSnapshot")]
    CandleSnapshot {
        coin: String,
        interval: String,
        #[serde(rename = "startTime")]
        start_time: u64,
        #[serde(rename = "endTime")]
        end_time: u64,
    },
}

// Info endpoint response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllMids(pub HashMap<String, String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    #[serde(rename = "assetPositions")]
    pub asset_positions: Vec<AssetPosition>,
    #[serde(rename = "crossMaintenanceMarginUsed")]
    pub cross_maintenance_margin_used: String,
    #[serde(rename = "crossMarginUsed")]
    pub cross_margin_used: String,
    #[serde(rename = "marginSummary")]
    pub margin_summary: MarginSummary,
    #[serde(rename = "withdrawable")]
    pub withdrawable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPosition {
    pub position: Position,
    #[serde(rename = "type")]
    pub position_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub coin: String,
    #[serde(rename = "entryPx")]
    pub entry_px: Option<String>,
    #[serde(rename = "leverage")]
    pub leverage: Leverage,
    #[serde(rename = "liquidationPx")]
    pub liquidation_px: Option<String>,
    #[serde(rename = "marginUsed")]
    pub margin_used: String,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: u32,
    #[serde(rename = "positionValue")]
    pub position_value: String,
    #[serde(rename = "returnOnEquity")]
    pub return_on_equity: String,
    pub szi: String,
    #[serde(rename = "unrealizedPnl")]
    pub unrealized_pnl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leverage {
    #[serde(rename = "type")]
    pub leverage_type: String,
    pub value: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginSummary {
    #[serde(rename = "accountValue")]
    pub account_value: String,
    #[serde(rename = "totalMarginUsed")]
    pub total_margin_used: String,
    #[serde(rename = "totalNtlPos")]
    pub total_ntl_pos: String,
    #[serde(rename = "totalRawUsd")]
    pub total_raw_usd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenOrder {
    pub coin: String,
    #[serde(rename = "limitPx")]
    pub limit_px: String,
    pub oid: u64,
    pub side: String,
    pub sz: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFill {
    pub coin: String,
    pub px: String,
    pub sz: String,
    pub side: String,
    pub time: u64,
    #[serde(rename = "startPosition")]
    pub start_position: String,
    pub dir: String,
    #[serde(rename = "closedPnl")]
    pub closed_pnl: String,
    pub hash: String,
    pub oid: u64,
    pub crossed: bool,
    pub fee: String,
    pub tid: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Book {
    pub coin: String,
    pub levels: Vec<[Vec<L2Level>; 2]>, // [bids, asks]
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Level {
    pub px: String,
    pub sz: String,
    pub n: u32, // number of orders
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    #[serde(rename = "T")]
    pub time: u64,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "n")]
    pub num_trades: u32,
}

// Exchange endpoint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub coin: String,
    #[serde(rename = "is_buy")]
    pub is_buy: bool,
    pub sz: String,
    #[serde(rename = "limit_px")]
    pub limit_px: String,
    #[serde(rename = "order_type")]
    pub order_type: OrderType,
    #[serde(rename = "reduce_only")]
    pub reduce_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrderType {
    Limit { limit: LimitOrder },
    Trigger { trigger: TriggerOrder },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitOrder {
    pub tif: TimeInForce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerOrder {
    #[serde(rename = "triggerPx")]
    pub trigger_px: String,
    #[serde(rename = "isMarket")]
    pub is_market: bool,
    pub tpsl: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    #[serde(rename = "Gtc")]
    Gtc,
    #[serde(rename = "Ioc")]
    Ioc,
    #[serde(rename = "Alo")]
    Alo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelRequest {
    pub coin: String,
    pub oid: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifyRequest {
    pub oid: u64,
    pub order: OrderRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub status: String,
    pub response: OrderResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponseData {
    #[serde(rename = "type")]
    pub response_type: String,
    pub data: Option<OrderData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderData {
    pub statuses: Vec<OrderStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    pub resting: Option<RestingOrder>,
    pub filled: Option<FilledOrder>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestingOrder {
    pub oid: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilledOrder {
    #[serde(rename = "avgPx")]
    pub avg_px: String,
    pub oid: u64,
    #[serde(rename = "totalSz")]
    pub total_sz: String,
}

// Authentication and signing types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAction {
    pub action: serde_json::Value,
    pub nonce: u64,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRequest {
    pub action: SignedAction,
    #[serde(rename = "vaultAddress", skip_serializing_if = "Option::is_none")]
    pub vault_address: Option<String>,
}
