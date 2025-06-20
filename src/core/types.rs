use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub base: String,
    pub quote: String,
    pub symbol: String,
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub symbol: Symbol,
    pub status: String,
    pub base_precision: i32,
    pub quote_precision: i32,
    pub min_qty: Option<String>,
    pub max_qty: Option<String>,
    pub min_price: Option<String>,
    pub max_price: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC, // Good Till Canceled
    IOC, // Immediate or Cancel
    FOK, // Fill or Kill
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: String,
    pub price: Option<String>,
    pub time_in_force: Option<TimeInForce>,
    pub stop_price: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: String,
    pub price: Option<String>,
    pub status: String,
    pub timestamp: i64,
}

// WebSocket Market Data Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub price: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
    pub open_time: i64,
    pub close_time: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub price: String,
    pub quantity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub last_update_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: String,
    pub id: i64,
    pub price: String,
    pub quantity: String,
    pub time: i64,
    pub is_buyer_maker: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub symbol: String,
    pub open_time: i64,
    pub close_time: i64,
    pub interval: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub close_price: String,
    pub volume: String,
    pub number_of_trades: i64,
    pub final_bar: bool,
}

/// Unified kline interval enum supporting all major exchanges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KlineInterval {
    // Seconds (supported by some exchanges)
    Seconds1,

    // Minutes
    Minutes1,
    Minutes3,
    Minutes5,
    Minutes15,
    Minutes30,

    // Hours
    Hours1,
    Hours2,
    Hours4,
    Hours6,
    Hours8,
    Hours12,

    // Days
    Days1,
    Days3,

    // Weeks
    Weeks1,

    // Months
    Months1,
}

impl KlineInterval {
    /// Convert to Binance format (e.g., "1m", "1h", "1d")
    pub fn to_binance_format(&self) -> String {
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

    /// Convert to Bybit format (e.g., "1", "60", "D")
    pub fn to_bybit_format(&self) -> String {
        match self {
            Self::Seconds1 | Self::Minutes1 => "1".to_string(), // Seconds not typically supported, Minutes1 is "1"
            Self::Minutes3 => "3".to_string(),
            Self::Minutes5 => "5".to_string(),
            Self::Minutes15 => "15".to_string(),
            Self::Minutes30 => "30".to_string(),
            Self::Hours1 => "60".to_string(),
            Self::Hours2 => "120".to_string(),
            Self::Hours4 => "240".to_string(),
            Self::Hours6 => "360".to_string(),
            Self::Hours8 => "480".to_string(),
            Self::Hours12 => "720".to_string(),
            Self::Days1 => "D".to_string(),
            Self::Days3 => "3D".to_string(), // May not be supported
            Self::Weeks1 => "W".to_string(),
            Self::Months1 => "M".to_string(),
        }
    }

    /// Convert to Backpack format (similar to Binance)
    pub fn to_backpack_format(&self) -> String {
        // Backpack typically uses similar format to Binance
        self.to_binance_format()
    }

    /// Convert to Hyperliquid format (if they support klines in future)
    pub fn to_hyperliquid_format(&self) -> String {
        // Hyperliquid currently doesn't support klines, but keeping for future
        self.to_binance_format()
    }

    /// Get all supported intervals
    pub fn all() -> Vec<Self> {
        vec![
            Self::Seconds1,
            Self::Minutes1,
            Self::Minutes3,
            Self::Minutes5,
            Self::Minutes15,
            Self::Minutes30,
            Self::Hours1,
            Self::Hours2,
            Self::Hours4,
            Self::Hours6,
            Self::Hours8,
            Self::Hours12,
            Self::Days1,
            Self::Days3,
            Self::Weeks1,
            Self::Months1,
        ]
    }

    /// Check if interval is supported by a specific exchange
    pub fn is_supported_by_binance(&self) -> bool {
        // Binance supports all intervals in our enum
        true
    }

    pub fn is_supported_by_bybit(&self) -> bool {
        match self {
            Self::Seconds1 | Self::Days3 => false, // Bybit doesn't support seconds or 3-day interval
            _ => true,
        }
    }

    pub fn is_supported_by_backpack(&self) -> bool {
        // Most intervals are supported, but seconds might not be
        !matches!(self, Self::Seconds1)
    }
}

impl fmt::Display for KlineInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            Self::Seconds1 => "1 second",
            Self::Minutes1 => "1 minute",
            Self::Minutes3 => "3 minutes",
            Self::Minutes5 => "5 minutes",
            Self::Minutes15 => "15 minutes",
            Self::Minutes30 => "30 minutes",
            Self::Hours1 => "1 hour",
            Self::Hours2 => "2 hours",
            Self::Hours4 => "4 hours",
            Self::Hours6 => "6 hours",
            Self::Hours8 => "8 hours",
            Self::Hours12 => "12 hours",
            Self::Days1 => "1 day",
            Self::Days3 => "3 days",
            Self::Weeks1 => "1 week",
            Self::Months1 => "1 month",
        };
        write!(f, "{}", description)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketDataType {
    Ticker(Ticker),
    OrderBook(OrderBook),
    Trade(Trade),
    Kline(Kline),
}

#[derive(Debug, Clone)]
pub enum SubscriptionType {
    Ticker,
    OrderBook { depth: Option<u32> },
    Trades,
    Klines { interval: KlineInterval },
}

#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub auto_reconnect: bool,
    pub ping_interval: Option<u64>, // seconds
    pub max_reconnect_attempts: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PositionSide {
    Long,
    Short,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub position_side: PositionSide,
    pub entry_price: String,
    pub position_amount: String,
    pub unrealized_pnl: String,
    pub liquidation_price: Option<String>,
    pub leverage: String,
}
