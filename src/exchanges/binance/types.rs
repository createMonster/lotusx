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
    pub timestamp: i64,
} 