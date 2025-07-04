use super::types::{BybitKlineData, BybitMarket};
use crate::core::types::{
    Kline, Market, MarketDataType, OrderSide, OrderType, Symbol, Ticker, TimeInForce, Trade,
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

#[allow(clippy::too_many_lines)]
pub fn parse_websocket_message(value: Value) -> Option<MarketDataType> {
    // Handle Bybit V5 WebSocket message format
    if let Some(topic) = value.get("topic").and_then(|t| t.as_str()) {
        if let Some(data) = value.get("data") {
            // Parse ticker data
            if topic.starts_with("tickers.") {
                if let Some(ticker_data) = data.as_object() {
                    let symbol = topic.strip_prefix("tickers.").unwrap_or("").to_string();
                    return Some(MarketDataType::Ticker(Ticker {
                        symbol,
                        price: ticker_data
                            .get("lastPrice")
                            .and_then(|p| p.as_str())
                            .unwrap_or("0")
                            .to_string(),
                        price_change: ticker_data
                            .get("price24hChg")
                            .and_then(|c| c.as_str())
                            .unwrap_or("0")
                            .to_string(),
                        price_change_percent: ticker_data
                            .get("price24hPcnt")
                            .and_then(|c| c.as_str())
                            .unwrap_or("0")
                            .to_string(),
                        high_price: ticker_data
                            .get("highPrice24h")
                            .and_then(|h| h.as_str())
                            .unwrap_or("0")
                            .to_string(),
                        low_price: ticker_data
                            .get("lowPrice24h")
                            .and_then(|l| l.as_str())
                            .unwrap_or("0")
                            .to_string(),
                        volume: ticker_data
                            .get("volume24h")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0")
                            .to_string(),
                        quote_volume: ticker_data
                            .get("turnover24h")
                            .and_then(|q| q.as_str())
                            .unwrap_or("0")
                            .to_string(),
                        open_time: 0,  // Not provided in Bybit ticker data
                        close_time: 0, // Not provided in Bybit ticker data
                        count: 0,      // Not provided in Bybit ticker data
                    }));
                }
            }

            // Parse trade data
            if topic.starts_with("publicTrade.") {
                if let Some(trades) = data.as_array() {
                    let symbol = topic.strip_prefix("publicTrade.").unwrap_or("").to_string();
                    for trade in trades {
                        if let Some(trade_obj) = trade.as_object() {
                            return Some(MarketDataType::Trade(Trade {
                                symbol,
                                id: trade_obj
                                    .get("i")
                                    .and_then(|i| i.as_str())
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or(0),
                                price: trade_obj
                                    .get("p")
                                    .and_then(|p| p.as_str())
                                    .unwrap_or("0")
                                    .to_string(),
                                quantity: trade_obj
                                    .get("v")
                                    .and_then(|q| q.as_str())
                                    .unwrap_or("0")
                                    .to_string(),
                                time: trade_obj.get("T").and_then(|t| t.as_i64()).unwrap_or(0),
                                is_buyer_maker: trade_obj
                                    .get("S")
                                    .and_then(|s| s.as_str())
                                    .is_some_and(|s| s == "Sell"),
                            }));
                        }
                    }
                }
            }

            // Parse kline data
            if topic.contains("kline.") {
                if let Some(klines) = data.as_array() {
                    let topic_parts: Vec<&str> = topic.split('.').collect();
                    if topic_parts.len() >= 3 {
                        let symbol = topic_parts[2].to_string();
                        let interval = topic_parts[1].to_string();

                        for kline in klines {
                            if let Some(kline_obj) = kline.as_object() {
                                return Some(MarketDataType::Kline(Kline {
                                    symbol,
                                    open_time: kline_obj
                                        .get("start")
                                        .and_then(|t| t.as_i64())
                                        .unwrap_or(0),
                                    close_time: kline_obj
                                        .get("end")
                                        .and_then(|t| t.as_i64())
                                        .unwrap_or(0),
                                    interval,
                                    open_price: kline_obj
                                        .get("open")
                                        .and_then(|p| p.as_str())
                                        .unwrap_or("0")
                                        .to_string(),
                                    high_price: kline_obj
                                        .get("high")
                                        .and_then(|p| p.as_str())
                                        .unwrap_or("0")
                                        .to_string(),
                                    low_price: kline_obj
                                        .get("low")
                                        .and_then(|p| p.as_str())
                                        .unwrap_or("0")
                                        .to_string(),
                                    close_price: kline_obj
                                        .get("close")
                                        .and_then(|p| p.as_str())
                                        .unwrap_or("0")
                                        .to_string(),
                                    volume: kline_obj
                                        .get("volume")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("0")
                                        .to_string(),
                                    number_of_trades: 0,
                                    final_bar: kline_obj
                                        .get("confirm")
                                        .and_then(|c| c.as_bool())
                                        .unwrap_or(true),
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
