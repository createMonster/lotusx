use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpApiResponse<T> {
    #[serde(rename = "retCode")]
    pub ret_code: i32,
    #[serde(rename = "retMsg")]
    pub ret_msg: String,
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpExchangeInfo {
    pub category: String,
    pub list: Vec<BybitPerpMarket>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpMarket {
    pub symbol: String,
    pub status: String,
    #[serde(rename = "baseCoin")]
    pub base_coin: String,
    #[serde(rename = "quoteCoin")]
    pub quote_coin: String,
    #[serde(rename = "settleCoin")]
    pub settle_coin: String,
    #[serde(rename = "priceScale")]
    pub price_scale: String, // V5 API returns this as string
    #[serde(rename = "lotSizeFilter")]
    pub lot_size_filter: BybitPerpLotSizeFilter,
    #[serde(rename = "priceFilter")]
    pub price_filter: BybitPerpPriceFilter,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpLotSizeFilter {
    #[serde(rename = "minOrderQty")]
    pub min_order_qty: String,
    #[serde(rename = "maxOrderQty")]
    pub max_order_qty: String,
    #[serde(rename = "qtyStep")]
    pub qty_step: String,
    #[serde(rename = "minNotionalValue")]
    pub min_notional_value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpPriceFilter {
    #[serde(rename = "minPrice")]
    pub min_price: String,
    #[serde(rename = "maxPrice")]
    pub max_price: String,
    #[serde(rename = "tickSize")]
    pub tick_size: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpLeverageFilter {
    #[serde(rename = "minLeverage")]
    pub min_leverage: String,
    #[serde(rename = "maxLeverage")]
    pub max_leverage: String,
    #[serde(rename = "leverageStep")]
    pub leverage_step: String,
}

// Account balance response structures
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpCoinBalance {
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
pub struct BybitPerpAccountList {
    #[serde(rename = "accountType")]
    pub account_type: String,
    pub coin: Vec<BybitPerpCoinBalance>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpAccountResult {
    pub list: Vec<BybitPerpAccountList>,
}

// Position response structures
#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpPosition {
    pub symbol: String,
    pub side: String,
    pub size: String,
    #[serde(rename = "avgPrice")]
    pub entry_price: String,
    #[serde(rename = "unrealisedPnl")]
    pub unrealised_pnl: String,
    #[serde(rename = "liqPrice")]
    pub liquidation_price: String,
    pub leverage: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BybitPerpPositionResult {
    pub list: Vec<BybitPerpPosition>,
    pub category: String,
    #[serde(rename = "nextPageCursor")]
    pub next_page_cursor: String,
}

#[derive(Debug, Serialize)]
pub struct BybitPerpOrderRequest {
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
pub struct BybitPerpOrderResponse {
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

// WebSocket Types for Perpetual Futures
#[derive(Debug, Deserialize)]
pub struct BybitPerpWebSocketMessage {
    pub topic: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: serde_json::Value,
    pub ts: i64,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpTickerData {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "prevPrice24h")]
    pub prev_price_24h: String,
    #[serde(rename = "price24hPcnt")]
    pub price_24h_pcnt: String,
    #[serde(rename = "highPrice24h")]
    pub high_price_24h: String,
    #[serde(rename = "lowPrice24h")]
    pub low_price_24h: String,
    #[serde(rename = "volume24h")]
    pub volume_24h: String,
    #[serde(rename = "turnover24h")]
    pub turnover_24h: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpOrderBookData {
    pub symbol: String,
    pub bids: Vec<[String; 2]>,
    pub asks: Vec<[String; 2]>,
    pub ts: i64,
    pub u: i64,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpTradeData {
    pub symbol: String,
    pub side: String,
    pub size: String,
    pub price: String,
    #[serde(rename = "tradeTimeMs")]
    pub trade_time_ms: i64,
    #[serde(rename = "tradeId")]
    pub trade_id: String,
}

#[derive(Debug, Deserialize)]
pub struct BybitPerpKlineData {
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
pub struct BybitPerpMarketsResult {
    pub list: Vec<BybitPerpMarket>,
}

// REST API K-line Types
#[derive(Debug, Deserialize)]
pub struct BybitPerpRestKline {
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
