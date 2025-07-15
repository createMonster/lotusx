use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Simple typed errors for the types subsystem
#[derive(Error, Debug)]
pub enum TypesError {
    #[error("Invalid symbol: {0}")]
    InvalidSymbol(String),
    #[error("Invalid price: {0}")]
    InvalidPrice(String),
    #[error("Invalid quantity: {0}")]
    InvalidQuantity(String),
    #[error("Invalid volume: {0}")]
    InvalidVolume(String),
    #[error("Parsing error: {0}")]
    ParseError(String),
}

/// Type-safe symbol representation - simplified
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Symbol {
    pub base: String,
    pub quote: String,
}

impl Symbol {
    /// Create a new symbol from base and quote assets
    pub fn new(base: impl Into<String>, quote: impl Into<String>) -> Result<Self, TypesError> {
        let base = base.into();
        let quote = quote.into();

        if base.is_empty() || quote.is_empty() {
            return Err(TypesError::InvalidSymbol(
                "Base and quote cannot be empty".to_string(),
            ));
        }

        Ok(Self { base, quote })
    }

    /// Create symbol from string like "BTCUSDT"
    pub fn from_string(s: &str) -> Result<Self, TypesError> {
        // Simple pattern matching for common quote currencies
        if let Some(base) = s.strip_suffix("USDT") {
            return Self::new(base, "USDT");
        }
        if let Some(base) = s.strip_suffix("USDC") {
            return Self::new(base, "USDC");
        }
        if let Some(base) = s.strip_suffix("BTC") {
            return Self::new(base, "BTC");
        }
        if let Some(base) = s.strip_suffix("ETH") {
            return Self::new(base, "ETH");
        }
        if let Some(base) = s.strip_suffix("USD") {
            return Self::new(base, "USD");
        }

        Err(TypesError::InvalidSymbol(format!(
            "Cannot parse symbol: {}",
            s
        )))
    }

    /// Get as string reference
    pub fn as_str(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.base, self.quote)
    }
}

impl FromStr for Symbol {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_string(s)
    }
}

/// Type-safe price representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Price(Decimal);

impl Price {
    pub const fn new(value: Decimal) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> Decimal {
        self.0
    }

    pub const ZERO: Self = Self(Decimal::ZERO);

    pub fn from_f64(value: f64) -> Self {
        Self(Decimal::from_f64_retain(value).unwrap_or_default())
    }
}

impl FromStr for Price {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decimal = s
            .parse::<Decimal>()
            .map_err(|e| TypesError::InvalidPrice(e.to_string()))?;
        Ok(Self(decimal))
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe quantity representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Quantity(Decimal);

impl Quantity {
    pub const fn new(value: Decimal) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> Decimal {
        self.0
    }

    pub const ZERO: Self = Self(Decimal::ZERO);

    pub fn from_f64(value: f64) -> Self {
        Self(Decimal::from_f64_retain(value).unwrap_or_default())
    }
}

impl FromStr for Quantity {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decimal = s
            .parse::<Decimal>()
            .map_err(|e| TypesError::InvalidQuantity(e.to_string()))?;
        Ok(Self(decimal))
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe volume representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Volume(Decimal);

impl Volume {
    pub const fn new(value: Decimal) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> Decimal {
        self.0
    }

    pub const ZERO: Self = Self(Decimal::ZERO);

    pub fn from_f64(value: f64) -> Self {
        Self(Decimal::from_f64_retain(value).unwrap_or_default())
    }
}

impl FromStr for Volume {
    type Err = TypesError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decimal = s
            .parse::<Decimal>()
            .map_err(|e| TypesError::InvalidVolume(e.to_string()))?;
        Ok(Self(decimal))
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Simple conversion helpers
pub mod conversion {
    use super::{Decimal, FromStr, Price, Quantity, Symbol, Volume};

    pub fn string_to_symbol(s: &str) -> Symbol {
        Symbol::from_string(s).unwrap_or_else(|_| Symbol::new(s, "USD").unwrap_or_default())
    }

    pub fn string_to_price(s: &str) -> Price {
        Price::from_str(s).unwrap_or(Price::ZERO)
    }

    pub fn string_to_quantity(s: &str) -> Quantity {
        Quantity::from_str(s).unwrap_or(Quantity::ZERO)
    }

    pub fn string_to_volume(s: &str) -> Volume {
        Volume::from_str(s).unwrap_or(Volume::ZERO)
    }

    pub fn string_to_decimal(s: &str) -> Decimal {
        s.parse().unwrap_or(Decimal::ZERO)
    }
}

// Core data structures
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

impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GTC => write!(f, "GTC"),
            Self::IOC => write!(f, "IOC"),
            Self::FOK => write!(f, "FOK"),
        }
    }
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

/// Kline interval enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KlineInterval {
    Minutes1,
    Minutes3,
    Minutes5,
    Minutes15,
    Minutes30,
    Hours1,
    Hours2,
    Hours4,
    Hours6,
    Hours8,
    Hours12,
    Days1,
    Days3,
    Weeks1,
    Months1,
}

impl KlineInterval {
    pub fn to_binance_format(&self) -> String {
        match self {
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

    pub fn to_bybit_format(&self) -> String {
        match self {
            Self::Minutes1 => "1".to_string(),
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
            Self::Days3 => "3D".to_string(),
            Self::Weeks1 => "W".to_string(),
            Self::Months1 => "M".to_string(),
        }
    }
}

impl fmt::Display for KlineInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
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
    pub ping_interval: Option<u64>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRate {
    pub symbol: Symbol,
    pub funding_rate: Option<Decimal>,
    pub previous_funding_rate: Option<Decimal>,
    pub next_funding_rate: Option<Decimal>,
    pub funding_time: Option<i64>,
    pub next_funding_time: Option<i64>,
    pub mark_price: Option<Price>,
    pub index_price: Option<Price>,
    pub timestamp: i64,
}
