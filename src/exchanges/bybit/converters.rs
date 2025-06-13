use crate::core::types::{Kline, Symbol};
use super::types::{BybitMarket, BybitKlineData};

pub fn convert_bybit_market_to_symbol(bybit_market: &BybitMarket) -> Symbol {
    Symbol {
        symbol: bybit_market.symbol.clone(),
        base: bybit_market.base_coin.clone(),
        quote: bybit_market.quote_coin.clone(),
    }
}

pub fn convert_bybit_kline_to_kline(symbol: String, interval: String, bybit_kline: &BybitKlineData) -> Kline {
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