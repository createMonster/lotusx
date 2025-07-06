use super::types as bybit_perp_types;
use super::types::{BybitPerpKlineData, BybitPerpMarket};
use crate::core::types::{
    Kline, Market, MarketDataType, OrderBook, OrderBookEntry, OrderSide, OrderType, Symbol, Ticker,
    TimeInForce, Trade,
};
use serde_json::Value;

/// Convert bybit perp market to core market type
pub fn convert_bybit_perp_market(bybit_perp_market: bybit_perp_types::BybitPerpMarket) -> Market {
    // Parse precision from price scale string
    let price_precision = bybit_perp_market.price_scale.parse::<i32>().unwrap_or(2);

    // For perpetuals, qty step indicates base precision
    let base_precision = bybit_perp_market
        .lot_size_filter
        .qty_step
        .parse::<f64>()
        .map(|p| (-p.log10()).ceil() as i32)
        .unwrap_or(3);

    Market {
        symbol: Symbol::new(bybit_perp_market.base_coin, bybit_perp_market.quote_coin)
            .unwrap_or_else(|_| crate::core::types::conversion::string_to_symbol(&bybit_perp_market.symbol)),
        status: bybit_perp_market.status,
        base_precision,
        quote_precision: price_precision,
        min_qty: Some(crate::core::types::conversion::string_to_quantity(&bybit_perp_market.lot_size_filter.min_order_qty)),
        max_qty: Some(crate::core::types::conversion::string_to_quantity(&bybit_perp_market.lot_size_filter.max_order_qty)),
        min_price: Some(crate::core::types::conversion::string_to_price(&bybit_perp_market.price_filter.min_price)),
        max_price: Some(crate::core::types::conversion::string_to_price(&bybit_perp_market.price_filter.max_price)),
    }
}

/// Convert order side to bybit perp format
pub fn convert_order_side(side: &OrderSide) -> String {
    match side {
        OrderSide::Buy => "Buy".to_string(),
        OrderSide::Sell => "Sell".to_string(),
    }
}

/// Convert order type to bybit perp format
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

/// Convert time in force to bybit perp format
pub fn convert_time_in_force(tif: &TimeInForce) -> String {
    match tif {
        TimeInForce::GTC => "GTC".to_string(),
        TimeInForce::IOC => "IOC".to_string(),
        TimeInForce::FOK => "FOK".to_string(),
    }
}

/// Convert bybit perp kline to core kline type
pub fn convert_bybit_perp_kline(
    symbol: String,
    interval: String,
    bybit_perp_kline: bybit_perp_types::BybitPerpRestKline,
) -> Kline {
    use crate::core::types::conversion;
    
    Kline {
        symbol: conversion::string_to_symbol(&symbol),
        open_time: bybit_perp_kline.start_time,
        close_time: bybit_perp_kline.end_time,
        interval,
        open_price: conversion::string_to_price(&bybit_perp_kline.open_price),
        high_price: conversion::string_to_price(&bybit_perp_kline.high_price),
        low_price: conversion::string_to_price(&bybit_perp_kline.low_price),
        close_price: conversion::string_to_price(&bybit_perp_kline.close_price),
        volume: conversion::string_to_volume(&bybit_perp_kline.volume),
        number_of_trades: 0, // Bybit doesn't provide this in REST API
        final_bar: true,
    }
}

/// Parse WebSocket message and convert to `MarketDataType`
pub fn parse_websocket_message(value: Value) -> Option<MarketDataType> {
    // Extract topic and data from Bybit WebSocket message
    let topic = value["topic"].as_str().unwrap_or("");
    let data = &value["data"];

    if topic.contains("ticker") {
        if let Ok(ticker) =
            serde_json::from_value::<bybit_perp_types::BybitPerpTickerData>(data.clone())
        {
            use crate::core::types::conversion;
            
            return Some(MarketDataType::Ticker(Ticker {
                symbol: conversion::string_to_symbol(&ticker.symbol),
                price: conversion::string_to_price(&ticker.last_price),
                price_change: conversion::string_to_price("0"), // Not provided in Bybit ticker
                price_change_percent: conversion::string_to_decimal(&ticker.price_24h_pcnt),
                high_price: conversion::string_to_price(&ticker.high_price_24h),
                low_price: conversion::string_to_price(&ticker.low_price_24h),
                volume: conversion::string_to_volume(&ticker.volume_24h),
                quote_volume: conversion::string_to_volume(&ticker.turnover_24h),
                open_time: 0,  // Not provided in Bybit ticker
                close_time: 0, // Not provided in Bybit ticker
                count: 0,      // Not provided in Bybit ticker
            }));
        }
    } else if topic.contains("orderbook") {
        if let Ok(orderbook) =
            serde_json::from_value::<bybit_perp_types::BybitPerpOrderBookData>(data.clone())
        {
            use crate::core::types::conversion;
            
            let bids = orderbook
                .bids
                .into_iter()
                .map(|[price, qty]| OrderBookEntry {
                    price: conversion::string_to_price(&price),
                    quantity: conversion::string_to_quantity(&qty),
                })
                .collect();

            let asks = orderbook
                .asks
                .into_iter()
                .map(|[price, qty]| OrderBookEntry {
                    price: conversion::string_to_price(&price),
                    quantity: conversion::string_to_quantity(&qty),
                })
                .collect();

            return Some(MarketDataType::OrderBook(OrderBook {
                symbol: conversion::string_to_symbol(&orderbook.symbol),
                bids,
                asks,
                last_update_id: orderbook.u,
            }));
        }
    } else if topic.contains("trade") {
        if let Ok(trade) =
            serde_json::from_value::<bybit_perp_types::BybitPerpTradeData>(data.clone())
        {
            use crate::core::types::conversion;
            
            return Some(MarketDataType::Trade(Trade {
                symbol: conversion::string_to_symbol(&trade.symbol),
                id: trade.trade_id.parse().unwrap_or(0),
                price: conversion::string_to_price(&trade.price),
                quantity: conversion::string_to_quantity(&trade.size),
                time: trade.trade_time_ms,
                is_buyer_maker: trade.side == "Sell",
            }));
        }
    } else if topic.contains("kline") {
        if let Ok(kline) =
            serde_json::from_value::<bybit_perp_types::BybitPerpKlineData>(data.clone())
        {
            use crate::core::types::conversion;
            
            return Some(MarketDataType::Kline(Kline {
                symbol: conversion::string_to_symbol(""), // Extract from topic
                open_time: kline.start_time,
                close_time: kline.end_time,
                interval: kline.interval,
                open_price: conversion::string_to_price(&kline.open_price),
                high_price: conversion::string_to_price(&kline.high_price),
                low_price: conversion::string_to_price(&kline.low_price),
                close_price: conversion::string_to_price(&kline.close_price),
                volume: conversion::string_to_volume(&kline.volume),
                number_of_trades: 0, // Not provided in Bybit kline
                final_bar: true,
            }));
        }
    }

    None
}

pub fn convert_bybit_perp_market_to_symbol(bybit_perp_market: &BybitPerpMarket) -> Symbol {
    Symbol::new(bybit_perp_market.base_coin.clone(), bybit_perp_market.quote_coin.clone())
        .unwrap_or_else(|_| crate::core::types::conversion::string_to_symbol(&bybit_perp_market.symbol))
}

pub fn convert_bybit_perp_kline_to_kline(
    symbol: String,
    interval: String,
    bybit_kline: &BybitPerpKlineData,
) -> Kline {
    use crate::core::types::conversion;
    
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
