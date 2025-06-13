use serde::{Deserialize, Serialize};

// REST API Response Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackExchangeInfo {
    pub status: String,
    pub time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackMarket {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    pub status: String,
    pub base_precision: i32,
    pub quote_precision: i32,
    pub min_qty: String,
    pub max_qty: String,
    pub min_price: String,
    pub max_price: String,
    pub tick_size: String,
    pub step_size: String,
    pub min_notional: String,
    pub max_notional: String,
    pub funding_rate_lower_bound: Option<String>,
    pub funding_rate_upper_bound: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackTicker {
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
pub struct BackpackOrderBookEntry {
    pub price: String,
    pub quantity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackOrderBook {
    pub symbol: String,
    pub bids: Vec<BackpackOrderBookEntry>,
    pub asks: Vec<BackpackOrderBookEntry>,
    pub last_update_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackRestKline {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
    pub quote_volume: String,
    pub number_of_trades: i64,
    pub taker_buy_base_volume: String,
    pub taker_buy_quote_volume: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackTrade {
    pub id: i64,
    pub price: String,
    pub quantity: String,
    pub quote_quantity: String,
    pub time: i64,
    pub is_buyer_maker: bool,
    pub is_best_match: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackMarkPrice {
    pub symbol: String,
    pub mark_price: String,
    pub index_price: String,
    pub estimated_funding_rate: String,
    pub next_funding_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackOpenInterest {
    pub symbol: String,
    pub open_interest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackFundingRate {
    pub symbol: String,
    pub funding_rate: String,
    pub funding_time: i64,
    pub next_funding_time: i64,
}

// Account Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackPosition {
    pub symbol: String,
    pub side: String,
    pub size: String,
    pub entry_price: String,
    pub mark_price: String,
    pub unrealized_pnl: String,
    pub liquidation_price: String,
    pub leverage: String,
    pub margin_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackOrder {
    pub order_id: i64,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: String,
    pub price: Option<String>,
    pub stop_price: Option<String>,
    pub time_in_force: String,
    pub status: String,
    pub executed_qty: String,
    pub cummulative_quote_qty: String,
    pub avg_price: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub working_type: Option<String>,
    pub price_protect: Option<bool>,
    pub close_position: Option<bool>,
    pub activation_price: Option<String>,
    pub callback_rate: Option<String>,
    pub realized_pnl: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackFill {
    pub symbol: String,
    pub order_id: i64,
    pub price: String,
    pub quantity: String,
    pub quote_quantity: String,
    pub commission: String,
    pub commission_asset: String,
    pub time: i64,
    pub is_buyer: bool,
    pub is_maker: bool,
    pub is_best_match: Option<bool>,
}

// Request Types

#[derive(Debug, Clone, Serialize)]
pub struct BackpackOrderRequest {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: String,
    pub price: Option<String>,
    pub time_in_force: Option<String>,
    pub client_order_id: Option<String>,
    pub stop_price: Option<String>,
    pub working_type: Option<String>,
    pub price_protect: Option<bool>,
    pub close_position: Option<bool>,
    pub activation_price: Option<String>,
    pub callback_rate: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackpackCancelOrderRequest {
    pub symbol: String,
    pub order_id: Option<i64>,
    pub client_order_id: Option<String>,
}

// WebSocket Types

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketTicker {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub s: String, // Symbol
    pub o: String, // Open price
    pub c: String, // Close price
    pub h: String, // High price
    pub l: String, // Low price
    pub v: String, // Volume
    pub V: String, // Quote volume
    pub n: i64,    // Number of trades
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketOrderBook {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub s: String, // Symbol
    pub a: Vec<Vec<String>>, // Asks [price, quantity]
    pub b: Vec<Vec<String>>, // Bids [price, quantity]
    pub U: i64,    // First update ID
    pub u: i64,    // Last update ID
    pub T: i64,    // Engine timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketTrade {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub s: String, // Symbol
    pub p: String, // Price
    pub q: String, // Quantity
    pub b: String, // Buyer order ID
    pub a: String, // Seller order ID
    pub t: i64,    // Trade ID
    pub T: i64,    // Engine timestamp
    pub m: bool,   // Is buyer maker
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketKline {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub s: String, // Symbol
    pub t: i64,    // K-line start time
    pub T: i64,    // K-line close time
    pub o: String, // Open price
    pub c: String, // Close price
    pub h: String, // High price
    pub l: String, // Low price
    pub v: String, // Volume
    pub n: i64,    // Number of trades
    pub X: bool,   // Is k-line closed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketMarkPrice {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub s: String, // Symbol
    pub p: String, // Mark price
    pub f: String, // Estimated funding rate
    pub i: String, // Index price
    pub n: i64,    // Next funding timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketOpenInterest {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub s: String, // Symbol
    pub o: String, // Open interest
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketLiquidation {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub q: String, // Quantity
    pub p: String, // Price
    pub S: String, // Side
    pub s: String, // Symbol
    pub T: i64,    // Engine timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketBookTicker {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub s: String, // Symbol
    pub a: String, // Inside ask price
    pub A: String, // Inside ask quantity
    pub b: String, // Inside bid price
    pub B: String, // Inside bid quantity
    pub u: String, // Update ID
    pub T: i64,    // Engine timestamp
}

// RFQ Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketRFQ {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub R: i64,    // RFQ ID
    pub C: String, // Client ID
    pub s: String, // Symbol
    pub S: String, // Side
    pub q: Option<String>, // Quantity (base asset)
    pub Q: Option<String>, // Quote quantity (quote asset)
    pub w: i64,    // Submission time
    pub W: i64,    // Expiry time
    pub X: String, // Status
    pub T: i64,    // Engine timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct BackpackWebSocketRFQUpdate {
    pub e: String, // Event type
    pub E: i64,    // Event time
    pub R: i64,    // RFQ ID
    pub u: Option<i64>, // Quote ID
    pub C: String, // Client ID
    pub s: String, // Symbol
    pub S: Option<String>, // Side
    pub q: Option<String>, // Quantity
    pub Q: Option<String>, // Quote quantity
    pub p: Option<String>, // Price
    pub w: Option<i64>,    // Submission time
    pub W: Option<i64>,    // Expiry time
    pub X: String, // Status
    pub T: i64,    // Engine timestamp
}

// Response Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackOrderResponse {
    pub order_id: i64,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: String,
    pub price: Option<String>,
    pub status: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackKlineData {
    pub symbol: String,
    pub interval: String,
    pub klines: Vec<BackpackRestKline>,
}

// Error Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackError {
    pub code: i32,
    pub msg: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<BackpackError>,
}

// WebSocket Message Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BackpackWebSocketMessage {
    Ticker(BackpackWebSocketTicker),
    OrderBook(BackpackWebSocketOrderBook),
    Trade(BackpackWebSocketTrade),
    Kline(BackpackWebSocketKline),
    MarkPrice(BackpackWebSocketMarkPrice),
    OpenInterest(BackpackWebSocketOpenInterest),
    Liquidation(BackpackWebSocketLiquidation),
    BookTicker(BackpackWebSocketBookTicker),
    RFQ(BackpackWebSocketRFQ),
    RFQUpdate(BackpackWebSocketRFQUpdate),
    Ping { ping: i64 },
    Pong { pong: i64 },
}

// WebSocket Subscription Request
#[derive(Debug, Clone, Serialize)]
pub struct BackpackWebSocketSubscription {
    pub method: String,
    pub params: Vec<String>,
    pub id: i64,
}

// WebSocket Subscription Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackWebSocketSubscriptionResponse {
    pub result: Option<Vec<String>>,
    pub id: i64,
}

// Account Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackAccountInfo {
    pub account_type: String,
    pub balances: Vec<BackpackBalance>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackpackTradingFee {
    pub symbol: String,
    pub maker_fee: String,
    pub taker_fee: String,
} 