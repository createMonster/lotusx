use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct BinancePerpMarket {
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
    pub filters: Vec<BinancePerpFilter>,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpFilter {
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
pub struct BinancePerpExchangeInfo {
    pub symbols: Vec<BinancePerpMarket>,
}

#[derive(Debug, Serialize)]
pub struct BinancePerpOrderRequest {
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
pub struct BinancePerpOrderResponse {
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
    #[serde(rename = "updateTime")]
    pub timestamp: i64,
}

// WebSocket Types for Perpetual Futures
#[derive(Debug, Deserialize)]
pub struct BinancePerpWebSocketTicker {
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

#[derive(Debug, Deserialize)]
pub struct BinancePerpWebSocketOrderBook {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "U")]
    pub first_update_id: i64,
    #[serde(rename = "u")]
    pub final_update_id: i64,
    #[serde(rename = "pu")]
    pub prev_final_update_id: i64,
    #[serde(rename = "b")]
    pub bids: Vec<[String; 2]>,
    #[serde(rename = "a")]
    pub asks: Vec<[String; 2]>,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpWebSocketTrade {
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

#[derive(Debug, Deserialize)]
pub struct BinancePerpWebSocketKline {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: BinancePerpKlineData,
}

#[derive(Debug, Deserialize)]
pub struct BinancePerpKlineData {
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
#[serde(rename_all = "camelCase")]
pub struct BinancePerpBalance {
    pub asset: String,
    pub balance: String,
    #[serde(rename = "availableBalance")]
    pub available_balance: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinancePerpPosition {
    pub symbol: String,
    #[serde(rename = "positionAmt")]
    pub position_amount: String,
    pub entry_price: String,
    #[serde(rename = "unRealizedProfit")]
    pub un_realized_profit: String,
    pub liquidation_price: String,
    pub leverage: String,
    pub position_side: crate::core::types::PositionSide,
}
