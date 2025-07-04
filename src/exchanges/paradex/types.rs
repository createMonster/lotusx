use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexAsset {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexMarket {
    pub symbol: String,
    pub base_asset: ParadexAsset,
    pub quote_asset: ParadexAsset,
    pub status: String,
    pub state: String,
    pub tick_size: String,
    pub step_size: String,
    pub min_order_size: String,
    pub max_order_size: String,
    pub min_price: String,
    pub max_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexOrder {
    pub id: String,
    pub client_id: String,
    pub market: String,
    pub side: String,
    pub order_type: String,
    pub size: String,
    pub price: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexFill {
    pub id: i64,
    pub market: String,
    pub side: String,
    pub size: String,
    pub price: String,
    pub fee: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexPosition {
    pub market: String,
    pub side: String,
    pub average_entry_price: String,
    pub size: String,
    pub unrealized_pnl: String,
    pub liquidation_price: Option<String>,
    pub leverage: String,
}
