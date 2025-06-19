use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BinancePerpError {
    #[error("Authentication failed: {message}, symbol={symbol:?}")]
    AuthError {
        message: String,
        symbol: Option<String>,
    },

    #[error("Order placement failed: code={code}, message={message}, symbol={symbol}")]
    OrderError {
        code: i32,
        message: String,
        symbol: String,
    },

    #[error("Market data fetch failed: {message}, symbol={symbol:?}")]
    MarketDataError {
        message: String,
        symbol: Option<String>,
    },

    #[error("Account data fetch failed: {message}")]
    AccountError { message: String },

    #[error("Rate limit exceeded: retry_after={retry_after_ms}ms")]
    RateLimitError { retry_after_ms: u64 },

    #[error("Network connectivity issue: {message}")]
    NetworkError { message: String },

    #[error("JSON parsing failed: {message}, context={context:?}")]
    ParseError {
        message: String,
        context: Option<String>,
    },

    #[error("Invalid parameters: {message}, symbol={symbol:?}")]
    ValidationError {
        message: String,
        symbol: Option<String>,
    },
}

impl BinancePerpError {
    #[cold]
    #[inline(never)]
    pub fn auth_error(message: impl Into<String>, symbol: Option<String>) -> Self {
        Self::AuthError {
            message: message.into(),
            symbol,
        }
    }

    #[cold]
    #[inline(never)]
    pub fn order_error(code: i32, message: impl Into<String>, symbol: impl Into<String>) -> Self {
        Self::OrderError {
            code,
            message: message.into(),
            symbol: symbol.into(),
        }
    }

    #[cold]
    #[inline(never)]
    pub fn market_data_error(message: impl Into<String>, symbol: Option<String>) -> Self {
        Self::MarketDataError {
            message: message.into(),
            symbol,
        }
    }

    #[cold]
    #[inline(never)]
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    #[cold]
    #[inline(never)]
    pub fn parse_error(message: impl Into<String>, context: Option<String>) -> Self {
        Self::ParseError {
            message: message.into(),
            context,
        }
    }

    #[cold]
    #[inline(never)]
    pub fn account_error(message: impl Into<String>) -> Self {
        Self::AccountError {
            message: message.into(),
        }
    }
}

impl From<BinancePerpError> for crate::core::errors::ExchangeError {
    fn from(err: BinancePerpError) -> Self {
        match err {
            BinancePerpError::AuthError { message, .. } => Self::AuthError(message),
            BinancePerpError::OrderError { code, message, .. } => Self::ApiError { code, message },
            BinancePerpError::NetworkError { message } => Self::NetworkError(message),
            BinancePerpError::ParseError { message, .. } => {
                Self::Other(format!("Parse error: {}", message))
            }
            _ => Self::Other(err.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpMarket {
    pub symbol: String,
    #[serde(rename = "baseAsset")]
    pub base_asset: String,
    #[serde(rename = "quoteAsset")]
    pub quote_asset: String,
    pub status: String,
    #[serde(rename = "baseAssetPrecision")]
    pub base_asset_precision: i32,
    #[serde(rename = "quotePrecision")]
    pub quote_precision: i32,
    pub filters: Vec<BinancePerpFilter>,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpFilter {
    #[serde(rename = "filterType")]
    pub filter_type: String,
    #[serde(rename = "minPrice")]
    pub min_price: Option<String>,
    #[serde(rename = "maxPrice")]
    pub max_price: Option<String>,
    #[serde(rename = "minQty")]
    pub min_qty: Option<String>,
    #[serde(rename = "maxQty")]
    pub max_qty: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpExchangeInfo {
    pub symbols: Vec<BinancePerpMarket>,
}

#[derive(Debug, Serialize)]
pub struct BinancePerpOrderRequest {
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub quantity: String,
    pub price: Option<String>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<String>,
    #[serde(rename = "stopPrice")]
    pub stop_price: Option<String>,
    pub timestamp: u64,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpOrderResponse {
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "origClientOrderId")]
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "origQty")]
    pub orig_qty: String,
    pub price: String,
    pub status: String,
    #[serde(rename = "updateTime")]
    pub update_time: i64,
}

// WebSocket Types for Perpetual Futures
#[derive(Debug, Deserialize)]
pub struct BinancePerpWebSocketTicker {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub price: String,
    #[serde(rename = "P")]
    pub price_change_percent: String,
    #[serde(rename = "p")]
    pub price_change: String,
    #[serde(rename = "h")]
    pub high_price: String,
    #[serde(rename = "l")]
    pub low_price: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "q")]
    pub quote_volume: String,
    #[serde(rename = "O")]
    pub open_time: i64,
    #[serde(rename = "C")]
    pub close_time: i64,
    #[serde(rename = "n")]
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpWebSocketOrderBook {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "U")]
    pub first_update_id: i64,
    #[serde(rename = "u")]
    pub final_update_id: i64,
    #[serde(rename = "pu")]
    pub prev_final_update_id: i64,
    #[serde(rename = "b")]
    pub bids: Vec<[String; 2]>,
    #[serde(rename = "a")]
    pub asks: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpWebSocketTrade {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "t")]
    pub id: i64,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub quantity: String,
    #[serde(rename = "T")]
    pub time: i64,
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpWebSocketKline {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: BinancePerpKlineData,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpKlineData {
    #[serde(rename = "t")]
    pub open_time: i64,
    #[serde(rename = "T")]
    pub close_time: i64,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "o")]
    pub open_price: String,
    #[serde(rename = "h")]
    pub high_price: String,
    #[serde(rename = "l")]
    pub low_price: String,
    #[serde(rename = "c")]
    pub close_price: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "n")]
    pub number_of_trades: i64,
    #[serde(rename = "x")]
    pub final_bar: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinancePerpBalance {
    pub asset: String,
    pub balance: String,
    #[serde(rename = "availableBalance")]
    pub available_balance: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinancePerpPosition {
    pub symbol: String,
    #[serde(rename = "positionAmt")]
    pub position_amt: String,
    #[serde(rename = "entryPrice")]
    pub entry_price: String,
    #[serde(rename = "unRealizedPnl")]
    pub un_realized_pnl: String,
    #[serde(rename = "liquidationPrice")]
    pub liquidation_price: String,
    pub leverage: String,
}

// REST API K-line Types
#[derive(Debug, Deserialize)]
pub struct BinancePerpRestKline {
    #[serde(rename = "0")]
    pub open_time: i64,
    #[serde(rename = "1")]
    pub open_price: String,
    #[serde(rename = "2")]
    pub high_price: String,
    #[serde(rename = "3")]
    pub low_price: String,
    #[serde(rename = "4")]
    pub close_price: String,
    #[serde(rename = "5")]
    pub volume: String,
    #[serde(rename = "6")]
    pub close_time: i64,
    #[serde(rename = "7")]
    pub quote_asset_volume: String,
    #[serde(rename = "8")]
    pub number_of_trades: i64,
    #[serde(rename = "9")]
    pub taker_buy_base_asset_volume: String,
    #[serde(rename = "10")]
    pub taker_buy_quote_asset_volume: String,
    #[serde(rename = "11")]
    pub ignore: String,
}
