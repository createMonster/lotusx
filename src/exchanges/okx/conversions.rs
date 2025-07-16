use super::types as okx_types;
use crate::core::types::{
    conversion, Kline, Market, MarketDataType, OrderBook, OrderBookEntry, OrderSide, OrderType,
    Symbol, Ticker, TimeInForce, Trade,
};
use serde_json::Value;
use std::str::FromStr;

/// Convert OKX market to core market type
pub fn convert_okx_market(okx_market: okx_types::OkxMarket) -> Result<Market, String> {
    // Parse symbol from inst_id (e.g., "BTC-USDT")
    let symbol = conversion::string_to_symbol(&okx_market.inst_id);

    // Convert tick size and lot size to appropriate types
    let tick_size = conversion::string_to_price(&okx_market.tick_sz);
    let lot_size = conversion::string_to_quantity(&okx_market.lot_sz);
    let min_size = conversion::string_to_quantity(&okx_market.min_sz);

    Ok(Market {
        symbol,
        status: okx_market.state,
        base_precision: 8, // OKX doesn't provide precision directly, using default
        quote_precision: 8,
        min_qty: Some(min_size),
        max_qty: None, // OKX doesn't specify max quantity directly
        min_price: Some(tick_size),
        max_price: None, // OKX doesn't specify max price directly
        tick_size: Some(tick_size),
        lot_size: Some(lot_size),
        exchange: "okx".to_string(),
        fees: None, // Would need separate API call to get fee information
    })
}

/// Convert OKX ticker to core ticker type
pub fn convert_okx_ticker(okx_ticker: okx_types::OkxTicker) -> Result<Ticker, String> {
    let symbol = conversion::string_to_symbol(&okx_ticker.inst_id);
    let last_price = conversion::string_to_price(&okx_ticker.last);
    let bid_price = conversion::string_to_price(&okx_ticker.bid_px);
    let ask_price = conversion::string_to_price(&okx_ticker.ask_px);
    let bid_quantity = conversion::string_to_quantity(&okx_ticker.bid_sz);
    let ask_quantity = conversion::string_to_quantity(&okx_ticker.ask_sz);

    // Parse timestamp
    let timestamp = okx_ticker
        .ts
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse timestamp: {}", e))?;

    // Calculate 24h change
    let open_24h = conversion::string_to_price(&okx_ticker.open_24h);
    let price_change_24h = last_price - open_24h;
    let price_change_percent_24h = if open_24h > 0.0 {
        (price_change_24h / open_24h) * 100.0
    } else {
        0.0
    };

    Ok(Ticker {
        symbol,
        last_price,
        bid_price,
        ask_price,
        bid_quantity,
        ask_quantity,
        high_24h: conversion::string_to_price(&okx_ticker.high_24h),
        low_24h: conversion::string_to_price(&okx_ticker.low_24h),
        volume_24h: conversion::string_to_quantity(&okx_ticker.vol_24h),
        quote_volume_24h: conversion::string_to_quantity(&okx_ticker.vol_ccy_24h),
        price_change_24h,
        price_change_percent_24h,
        timestamp,
        exchange: "okx".to_string(),
    })
}

/// Convert OKX order book to core order book type
pub fn convert_okx_order_book(
    okx_order_book: okx_types::OkxOrderBook,
    symbol: &str,
) -> Result<OrderBook, String> {
    let symbol = conversion::string_to_symbol(symbol);

    // Parse timestamp
    let timestamp = okx_order_book
        .ts
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse timestamp: {}", e))?;

    // Convert bids and asks
    let mut bids = Vec::new();
    for bid_array in okx_order_book.bids {
        if bid_array.len() >= 2 {
            let price = conversion::string_to_price(&bid_array[0]);
            let quantity = conversion::string_to_quantity(&bid_array[1]);
            bids.push(OrderBookEntry { price, quantity });
        }
    }

    let mut asks = Vec::new();
    for ask_array in okx_order_book.asks {
        if ask_array.len() >= 2 {
            let price = conversion::string_to_price(&ask_array[0]);
            let quantity = conversion::string_to_quantity(&ask_array[1]);
            asks.push(OrderBookEntry { price, quantity });
        }
    }

    Ok(OrderBook {
        symbol,
        bids,
        asks,
        timestamp,
        exchange: "okx".to_string(),
    })
}

/// Convert OKX trade to core trade type
pub fn convert_okx_trade(okx_trade: okx_types::OkxTrade) -> Result<Trade, String> {
    let symbol = conversion::string_to_symbol(&okx_trade.inst_id);
    let price = conversion::string_to_price(&okx_trade.px);
    let quantity = conversion::string_to_quantity(&okx_trade.sz);

    // Parse timestamp
    let timestamp = okx_trade
        .ts
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse timestamp: {}", e))?;

    // Convert side
    let side = match okx_trade.side.as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        _ => return Err(format!("Unknown trade side: {}", okx_trade.side)),
    };

    Ok(Trade {
        symbol,
        price,
        quantity,
        side,
        timestamp,
        trade_id: Some(okx_trade.trade_id),
        exchange: "okx".to_string(),
    })
}

/// Convert OKX kline to core kline type
pub fn convert_okx_kline(okx_kline: okx_types::OkxKline, symbol: &str) -> Result<Kline, String> {
    let symbol = conversion::string_to_symbol(symbol);

    // Parse timestamp
    let timestamp = okx_kline
        .ts
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse timestamp: {}", e))?;

    Ok(Kline {
        symbol,
        open: conversion::string_to_price(&okx_kline.o),
        high: conversion::string_to_price(&okx_kline.h),
        low: conversion::string_to_price(&okx_kline.l),
        close: conversion::string_to_price(&okx_kline.c),
        volume: conversion::string_to_quantity(&okx_kline.vol),
        timestamp,
        exchange: "okx".to_string(),
    })
}

/// Convert core order side to OKX order side
pub fn convert_order_side_to_okx(side: OrderSide) -> String {
    match side {
        OrderSide::Buy => "buy".to_string(),
        OrderSide::Sell => "sell".to_string(),
    }
}

/// Convert core order type to OKX order type
pub fn convert_order_type_to_okx(
    order_type: OrderType,
    time_in_force: Option<TimeInForce>,
) -> String {
    match order_type {
        OrderType::Market => "market".to_string(),
        OrderType::Limit => {
            // Handle time-in-force for limit orders
            match time_in_force {
                Some(TimeInForce::IOC) => "ioc".to_string(),
                Some(TimeInForce::FOK) => "fok".to_string(),
                Some(TimeInForce::PostOnly) => "post_only".to_string(),
                _ => "limit".to_string(),
            }
        }
        OrderType::StopLoss => "conditional".to_string(),
        OrderType::TakeProfit => "conditional".to_string(),
        OrderType::LimitMaker => "post_only".to_string(),
    }
}

/// Convert OKX order state to simplified status
pub fn convert_okx_order_state(state: &str) -> String {
    match state {
        "live" => "NEW".to_string(),
        "partially_filled" => "PARTIALLY_FILLED".to_string(),
        "filled" => "FILLED".to_string(),
        "canceled" => "CANCELED".to_string(),
        _ => state.to_uppercase(),
    }
}

/// Convert symbol to OKX instrument ID format
pub fn convert_symbol_to_okx_inst_id(symbol: &Symbol) -> String {
    format!("{}-{}", symbol.base(), symbol.quote())
}

/// Helper function to convert OKX WebSocket ticker message
pub fn convert_okx_ws_ticker(data: &Value, inst_id: &str) -> Result<Ticker, String> {
    // Extract ticker data from WebSocket message
    if let Some(ticker_array) = data.as_array().and_then(|arr| arr.first()) {
        let ticker_obj = ticker_array
            .as_object()
            .ok_or("Invalid ticker object structure")?;

        let okx_ticker = okx_types::OkxTicker {
            inst_type: "SPOT".to_string(),
            inst_id: inst_id.to_string(),
            last: ticker_obj
                .get("last")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            last_sz: ticker_obj
                .get("lastSz")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            ask_px: ticker_obj
                .get("askPx")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            ask_sz: ticker_obj
                .get("askSz")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            bid_px: ticker_obj
                .get("bidPx")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            bid_sz: ticker_obj
                .get("bidSz")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            open_24h: ticker_obj
                .get("open24h")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            high_24h: ticker_obj
                .get("high24h")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            low_24h: ticker_obj
                .get("low24h")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            vol_ccy_24h: ticker_obj
                .get("volCcy24h")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            vol_24h: ticker_obj
                .get("vol24h")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            ts: ticker_obj
                .get("ts")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            sod_utc0: ticker_obj
                .get("sodUtc0")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
            sod_utc8: ticker_obj
                .get("sodUtc8")
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string(),
        };

        convert_okx_ticker(okx_ticker)
    } else {
        Err("Invalid ticker data format".to_string())
    }
}

/// Helper function to convert OKX WebSocket order book message
pub fn convert_okx_ws_order_book(data: &Value, inst_id: &str) -> Result<OrderBook, String> {
    // Extract order book data from WebSocket message
    if let Some(book_array) = data.as_array().and_then(|arr| arr.first()) {
        let book_obj = book_array
            .as_object()
            .ok_or("Invalid order book object structure")?;

        // Extract bids and asks arrays
        let bids_value = book_obj.get("bids").ok_or("Missing bids field")?;
        let asks_value = book_obj.get("asks").ok_or("Missing asks field")?;

        let bids = bids_value
            .as_array()
            .ok_or("Bids is not an array")?
            .iter()
            .filter_map(|bid| {
                bid.as_array().and_then(|arr| {
                    if arr.len() >= 2 {
                        Some(vec![
                            arr[0].as_str()?.to_string(),
                            arr[1].as_str()?.to_string(),
                        ])
                    } else {
                        None
                    }
                })
            })
            .collect();

        let asks = asks_value
            .as_array()
            .ok_or("Asks is not an array")?
            .iter()
            .filter_map(|ask| {
                ask.as_array().and_then(|arr| {
                    if arr.len() >= 2 {
                        Some(vec![
                            arr[0].as_str()?.to_string(),
                            arr[1].as_str()?.to_string(),
                        ])
                    } else {
                        None
                    }
                })
            })
            .collect();

        let ts = book_obj
            .get("ts")
            .and_then(|v| v.as_str())
            .unwrap_or("0")
            .to_string();

        let okx_order_book = okx_types::OkxOrderBook { asks, bids, ts };

        convert_okx_order_book(okx_order_book, inst_id)
    } else {
        Err("Invalid order book data format".to_string())
    }
}

/// Helper function to convert OKX WebSocket trade message
pub fn convert_okx_ws_trade(data: &Value, inst_id: &str) -> Result<Vec<Trade>, String> {
    // Extract trade data from WebSocket message
    if let Some(trades_array) = data.as_array() {
        let mut trades = Vec::new();

        for trade_value in trades_array {
            if let Some(trade_obj) = trade_value.as_object() {
                let okx_trade = okx_types::OkxTrade {
                    inst_id: inst_id.to_string(),
                    trade_id: trade_obj
                        .get("tradeId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    px: trade_obj
                        .get("px")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    sz: trade_obj
                        .get("sz")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    side: trade_obj
                        .get("side")
                        .and_then(|v| v.as_str())
                        .unwrap_or("buy")
                        .to_string(),
                    ts: trade_obj
                        .get("ts")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    count: trade_obj
                        .get("count")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                };

                trades.push(convert_okx_trade(okx_trade)?);
            }
        }

        Ok(trades)
    } else {
        Err("Invalid trade data format".to_string())
    }
}
