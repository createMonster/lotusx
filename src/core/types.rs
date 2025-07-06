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
    /// Create a new symbol with validation
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Result<Self, TypesError> {
        let base = base.into();
        let quote = quote.into();

        if base.is_empty() || quote.is_empty() {
            return Err(TypesError::InvalidSymbol(
                "Base and quote assets cannot be empty".to_string(),
            ));
        }

        Ok(Symbol { base, quote })
    }

    /// Create from symbol string like "BTCUSDT"
    pub fn from_string(symbol: &str) -> Result<Self, TypesError> {
        // This is a simplified parser - in practice, you'd need exchange-specific parsing
        if symbol.len() < 6 {
            return Err(TypesError::InvalidSymbol("Symbol too short".to_string()));
        }

        // Common patterns for symbol separation
        if symbol.ends_with("USDT") {
            let base = symbol.strip_suffix("USDT").unwrap();
            Ok(Symbol::new(base, "USDT")?)
        } else if symbol.ends_with("BTC") {
            let base = symbol.strip_suffix("BTC").unwrap();
            Ok(Symbol::new(base, "BTC")?)
        } else if symbol.ends_with("ETH") {
            let base = symbol.strip_suffix("ETH").unwrap();
            Ok(Symbol::new(base, "ETH")?)
        } else if symbol.ends_with("USD") {
            let base = symbol.strip_suffix("USD").unwrap();
            Ok(Symbol::new(base, "USD")?)
        } else {
            Err(TypesError::InvalidSymbol(
                "Unable to parse symbol".to_string(),
            ))
        }
    }

    /// Get the symbol string (base + quote)
    pub fn to_string(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }

    /// Get as string reference for method calls expecting &str
    pub fn as_str(&self) -> String {
        self.to_string()
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)
    }
}

/// Type-safe price representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Price(#[serde(with = "rust_decimal::serde::str")] pub Decimal);

impl Price {
    pub fn new(value: Decimal) -> Self {
        Price(value)
    }

    pub fn from_str(s: &str) -> Result<Self, TypesError> {
        Ok(Price(s.parse()?))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Price {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// Type-safe quantity representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Quantity(#[serde(with = "rust_decimal::serde::str")] pub Decimal);

impl Quantity {
    pub fn new(value: Decimal) -> Self {
        Quantity(value)
    }

    pub fn from_str(s: &str) -> Result<Self, TypesError> {
        Ok(Quantity(s.parse()?))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Quantity {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// Type-safe volume representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Volume(#[serde(with = "rust_decimal::serde::str")] pub Decimal);

impl Volume {
    pub fn new(value: Decimal) -> Self {
        Volume(value)
    }

    pub fn from_str(s: &str) -> Result<Self, TypesError> {
        Ok(Volume(s.parse()?))
    }

    pub fn value(&self) -> Decimal {
        self.0
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Volume {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

/// HFT-compliant conversion helpers for safe type conversions
pub mod conversion {
    use super::*;

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
