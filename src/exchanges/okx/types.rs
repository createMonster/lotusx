use serde::{Deserialize, Serialize};

/// OKX API standard response wrapper
#[derive(Debug, Deserialize, Serialize)]
pub struct OkxResponse<T> {
    pub code: String,
    pub msg: String,
    pub data: T,
}

/// OKX Market (Instrument) information
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxMarket {
    pub inst_type: String,          // SPOT, MARGIN, SWAP, FUTURES, OPTION
    pub inst_id: String,            // e.g., BTC-USDT
    pub uly: Option<String>,        // Underlying (for derivatives)
    pub base_ccy: String,           // Base currency
    pub quote_ccy: String,          // Quote currency
    pub settle_ccy: Option<String>, // Settlement currency
    pub ct_val: Option<String>,     // Contract value
    pub ct_mult: Option<String>,    // Contract multiplier
    pub ct_val_ccy: Option<String>, // Contract value currency
    pub opt_type: Option<String>,   // Option type (C/P)
    pub stk: Option<String>,        // Strike price
    pub list_time: Option<String>,  // Listing time
    pub exp_time: Option<String>,   // Expiry time
    pub lever: Option<String>,      // Max leverage
    pub tick_sz: String,            // Tick size
    pub lot_sz: String,             // Lot size
    pub min_sz: String,             // Minimum order size
    pub ct_type: Option<String>,    // Contract type
    pub alias: Option<String>,      // Alias
    pub state: String,              // State: live, suspend, preopen, test
}

/// OKX Order request
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxOrderRequest {
    pub inst_id: String,  // Instrument ID
    pub td_mode: String,  // Trade mode: cash, cross, isolated
    pub side: String,     // Order side: buy, sell
    pub ord_type: String, // Order type: market, limit, post_only, fok, ioc
    pub sz: String,       // Quantity to buy or sell
    #[serde(skip_serializing_if = "Option::is_none")]
    pub px: Option<String>, // Order price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<String>, // Client order ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>, // Order tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tgt_ccy: Option<String>, // Target currency: base_ccy, quote_ccy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ban_amend: Option<bool>, // Disallow amend
}

/// OKX Order response
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxOrderResponse {
    pub ord_id: String,            // Order ID
    pub cl_ord_id: Option<String>, // Client order ID
    pub tag: Option<String>,       // Order tag
    pub s_code: String,            // Success code
    pub s_msg: String,             // Success message
}

/// OKX Order details
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxOrder {
    pub inst_type: String,         // Instrument type
    pub inst_id: String,           // Instrument ID
    pub ord_id: String,            // Order ID
    pub cl_ord_id: Option<String>, // Client order ID
    pub tag: Option<String>,       // Order tag
    pub px: String,                // Price
    pub sz: String,                // Size
    pub ord_type: String,          // Order type
    pub side: String,              // Order side
    pub pos_side: Option<String>,  // Position side
    pub td_mode: String,           // Trade mode
    pub acc_fill_sz: String,       // Accumulated fill size
    pub fill_px: String,           // Fill price
    pub trade_id: String,          // Trade ID
    pub fill_sz: String,           // Fill size
    pub fill_time: String,         // Fill time
    pub avg_px: String,            // Average price
    pub state: String,             // Order state
    pub lever: Option<String>,     // Leverage
    pub fee_ccy: String,           // Fee currency
    pub fee: String,               // Fee
    pub rebate_ccy: String,        // Rebate currency
    pub rebate: String,            // Rebate
    pub tgt_ccy: Option<String>,   // Target currency
    pub category: String,          // Category
    pub u_time: String,            // Update time
    pub c_time: String,            // Creation time
}

/// OKX Account balance
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxBalance {
    pub ccy: String,            // Currency
    pub eq: String,             // Equity
    pub cash_bal: String,       // Cash balance
    pub upl: String,            // Unrealized P&L
    pub avail_eq: String,       // Available equity
    pub dis_eq: String,         // Discounted equity
    pub avail_bal: String,      // Available balance
    pub frozen_bal: String,     // Frozen balance
    pub ord_frozen: String,     // Margin frozen for open orders
    pub liab: String,           // Liabilities
    pub upl_liab: String,       // Unrealized P&L of liabilities
    pub cross_liab: String,     // Cross liabilities
    pub iso_liab: String,       // Isolated liabilities
    pub mgn_ratio: String,      // Margin ratio
    pub interest: String,       // Interest
    pub twap: String,           // TWAP
    pub max_loan: String,       // Max loan
    pub eq_usd: String,         // Equity in USD
    pub notional_lever: String, // Notional leverage
    pub stgy_eq: String,        // Strategy equity
    pub iso_upl: String,        // Isolated unrealized P&L
}

/// OKX Account information
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxAccountInfo {
    pub uid: String,              // User ID
    pub acct_lv: String,          // Account level
    pub pos_mode: String,         // Position mode
    pub auto_loan: bool,          // Auto loan enabled
    pub greeks_type: String,      // Greeks type
    pub level: String,            // Level
    pub level_tmp: String,        // Temporary level
    pub mgn_iso: String,          // Isolated margin
    pub total_eq: String,         // Total equity
    pub iso_eq: String,           // Isolated equity
    pub adj_eq: String,           // Adjusted equity
    pub ord_froz: String,         // Order frozen
    pub imr: String,              // Initial margin requirement
    pub mmr: String,              // Maintenance margin requirement
    pub notional_usd: String,     // Notional in USD
    pub upl: String,              // Unrealized P&L
    pub details: Vec<OkxBalance>, // Balance details
}

/// OKX Ticker data
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxTicker {
    pub inst_type: String,   // Instrument type
    pub inst_id: String,     // Instrument ID
    pub last: String,        // Last traded price
    pub last_sz: String,     // Last traded size
    pub ask_px: String,      // Best ask price
    pub ask_sz: String,      // Best ask size
    pub bid_px: String,      // Best bid price
    pub bid_sz: String,      // Best bid size
    pub open_24h: String,    // 24h opening price
    pub high_24h: String,    // 24h highest price
    pub low_24h: String,     // 24h lowest price
    pub vol_ccy_24h: String, // 24h volume in quote currency
    pub vol_24h: String,     // 24h volume in base currency
    pub ts: String,          // Timestamp
    pub sod_utc0: String,    // Start of day UTC+0
    pub sod_utc8: String,    // Start of day UTC+8
}

/// OKX Order book data
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OkxOrderBook {
    pub asks: Vec<Vec<String>>, // Ask orders [price, size, liquidated_orders, order_count]
    pub bids: Vec<Vec<String>>, // Bid orders [price, size, liquidated_orders, order_count]
    pub ts: String,             // Timestamp
}

/// OKX Trade data
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxTrade {
    pub inst_id: String,       // Instrument ID
    pub trade_id: String,      // Trade ID
    pub px: String,            // Price
    pub sz: String,            // Size
    pub side: String,          // Side
    pub ts: String,            // Timestamp
    pub count: Option<String>, // Trade count (for aggregated trades)
}

/// OKX Candlestick data
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OkxKline {
    pub ts: String,            // Timestamp
    pub o: String,             // Open price
    pub h: String,             // High price
    pub l: String,             // Low price
    pub c: String,             // Close price
    pub vol: String,           // Volume in base currency
    pub vol_ccy: String,       // Volume in quote currency
    pub vol_ccy_quote: String, // Volume in quote currency
    pub confirm: String,       // Confirmation status
}

/// OKX WebSocket subscription request
#[derive(Debug, Serialize, Clone)]
pub struct OkxWsRequest {
    pub op: String,              // Operation: subscribe, unsubscribe, login
    pub args: Vec<OkxWsChannel>, // Channel arguments
}

/// OKX WebSocket channel
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxWsChannel {
    pub channel: String,             // Channel name
    pub inst_type: Option<String>,   // Instrument type
    pub inst_family: Option<String>, // Instrument family
    pub inst_id: Option<String>,     // Instrument ID
}

/// OKX WebSocket response
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OkxWsResponse<T> {
    pub arg: Option<OkxWsChannel>, // Channel info
    pub data: Option<T>,           // Data payload
    pub action: Option<String>,    // Action type
    pub code: Option<String>,      // Response code
    pub msg: Option<String>,       // Response message
    pub event: Option<String>,     // Event type
}

/// OKX Error response
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkxError {
    pub s_code: String, // Error code
    pub s_msg: String,  // Error message
}
