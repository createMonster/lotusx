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
    Klines { interval: String },
}

#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub auto_reconnect: bool,
    pub ping_interval: Option<u64>, // seconds
    pub max_reconnect_attempts: Option<u32>,
}
