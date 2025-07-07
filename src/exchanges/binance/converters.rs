use super::types as binance_types;
use crate::core::types::{
    conversion, Kline, Market, MarketDataType, OrderBook, OrderBookEntry, OrderSide, OrderType,
    Symbol, Ticker, TimeInForce, Trade,
};
use serde_json::Value;

/// Convert binance market to core market type
pub fn convert_binance_market(
    binance_market: binance_types::BinanceMarket,
) -> Result<Market, String> {
    let mut min_qty = None;
    let mut max_qty = None;
    let mut min_price = None;
    let mut max_price = None;

    for filter in &binance_market.filters {
        match filter.filter_type.as_str() {
            "LOT_SIZE" => {
                if let Some(min_q) = &filter.min_qty {
                    min_qty = Some(conversion::string_to_quantity(min_q));
                }
                if let Some(max_q) = &filter.max_qty {
                    max_qty = Some(conversion::string_to_quantity(max_q));
                }
            }
            "PRICE_FILTER" => {
                if let Some(min_p) = &filter.min_price {
                    min_price = Some(conversion::string_to_price(min_p));
                }
                if let Some(max_p) = &filter.max_price {
                    max_price = Some(conversion::string_to_price(max_p));
                }
            }
            _ => {}
        }
    }

    let symbol = Symbol::new(binance_market.base_asset, binance_market.quote_asset)
        .unwrap_or_else(|_| conversion::string_to_symbol(&binance_market.symbol));

    Ok(Market {
        symbol,
        status: binance_market.status,
        base_precision: binance_market.base_asset_precision,
        quote_precision: binance_market.quote_precision,
        min_qty,
        max_qty,
        min_price,
        max_price,
    })
}

/// Convert order side to binance format
pub fn convert_order_side(side: &OrderSide) -> String {
    match side {
        OrderSide::Buy => "BUY".to_string(),
        OrderSide::Sell => "SELL".to_string(),
    }
}

/// Convert order type to binance format
pub fn convert_order_type(order_type: &OrderType) -> String {
    match order_type {
        OrderType::Market => "MARKET".to_string(),
        OrderType::Limit => "LIMIT".to_string(),
        OrderType::StopLoss => "STOP_LOSS".to_string(),
        OrderType::StopLossLimit => "STOP_LOSS_LIMIT".to_string(),
        OrderType::TakeProfit => "TAKE_PROFIT".to_string(),
        OrderType::TakeProfitLimit => "TAKE_PROFIT_LIMIT".to_string(),
    }
}

/// Convert time in force to binance format
pub fn convert_time_in_force(tif: &TimeInForce) -> String {
    match tif {
        TimeInForce::GTC => "GTC".to_string(),
        TimeInForce::IOC => "IOC".to_string(),
        TimeInForce::FOK => "FOK".to_string(),
    }
}

/// Parse websocket message from binance
pub fn parse_websocket_message(value: Value) -> Option<MarketDataType> {
    if let Some(stream) = value.get("stream").and_then(|s| s.as_str()) {
        if let Some(data) = value.get("data") {
            if stream.contains("@ticker") {
                if let Ok(ticker) =
                    serde_json::from_value::<binance_types::BinanceWebSocketTicker>(data.clone())
                {
                    // Convert string fields to proper types using conversion helpers
                    let symbol = conversion::string_to_symbol(&ticker.symbol);
                    let price = conversion::string_to_price(&ticker.price);
                    let price_change = conversion::string_to_price(&ticker.price_change);
                    let price_change_percent =
                        conversion::string_to_decimal(&ticker.price_change_percent);
                    let high_price = conversion::string_to_price(&ticker.high_price);
                    let low_price = conversion::string_to_price(&ticker.low_price);
                    let volume = conversion::string_to_volume(&ticker.volume);
                    let quote_volume = conversion::string_to_volume(&ticker.quote_volume);

                    return Some(MarketDataType::Ticker(Ticker {
                        symbol,
                        price,
                        price_change,
                        price_change_percent,
                        high_price,
                        low_price,
                        volume,
                        quote_volume,
                        open_time: ticker.open_time,
                        close_time: ticker.close_time,
                        count: ticker.count,
                    }));
                }
            } else if stream.contains("@depth") {
                if let Ok(depth) =
                    serde_json::from_value::<binance_types::BinanceWebSocketOrderBook>(data.clone())
                {
                    let symbol = conversion::string_to_symbol(&depth.symbol);

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
                        symbol,
                        bids,
                        asks,
                        last_update_id: depth.final_update_id,
                    }));
                }
            } else if stream.contains("@trade") {
                if let Ok(trade) =
                    serde_json::from_value::<binance_types::BinanceWebSocketTrade>(data.clone())
                {
                    let symbol = conversion::string_to_symbol(&trade.symbol);
                    let price = conversion::string_to_price(&trade.price);
                    let quantity = conversion::string_to_quantity(&trade.quantity);

                    return Some(MarketDataType::Trade(Trade {
                        symbol,
                        id: trade.id,
                        price,
                        quantity,
                        time: trade.time,
                        is_buyer_maker: trade.is_buyer_maker,
                    }));
                }
            } else if stream.contains("@kline") {
                if let Ok(kline_data) =
                    serde_json::from_value::<binance_types::BinanceWebSocketKline>(data.clone())
                {
                    let symbol = conversion::string_to_symbol(&kline_data.symbol);
                    let open_price = conversion::string_to_price(&kline_data.kline.open_price);
                    let high_price = conversion::string_to_price(&kline_data.kline.high_price);
                    let low_price = conversion::string_to_price(&kline_data.kline.low_price);
                    let close_price = conversion::string_to_price(&kline_data.kline.close_price);
                    let volume = conversion::string_to_volume(&kline_data.kline.volume);

                    return Some(MarketDataType::Kline(Kline {
                        symbol,
                        open_time: kline_data.kline.open_time,
                        close_time: kline_data.kline.close_time,
                        interval: kline_data.kline.interval,
                        open_price,
                        high_price,
                        low_price,
                        close_price,
                        volume,
                        number_of_trades: kline_data.kline.number_of_trades,
                        final_bar: kline_data.kline.final_bar,
                    }));
                }
            }
        }
    }
    None
}
