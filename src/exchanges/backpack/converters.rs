use crate::core::types::{
    Balance, Kline, Market, MarketDataType, OrderBook, OrderBookEntry, Position, PositionSide,
    Symbol, Ticker, Trade,
};
use crate::exchanges::backpack::types::{
    BackpackBalance, BackpackMarket, BackpackOrderBook, BackpackPosition, BackpackRestKline,
    BackpackTicker, BackpackTrade, BackpackWebSocketKline, BackpackWebSocketOrderBook,
    BackpackWebSocketTicker, BackpackWebSocketTrade,
};

/// Convert Backpack market to core Market type
pub fn convert_market(backpack_market: BackpackMarket) -> Market {
    Market {
        symbol: Symbol {
            base: backpack_market.base_asset,
            quote: backpack_market.quote_asset,
            symbol: backpack_market.symbol,
        },
        status: backpack_market.status,
        base_precision: backpack_market.base_precision,
        quote_precision: backpack_market.quote_precision,
        min_qty: Some(backpack_market.min_qty),
        max_qty: Some(backpack_market.max_qty),
        min_price: Some(backpack_market.min_price),
        max_price: Some(backpack_market.max_price),
    }
}

/// Convert Backpack balance to core Balance type
pub fn convert_balance(backpack_balance: BackpackBalance) -> Balance {
    Balance {
        asset: backpack_balance.asset,
        free: backpack_balance.free,
        locked: backpack_balance.locked,
    }
}

/// Convert Backpack position to core Position type
pub fn convert_position(backpack_position: BackpackPosition) -> Position {
    Position {
        symbol: backpack_position.symbol,
        position_side: match backpack_position.side.as_str() {
            "LONG" => PositionSide::Long,
            "SHORT" => PositionSide::Short,
            _ => PositionSide::Both,
        },
        entry_price: backpack_position.entry_price,
        position_amount: backpack_position.size,
        unrealized_pnl: backpack_position.unrealized_pnl,
        liquidation_price: Some(backpack_position.liquidation_price),
        leverage: backpack_position.leverage,
    }
}

/// Convert Backpack ticker to core Ticker type
pub fn convert_ticker(backpack_ticker: BackpackTicker) -> Ticker {
    Ticker {
        symbol: backpack_ticker.symbol,
        price: backpack_ticker.price,
        price_change: backpack_ticker.price_change,
        price_change_percent: backpack_ticker.price_change_percent,
        high_price: backpack_ticker.high_price,
        low_price: backpack_ticker.low_price,
        volume: backpack_ticker.volume,
        quote_volume: backpack_ticker.quote_volume,
        open_time: backpack_ticker.open_time,
        close_time: backpack_ticker.close_time,
        count: backpack_ticker.count,
    }
}

/// Convert Backpack order book to core `OrderBook` type
pub fn convert_order_book(backpack_order_book: BackpackOrderBook) -> OrderBook {
    OrderBook {
        symbol: backpack_order_book.symbol,
        bids: backpack_order_book
            .bids
            .into_iter()
            .map(|b| OrderBookEntry {
                price: b.price,
                quantity: b.quantity,
            })
            .collect(),
        asks: backpack_order_book
            .asks
            .into_iter()
            .map(|a| OrderBookEntry {
                price: a.price,
                quantity: a.quantity,
            })
            .collect(),
        last_update_id: backpack_order_book.last_update_id,
    }
}

/// Convert Backpack trade to core Trade type
pub fn convert_trade(backpack_trade: BackpackTrade) -> Trade {
    Trade {
        symbol: String::new(), // Symbol not available in trade data
        id: backpack_trade.id,
        price: backpack_trade.price,
        quantity: backpack_trade.quantity,
        time: backpack_trade.time,
        is_buyer_maker: backpack_trade.is_buyer_maker,
    }
}

/// Convert Backpack REST kline to core Kline type
pub fn convert_rest_kline(
    backpack_kline: BackpackRestKline,
    symbol: String,
    interval: String,
) -> Kline {
    Kline {
        symbol,
        open_time: backpack_kline.open_time,
        close_time: backpack_kline.close_time,
        interval,
        open_price: backpack_kline.open,
        high_price: backpack_kline.high,
        low_price: backpack_kline.low,
        close_price: backpack_kline.close,
        volume: backpack_kline.volume,
        number_of_trades: backpack_kline.number_of_trades,
        final_bar: true, // Always true for historical data
    }
}

/// Convert Backpack WebSocket ticker to core Ticker type
pub fn convert_ws_ticker(backpack_ws_ticker: BackpackWebSocketTicker) -> Ticker {
    Ticker {
        symbol: backpack_ws_ticker.s,
        price: backpack_ws_ticker.c,
        price_change: "0".to_string(), // Not available in WebSocket
        price_change_percent: "0".to_string(), // Not available in WebSocket
        high_price: backpack_ws_ticker.h,
        low_price: backpack_ws_ticker.l,
        volume: backpack_ws_ticker.v,
        quote_volume: backpack_ws_ticker.V,
        open_time: 0, // Not available in WebSocket
        close_time: backpack_ws_ticker.E,
        count: backpack_ws_ticker.n,
    }
}

/// Convert Backpack WebSocket order book to core `OrderBook` type
pub fn convert_ws_order_book(backpack_ws_order_book: BackpackWebSocketOrderBook) -> OrderBook {
    OrderBook {
        symbol: backpack_ws_order_book.s,
        bids: backpack_ws_order_book
            .b
            .into_iter()
            .map(|b| OrderBookEntry {
                price: b[0].clone(),
                quantity: b[1].clone(),
            })
            .collect(),
        asks: backpack_ws_order_book
            .a
            .into_iter()
            .map(|a| OrderBookEntry {
                price: a[0].clone(),
                quantity: a[1].clone(),
            })
            .collect(),
        last_update_id: backpack_ws_order_book.u,
    }
}

/// Convert Backpack WebSocket trade to core Trade type
pub fn convert_ws_trade(backpack_ws_trade: BackpackWebSocketTrade) -> Trade {
    Trade {
        symbol: backpack_ws_trade.s,
        id: backpack_ws_trade.t,
        price: backpack_ws_trade.p,
        quantity: backpack_ws_trade.q,
        time: backpack_ws_trade.T,
        is_buyer_maker: backpack_ws_trade.m,
    }
}

/// Convert Backpack WebSocket kline to core Kline type
pub fn convert_ws_kline(backpack_ws_kline: BackpackWebSocketKline, interval: String) -> Kline {
    Kline {
        symbol: backpack_ws_kline.s,
        open_time: backpack_ws_kline.t,
        close_time: backpack_ws_kline.T,
        interval,
        open_price: backpack_ws_kline.o,
        high_price: backpack_ws_kline.h,
        low_price: backpack_ws_kline.l,
        close_price: backpack_ws_kline.c,
        volume: backpack_ws_kline.v,
        number_of_trades: backpack_ws_kline.n,
        final_bar: backpack_ws_kline.X,
    }
}

/// Convert Backpack WebSocket message to core `MarketDataType`
pub fn convert_ws_message(
    backpack_ws_message: crate::exchanges::backpack::types::BackpackWebSocketMessage,
) -> Option<MarketDataType> {
    match backpack_ws_message {
        crate::exchanges::backpack::types::BackpackWebSocketMessage::Ticker(ticker) => {
            Some(MarketDataType::Ticker(convert_ws_ticker(ticker)))
        }
        crate::exchanges::backpack::types::BackpackWebSocketMessage::OrderBook(orderbook) => {
            Some(MarketDataType::OrderBook(convert_ws_order_book(orderbook)))
        }
        crate::exchanges::backpack::types::BackpackWebSocketMessage::Trade(trade) => {
            Some(MarketDataType::Trade(convert_ws_trade(trade)))
        }
        crate::exchanges::backpack::types::BackpackWebSocketMessage::Kline(kline) => Some(
            MarketDataType::Kline(convert_ws_kline(kline, "1m".to_string())),
        ),
        _ => None, // Ignore other message types
    }
}
