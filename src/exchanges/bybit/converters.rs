use super::types::{BybitKlineData, BybitMarket};
use crate::core::types::{
    Kline, Market, MarketDataType, OrderSide, OrderType, Symbol, Ticker, TimeInForce, Trade,
    conversion,
};
use serde_json::Value;

pub fn convert_bybit_market_to_symbol(bybit_market: &BybitMarket) -> Symbol {
    Symbol::new(
        bybit_market.base_coin.clone(),
        bybit_market.quote_coin.clone(),
    )
    .unwrap_or_else(|_| crate::core::types::conversion::string_to_symbol(&bybit_market.symbol))
}

pub fn convert_bybit_market(bybit_market: BybitMarket) -> Market {
    Market {
        symbol: Symbol::new(bybit_market.base_coin, bybit_market.quote_coin).unwrap_or_else(|_| {
            crate::core::types::conversion::string_to_symbol(&bybit_market.symbol)
        }),
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
        symbol: conversion::string_to_symbol(&symbol),
        open_time: bybit_kline.start_time,
        close_time: bybit_kline.end_time,
        interval,
        open_price: conversion::string_to_price(&bybit_kline.open_price),
        high_price: conversion::string_to_price(&bybit_kline.high_price),
        low_price: conversion::string_to_price(&bybit_kline.low_price),
        close_price: conversion::string_to_price(&bybit_kline.close_price),
        volume: conversion::string_to_volume(&bybit_kline.volume),
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
                        symbol: conversion::string_to_symbol(&symbol),
                        price: conversion::string_to_price(
                            ticker_data
                                .get("lastPrice")
                                .and_then(|p| p.as_str())
                                .unwrap_or("0")
                        ),
                        price_change: conversion::string_to_price(
                            ticker_data
                                .get("price24hChg")
                                .and_then(|c| c.as_str())
                                .unwrap_or("0")
                        ),
                        price_change_percent: conversion::string_to_decimal(
                            ticker_data
                                .get("price24hPcnt")
                                .and_then(|c| c.as_str())
                                .unwrap_or("0")
                        ),
                        high_price: conversion::string_to_price(
                            ticker_data
                                .get("highPrice24h")
                                .and_then(|h| h.as_str())
                                .unwrap_or("0")
                        ),
                        low_price: conversion::string_to_price(
                            ticker_data
                                .get("lowPrice24h")
                                .and_then(|l| l.as_str())
                                .unwrap_or("0")
                        ),
                        volume: conversion::string_to_volume(
                            ticker_data
                                .get("volume24h")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0")
                        ),
                        quote_volume: conversion::string_to_volume(
                            ticker_data
                                .get("turnover24h")
                                .and_then(|q| q.as_str())
                                .unwrap_or("0")
                        ),
                        open_time: 0,
                        close_time: 0,
                        count: 0,
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
                                symbol: conversion::string_to_symbol(&symbol),
                                id: trade_obj
                                    .get("i")
                                    .and_then(|i| i.as_str())
                                    .and_then(|s| s.parse::<i64>().ok())
                                    .unwrap_or(0),
                                price: conversion::string_to_price(
                                    trade_obj
                                        .get("p")
                                        .and_then(|p| p.as_str())
                                        .unwrap_or("0")
                                ),
                                quantity: conversion::string_to_quantity(
                                    trade_obj
                                        .get("v")
                                        .and_then(|q| q.as_str())
                                        .unwrap_or("0")
                                ),
                                time: trade_obj
                                    .get("T")
                                    .and_then(|t| t.as_i64())
                                    .unwrap_or(0),
                                is_buyer_maker: trade_obj
                                    .get("S")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s == "Buy")
                                    .unwrap_or(false),
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
                                    symbol: conversion::string_to_symbol(&symbol),
                                    open_time: kline_obj
                                        .get("start")
                                        .and_then(|t| t.as_i64())
                                        .unwrap_or(0),
                                    close_time: kline_obj
                                        .get("end")
                                        .and_then(|t| t.as_i64())
                                        .unwrap_or(0),
                                                                                                             interval,
                                    open_price: conversion::string_to_price(
                                        kline_obj
                                            .get("open")
                                            .and_then(|p| p.as_str())
                                            .unwrap_or("0")
                                    ),
                                    high_price: conversion::string_to_price(
                                        kline_obj
                                            .get("high")
                                            .and_then(|p| p.as_str())
                                            .unwrap_or("0")
                                    ),
                                    low_price: conversion::string_to_price(
                                        kline_obj
                                            .get("low")
                                            .and_then(|p| p.as_str())
                                            .unwrap_or("0")
                                    ),
                                    close_price: conversion::string_to_price(
                                        kline_obj
                                            .get("close")
                                            .and_then(|p| p.as_str())
                                            .unwrap_or("0")
                                    ),
                                    volume: conversion::string_to_volume(
                                        kline_obj
                                            .get("volume")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("0")
                                    ),
                                    number_of_trades: 0,
                                    final_bar: true,
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
