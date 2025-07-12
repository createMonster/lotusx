use crate::core::types::{
    conversion::{
        string_to_decimal, string_to_price, string_to_quantity, string_to_symbol, string_to_volume,
    },
    Balance, Kline, Market, MarketDataType, OrderBook, OrderBookEntry, Position, PositionSide,
    Ticker, Trade,
};
use crate::exchanges::binance_perp::types::{
    BinancePerpBalance, BinancePerpMarket, BinancePerpPosition, BinancePerpRestKline,
    BinancePerpWebSocketKline, BinancePerpWebSocketOrderBook, BinancePerpWebSocketTicker,
    BinancePerpWebSocketTrade,
};
use rust_decimal::Decimal;
use tracing::warn;

/// Convert Binance Perpetual market to core Market type
pub fn convert_binance_perp_market(binance_market: BinancePerpMarket) -> Market {
    Market {
        symbol: string_to_symbol(&binance_market.symbol),
        status: binance_market.status,
        base_precision: binance_market.base_asset_precision,
        quote_precision: binance_market.quote_precision,
        min_qty: binance_market
            .filters
            .iter()
            .find(|f| f.filter_type == "LOT_SIZE")
            .and_then(|f| f.min_qty.as_ref())
            .map(|s| string_to_quantity(s)),
        max_qty: binance_market
            .filters
            .iter()
            .find(|f| f.filter_type == "LOT_SIZE")
            .and_then(|f| f.max_qty.as_ref())
            .map(|s| string_to_quantity(s)),
        min_price: binance_market
            .filters
            .iter()
            .find(|f| f.filter_type == "PRICE_FILTER")
            .and_then(|f| f.min_price.as_ref())
            .map(|s| string_to_price(s)),
        max_price: binance_market
            .filters
            .iter()
            .find(|f| f.filter_type == "PRICE_FILTER")
            .and_then(|f| f.max_price.as_ref())
            .map(|s| string_to_price(s)),
    }
}

/// Convert Binance Perpetual balance to core Balance type
pub fn convert_binance_perp_balance(binance_balance: &BinancePerpBalance) -> Balance {
    let free = string_to_quantity(&binance_balance.available_balance);
    let total = string_to_quantity(&binance_balance.balance);
    let locked = crate::core::types::Quantity::new(total.value() - free.value());

    Balance {
        asset: binance_balance.asset.clone(),
        free,
        locked,
    }
}

/// Convert Binance Perpetual position to core Position type
pub fn convert_binance_perp_position(binance_position: &BinancePerpPosition) -> Position {
    let position_amount = string_to_quantity(&binance_position.position_amt);
    let position_side = match position_amount.value().cmp(&Decimal::ZERO) {
        std::cmp::Ordering::Greater => PositionSide::Long,
        std::cmp::Ordering::Less => PositionSide::Short,
        std::cmp::Ordering::Equal => PositionSide::Both,
    };

    Position {
        symbol: string_to_symbol(&binance_position.symbol),
        position_side,
        entry_price: string_to_price(&binance_position.entry_price),
        position_amount,
        unrealized_pnl: string_to_decimal(&binance_position.un_realized_pnl),
        liquidation_price: Some(string_to_price(&binance_position.liquidation_price)),
        leverage: string_to_decimal(&binance_position.leverage),
    }
}

/// Convert Binance Perpetual REST kline to core Kline type
pub fn convert_binance_perp_rest_kline(binance_kline: &BinancePerpRestKline) -> Kline {
    Kline {
        symbol: string_to_symbol(""), // Symbol should be set by caller
        open_time: binance_kline.open_time,
        close_time: binance_kline.close_time,
        interval: String::new(), // Interval should be set by caller
        open_price: string_to_price(&binance_kline.open_price),
        high_price: string_to_price(&binance_kline.high_price),
        low_price: string_to_price(&binance_kline.low_price),
        close_price: string_to_price(&binance_kline.close_price),
        volume: string_to_volume(&binance_kline.volume),
        number_of_trades: binance_kline.number_of_trades,
        final_bar: true, // REST klines are always final
    }
}

/// Parse WebSocket message and convert to core `MarketDataType`
pub fn parse_websocket_message(message: serde_json::Value) -> Option<MarketDataType> {
    let message_str = message.to_string();

    // Try to parse as different WebSocket message types
    if let Ok(ticker) = serde_json::from_str::<BinancePerpWebSocketTicker>(&message_str) {
        Some(MarketDataType::Ticker(Ticker {
            symbol: string_to_symbol(&ticker.symbol),
            price: string_to_price(&ticker.price),
            price_change: string_to_price(&ticker.price_change),
            price_change_percent: string_to_decimal(&ticker.price_change_percent),
            high_price: string_to_price(&ticker.high_price),
            low_price: string_to_price(&ticker.low_price),
            volume: string_to_volume(&ticker.volume),
            quote_volume: string_to_volume(&ticker.quote_volume),
            open_time: ticker.open_time,
            close_time: ticker.close_time,
            count: ticker.count,
        }))
    } else if let Ok(order_book) =
        serde_json::from_str::<BinancePerpWebSocketOrderBook>(&message_str)
    {
        Some(MarketDataType::OrderBook(OrderBook {
            symbol: string_to_symbol(&order_book.symbol),
            bids: order_book
                .bids
                .iter()
                .map(|[price, quantity]| OrderBookEntry {
                    price: string_to_price(price),
                    quantity: string_to_quantity(quantity),
                })
                .collect(),
            asks: order_book
                .asks
                .iter()
                .map(|[price, quantity]| OrderBookEntry {
                    price: string_to_price(price),
                    quantity: string_to_quantity(quantity),
                })
                .collect(),
            last_update_id: order_book.final_update_id,
        }))
    } else if let Ok(trade) = serde_json::from_str::<BinancePerpWebSocketTrade>(&message_str) {
        Some(MarketDataType::Trade(Trade {
            symbol: string_to_symbol(&trade.symbol),
            id: trade.id,
            price: string_to_price(&trade.price),
            quantity: string_to_quantity(&trade.quantity),
            time: trade.time,
            is_buyer_maker: trade.is_buyer_maker,
        }))
    } else if let Ok(kline) = serde_json::from_str::<BinancePerpWebSocketKline>(&message_str) {
        Some(MarketDataType::Kline(Kline {
            symbol: string_to_symbol(&kline.symbol),
            open_time: kline.kline.open_time,
            close_time: kline.kline.close_time,
            interval: kline.kline.interval,
            open_price: string_to_price(&kline.kline.open_price),
            high_price: string_to_price(&kline.kline.high_price),
            low_price: string_to_price(&kline.kline.low_price),
            close_price: string_to_price(&kline.kline.close_price),
            volume: string_to_volume(&kline.kline.volume),
            number_of_trades: kline.kline.number_of_trades,
            final_bar: kline.kline.final_bar,
        }))
    } else {
        warn!("Failed to parse WebSocket message: {}", message_str);
        None
    }
}
