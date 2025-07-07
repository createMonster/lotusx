use super::types as binance_perp_types;
use crate::core::types::{
    conversion, Kline, Market, MarketDataType, OrderBook, OrderBookEntry, OrderSide, OrderType,
    Symbol, Ticker, TimeInForce, Trade,
};
use serde_json::Value;

/// Convert binance perp market to core market type
pub fn convert_binance_perp_market(
    binance_market: binance_perp_types::BinancePerpMarket,
) -> Market {
    let mut min_qty = None;
    let mut max_qty = None;
    let mut min_price = None;
    let mut max_price = None;

    for filter in &binance_market.filters {
        match filter.filter_type.as_str() {
            "LOT_SIZE" => {
                min_qty = filter
                    .min_qty
                    .as_ref()
                    .map(|q| conversion::string_to_quantity(q));
                max_qty = filter
                    .max_qty
                    .as_ref()
                    .map(|q| conversion::string_to_quantity(q));
            }
            "PRICE_FILTER" => {
                min_price = filter
                    .min_price
                    .as_ref()
                    .map(|p| conversion::string_to_price(p));
                max_price = filter
                    .max_price
                    .as_ref()
                    .map(|p| conversion::string_to_price(p));
            }
            _ => {}
        }
    }

    Market {
        symbol: Symbol::new(binance_market.base_asset, binance_market.quote_asset)
            .unwrap_or_else(|_| conversion::string_to_symbol(&binance_market.symbol)),
        status: binance_market.status,
        base_precision: binance_market.base_asset_precision,
        quote_precision: binance_market.quote_precision,
        min_qty,
        max_qty,
        min_price,
        max_price,
    }
}

/// Convert order side to binance perp format
pub fn convert_order_side(side: &OrderSide) -> String {
    match side {
        OrderSide::Buy => "BUY".to_string(),
        OrderSide::Sell => "SELL".to_string(),
    }
}

/// Convert order type to binance perp format
pub fn convert_order_type(order_type: &OrderType) -> String {
    match order_type {
        OrderType::Market => "MARKET".to_string(),
        OrderType::Limit => "LIMIT".to_string(),
        OrderType::StopLoss => "STOP".to_string(),
        OrderType::StopLossLimit => "STOP_MARKET".to_string(),
        OrderType::TakeProfit => "TAKE_PROFIT".to_string(),
        OrderType::TakeProfitLimit => "TAKE_PROFIT_MARKET".to_string(),
    }
}

/// Convert time in force to binance perp format
pub fn convert_time_in_force(tif: &TimeInForce) -> String {
    match tif {
        TimeInForce::GTC => "GTC".to_string(),
        TimeInForce::IOC => "IOC".to_string(),
        TimeInForce::FOK => "FOK".to_string(),
    }
}

/// Parse websocket message from binance perp
pub fn parse_websocket_message(value: Value) -> Option<MarketDataType> {
    if let Some(stream) = value.get("stream").and_then(|s| s.as_str()) {
        if let Some(data) = value.get("data") {
            if stream.contains("@ticker") {
                if let Ok(ticker) = serde_json::from_value::<
                    binance_perp_types::BinancePerpWebSocketTicker,
                >(data.clone())
                {
                    return Some(MarketDataType::Ticker(Ticker {
                        symbol: conversion::string_to_symbol(&ticker.symbol),
                        price: conversion::string_to_price(&ticker.price),
                        price_change: conversion::string_to_price(&ticker.price_change),
                        price_change_percent: conversion::string_to_decimal(
                            &ticker.price_change_percent,
                        ),
                        high_price: conversion::string_to_price(&ticker.high_price),
                        low_price: conversion::string_to_price(&ticker.low_price),
                        volume: conversion::string_to_volume(&ticker.volume),
                        quote_volume: conversion::string_to_volume(&ticker.quote_volume),
                        open_time: ticker.open_time,
                        close_time: ticker.close_time,
                        count: ticker.count,
                    }));
                }
            } else if stream.contains("@depth") {
                if let Ok(depth) = serde_json::from_value::<
                    binance_perp_types::BinancePerpWebSocketOrderBook,
                >(data.clone())
                {
                    let bids = depth
                        .bids
                        .into_iter()
                        .map(|b| OrderBookEntry {
                            price: conversion::string_to_price(&b[0]),
                            quantity: conversion::string_to_quantity(&b[1]),
                        })
                        .collect();
                    let asks = depth
                        .asks
                        .into_iter()
                        .map(|a| OrderBookEntry {
                            price: conversion::string_to_price(&a[0]),
                            quantity: conversion::string_to_quantity(&a[1]),
                        })
                        .collect();

                    return Some(MarketDataType::OrderBook(OrderBook {
                        symbol: conversion::string_to_symbol(&depth.symbol),
                        bids,
                        asks,
                        last_update_id: depth.final_update_id,
                    }));
                }
            } else if stream.contains("@aggTrade") {
                if let Ok(trade) = serde_json::from_value::<
                    binance_perp_types::BinancePerpWebSocketTrade,
                >(data.clone())
                {
                    return Some(MarketDataType::Trade(Trade {
                        symbol: conversion::string_to_symbol(&trade.symbol),
                        id: trade.id,
                        price: conversion::string_to_price(&trade.price),
                        quantity: conversion::string_to_quantity(&trade.quantity),
                        time: trade.time,
                        is_buyer_maker: trade.is_buyer_maker,
                    }));
                }
            } else if stream.contains("@kline") {
                if let Ok(kline_data) = serde_json::from_value::<
                    binance_perp_types::BinancePerpWebSocketKline,
                >(data.clone())
                {
                    return Some(MarketDataType::Kline(Kline {
                        symbol: conversion::string_to_symbol(&kline_data.symbol),
                        open_time: kline_data.kline.open_time,
                        close_time: kline_data.kline.close_time,
                        interval: kline_data.kline.interval,
                        open_price: conversion::string_to_price(&kline_data.kline.open_price),
                        high_price: conversion::string_to_price(&kline_data.kline.high_price),
                        low_price: conversion::string_to_price(&kline_data.kline.low_price),
                        close_price: conversion::string_to_price(&kline_data.kline.close_price),
                        volume: conversion::string_to_volume(&kline_data.kline.volume),
                        number_of_trades: kline_data.kline.number_of_trades,
                        final_bar: kline_data.kline.final_bar,
                    }));
                }
            }
        }
    }
    None
}
