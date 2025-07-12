use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct BinanceMarket {
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
    pub filters: Vec<BinanceFilter>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceFilter {
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
pub struct BinanceExchangeInfo {
    pub symbols: Vec<BinanceMarket>,
}

#[derive(Debug, Serialize)]
pub struct BinanceOrderRequest {
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
pub struct BinanceOrderResponse {
    #[serde(rename = "orderId")]
    pub order_id: u64,
    #[serde(rename = "origClientOrderId")]
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "origQty")]
    pub quantity: String,
    pub price: String,
    pub status: String,
    #[serde(rename = "transactTime")]
    pub timestamp: i32,
}

// WebSocket Types
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceWebSocketTicker {
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

#[derive(Debug, Clone, Deserialize)]
pub struct BinanceWebSocketOrderBook {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "U")]
    #[allow(dead_code)]
    pub first_update_id: i64,
    #[serde(rename = "u")]
    pub final_update_id: i64,
    #[serde(rename = "b")]
    pub bids: Vec<[String; 2]>,
    #[serde(rename = "a")]
    pub asks: Vec<[String; 2]>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BinanceWebSocketTrade {
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

#[derive(Debug, Clone, Deserialize)]
pub struct BinanceWebSocketKline {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: BinanceKlineData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BinanceKlineData {
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
pub struct BinanceBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceAccountInfo {
    pub balances: Vec<BinanceBalance>,
}

// REST API K-line Types
#[derive(Debug, Deserialize)]
pub struct BinanceRestKline {
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
