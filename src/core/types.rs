use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// HFT-compliant typed errors for the types subsystem
#[derive(Error, Debug)]
pub enum TypesError {
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    #[error("Invalid price: {0}")]
    InvalidPrice(#[from] rust_decimal::Error),
    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),
    #[error("Invalid volume: {0}")]
    InvalidVolume(String),
    #[error("Parsing error: {0}")]
    ParseError(String),
}

/// Type-safe symbol representation with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub base: String,
    pub quote: String,
}

impl Symbol {
    /// Create a new symbol from base and quote assets
    pub fn new(base: String, quote: String) -> Result<Self, TypesError> {
        if base.is_empty() || quote.is_empty() {
            return Err(TypesError::InvalidSymbol(
                "Base and quote cannot be empty".to_string(),
            ));
        }
        Ok(Self { base, quote })
    }

    /// Create symbol from string like "BTCUSDT"  
    pub fn from_string(s: &str) -> Result<Self, TypesError> {
        let base = s
            .replace("USDT", "")
            .replace("BTC", "")
            .replace("ETH", "")
            .replace("USD", "");
        match s {
            s if s.ends_with("USDT") => Ok(Self::new(base, "USDT".to_string())?),
            s if s.ends_with("BTC") => Ok(Self::new(base, "BTC".to_string())?),
            s if s.ends_with("ETH") => Ok(Self::new(base, "ETH".to_string())?),
            s if s.ends_with("USD") => Ok(Self::new(base, "USD".to_string())?),
            _ => Err(TypesError::InvalidSymbol(format!(
                "Cannot parse symbol: {}",
                s
            ))),
        }
    }

    /// Get as string reference for method calls expecting &str
    pub fn as_str(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.base, self.quote)
    }
}

/// Type-safe price representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Price(Decimal);

impl Price {
    /// Create a new price
    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

    /// Get the decimal value
    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl std::str::FromStr for Price {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe quantity representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Quantity(Decimal);

impl Quantity {
    /// Create a new quantity
    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

    /// Get the decimal value
    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl std::str::FromStr for Quantity {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe volume representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Volume(Decimal);

impl Volume {
    /// Create a new volume
    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

    /// Get the decimal value
    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl std::str::FromStr for Volume {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// HFT-compliant conversion helpers for safe type conversions
pub mod conversion {
    use super::{Decimal, Price, Quantity, Symbol, Volume};
    use std::str::FromStr;

    /// Convert string to Symbol with fallback
    #[inline]
    pub fn string_to_symbol(s: &str) -> Symbol {
        Symbol::from_string(s).unwrap_or_else(|_| {
            // Fallback: treat as base asset with USD quote
            Symbol {
                base: s.to_string(),
                quote: "USD".to_string(),
            }
        })
    }

    /// Convert string to Price with fallback
    #[inline]
    pub fn string_to_price(s: &str) -> Price {
        Price::from_str(s).unwrap_or_else(|_| Price::new(Decimal::from(0)))
    }

    /// Convert string to Quantity with fallback
    #[inline]
    pub fn string_to_quantity(s: &str) -> Quantity {
        Quantity::from_str(s).unwrap_or_else(|_| Quantity::new(Decimal::from(0)))
    }

    /// Convert string to Volume with fallback
    #[inline]
    pub fn string_to_volume(s: &str) -> Volume {
        Volume::from_str(s).unwrap_or_else(|_| Volume::new(Decimal::from(0)))
    }

    /// Convert string to Decimal with fallback
    #[inline]
    pub fn string_to_decimal(s: &str) -> Decimal {
        s.parse().unwrap_or_else(|_| Decimal::from(0))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    pub symbol: Symbol,
    pub status: String,
    pub base_precision: i32,
    pub quote_precision: i32,
    pub min_qty: Option<Quantity>,
    pub max_qty: Option<Quantity>,
    pub min_price: Option<Price>,
    pub max_price: Option<Price>,
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
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub price: Option<Price>,
    pub time_in_force: Option<TimeInForce>,
    pub stop_price: Option<Price>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: Symbol,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Quantity,
    pub price: Option<Price>,
    pub status: String,
    pub timestamp: i64,
}

// WebSocket Market Data Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: Symbol,
    pub price: Price,
    pub price_change: Price,
    pub price_change_percent: Decimal,
    pub high_price: Price,
    pub low_price: Price,
    pub volume: Volume,
    pub quote_volume: Volume,
    pub open_time: i64,
    pub close_time: i64,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub price: Price,
    pub quantity: Quantity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: Symbol,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub last_update_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub symbol: Symbol,
    pub id: i64,
    pub price: Price,
    pub quantity: Quantity,
    pub time: i64,
    pub is_buyer_maker: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub symbol: Symbol,
    pub open_time: i64,
    pub close_time: i64,
    pub interval: String,
    pub open_price: Price,
    pub high_price: Price,
    pub low_price: Price,
    pub close_price: Price,
    pub volume: Volume,
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
    pub free: Quantity,
    pub locked: Quantity,
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
    pub symbol: Symbol,
    pub position_side: PositionSide,
    pub entry_price: Price,
    pub position_amount: Quantity,
    pub unrealized_pnl: Decimal,
    pub liquidation_price: Option<Price>,
    pub leverage: Decimal,
}

/// Funding rate information for perpetual futures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRate {
    pub symbol: Symbol,
    pub funding_rate: Option<Decimal>, // Current/upcoming funding rate
    pub previous_funding_rate: Option<Decimal>, // Most recently applied rate
    pub next_funding_rate: Option<Decimal>, // Predicted next rate (if available)
    pub funding_time: Option<i64>,     // When current rate applies
    pub next_funding_time: Option<i64>, // When next rate applies
    pub mark_price: Option<Price>,     // Current mark price
    pub index_price: Option<Price>,    // Current index price
    pub timestamp: i64,                // Response timestamp
}

/// Funding rate interval for historical queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FundingRateInterval {
    Hours8,  // Every 8 hours (most common)
    Hours1,  // Every hour (some exchanges)
    Hours4,  // Every 4 hours
    Hours12, // Every 12 hours
}

impl FundingRateInterval {
    pub fn to_seconds(&self) -> i64 {
        match self {
            Self::Hours1 => 3600,
            Self::Hours4 => 14400,
            Self::Hours8 => 28800,
            Self::Hours12 => 43200,
        }
    }
}
