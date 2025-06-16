use super::types::{BybitKlineData, BybitMarket};
use crate::core::types::{
    Kline, Market, MarketDataType, OrderSide, OrderType, Symbol, TimeInForce,
};
use serde_json::Value;

pub fn convert_bybit_market_to_symbol(bybit_market: &BybitMarket) -> Symbol {
    Symbol {
        symbol: bybit_market.symbol.clone(),
        base: bybit_market.base_coin.clone(),
        quote: bybit_market.quote_coin.clone(),
    }
}

pub fn convert_bybit_market(bybit_market: BybitMarket) -> Market {
    Market {
        symbol: Symbol {
            base: bybit_market.base_coin,
            quote: bybit_market.quote_coin,
            symbol: bybit_market.symbol.clone(),
        },
        status: bybit_market.status,
        base_precision: 8, // Default precision for spot markets
        quote_precision: 8,
        min_qty: None,
        max_qty: None,
        min_price: None,
        max_price: None,
    }
}

/// Convert order side to Bybit format
pub fn convert_order_side(side: &OrderSide) -> String {
    match side {
        OrderSide::Buy => "Buy".to_string(),
        OrderSide::Sell => "Sell".to_string(),
    }
}

/// Convert order type to Bybit format
pub fn convert_order_type(order_type: &OrderType) -> String {
    match order_type {
        OrderType::Market => "Market".to_string(),
        OrderType::Limit => "Limit".to_string(),
        OrderType::StopLoss => "StopMarket".to_string(),
        OrderType::StopLossLimit => "StopLimit".to_string(),
        OrderType::TakeProfit => "TakeProfit".to_string(),
        OrderType::TakeProfitLimit => "TakeProfitLimit".to_string(),
    }
}

/// Convert time in force to Bybit format
pub fn convert_time_in_force(tif: &TimeInForce) -> String {
    match tif {
        TimeInForce::GTC => "GTC".to_string(),
        TimeInForce::IOC => "IOC".to_string(),
        TimeInForce::FOK => "FOK".to_string(),
    }
}

pub fn convert_bybit_kline_to_kline(
    symbol: String,
    interval: String,
    bybit_kline: &BybitKlineData,
) -> Kline {
    Kline {
        symbol,
        open_time: bybit_kline.start_time,
        close_time: bybit_kline.end_time,
        interval,
        open_price: bybit_kline.open_price.clone(),
        high_price: bybit_kline.high_price.clone(),
        low_price: bybit_kline.low_price.clone(),
        close_price: bybit_kline.close_price.clone(),
        volume: bybit_kline.volume.clone(),
        number_of_trades: 0, // Bybit doesn't provide this
        final_bar: true,
    }
}

pub fn parse_websocket_message(_value: Value) -> Option<MarketDataType> {
    // Placeholder implementation for WebSocket message parsing
    // This would need to be implemented based on Bybit's WebSocket message format
    None
}
