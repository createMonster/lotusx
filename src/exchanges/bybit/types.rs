use serde::{Deserialize, Serialize};

// API response wrapper for V5 API
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitApiResponse<T> {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: T,
}

// Market data types
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitMarket {
    pub symbol: String,
    pub status: String,
    #[serde(rename = "baseCoin")]
    pub base_coin: String,
    #[serde(rename = "quoteCoin")]
    pub quote_coin: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitMarketsResult {
    pub list: Vec<BybitMarket>,
}

// Account balance types for UNIFIED account
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitCoinBalance {
    pub coin: String,
    #[serde(rename = "walletBalance")]
    pub wallet_balance: String,
    pub locked: String,
    pub equity: String,
    #[serde(rename = "usdValue")]
    pub usd_value: String,
    #[serde(rename = "availableToWithdraw")]
    pub available_to_withdraw: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitAccountList {
    #[serde(rename = "accountType")]
    pub account_type: String,
    pub coin: Vec<BybitCoinBalance>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitAccountResult {
    pub list: Vec<BybitAccountList>,
}

#[derive(Debug, Deserialize)]
pub struct BybitExchangeInfo {
    pub category: String,
    pub list: Vec<BybitMarket>,
}

#[derive(Debug, Deserialize)]
pub struct BybitLotSizeFilter {
    #[serde(rename = "basePrecision")]
    pub base_precision: String,
    #[serde(rename = "quotePrecision")]
    pub quote_precision: String,
    #[serde(rename = "minOrderQty")]
    pub min_order_qty: String,
    #[serde(rename = "maxOrderQty")]
    pub max_order_qty: String,
    #[serde(rename = "minOrderAmt")]
    pub min_order_amt: String,
    #[serde(rename = "maxOrderAmt")]
    pub max_order_amt: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitPriceFilter {
    #[serde(rename = "tickSize")]
    pub tick_size: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitAccountInfo {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: BybitAccountResult,
}

#[derive(Debug, Deserialize)]
pub struct BybitFilter {
    pub filter_type: String,
    pub min_price: Option<String>,
    pub max_price: Option<String>,
    pub min_qty: Option<String>,
    pub max_qty: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BybitV5Result {
    pub category: String,
    pub list: Vec<BybitMarket>,
}

#[derive(Debug, Serialize)]
pub struct BybitOrderRequest {
    pub category: String,
    pub symbol: String,
    pub side: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    pub qty: String,
    pub price: Option<String>,
    #[serde(rename = "timeInForce")]
    pub time_in_force: Option<String>,
    #[serde(rename = "stopPrice")]
    pub stop_price: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BybitOrderResponse {
    #[serde(rename = "orderId")]
    pub order_id: String,
    #[serde(rename = "orderLinkId")]
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    #[serde(rename = "orderType")]
    pub order_type: String,
    pub qty: String,
    pub price: String,
    #[serde(rename = "orderStatus")]
    pub status: String,
    #[serde(rename = "createdTime")]
    pub timestamp: i64,
}

// WebSocket Types
#[derive(Debug, Deserialize)]
pub struct BybitWebSocketTicker {
    pub symbol: String,
    pub price: String,
    pub price_24h_pcnt: String,
    pub price_1h_pcnt: String,
    pub high_price_24h: String,
    pub low_price_24h: String,
    pub turnover_24h: String,
    pub volume_24h: String,
    pub usd_index_price: String,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitWebSocketOrderBook {
    pub symbol: String,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
    pub timestamp: String,
    pub update_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct BybitWebSocketTrade {
    pub symbol: String,
    pub price: String,
    pub size: String,
    pub side: String,
    pub timestamp: String,
    pub trade_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitWebSocketKline {
    pub symbol: String,
    pub kline: BybitKlineData,
}

#[derive(Debug, Deserialize)]
pub struct BybitKlineData {
    pub start_time: i64,
    pub end_time: i64,
    pub interval: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub close_price: String,
    pub volume: String,
    pub turnover: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitRestKline {
    pub start_time: i64,
    pub end_time: i64,
    pub interval: String,
    pub open_price: String,
    pub high_price: String,
    pub low_price: String,
    pub close_price: String,
    pub volume: String,
    pub turnover: String,
} 