use crate::core::errors::ExchangeError;
use crate::core::kernel::codec::WsCodec;
use crate::core::types::{
    conversion, Kline, KlineInterval, MarketDataType, OrderBook, OrderBookEntry, Ticker, Trade,
};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use tracing::warn;

/// Hyperliquid WebSocket message types
#[derive(Debug, Clone)]
pub enum HyperliquidWsMessage {
    Ticker(Ticker),
    OrderBook(OrderBook),
    Trade(Trade),
    Kline(Kline),
    Heartbeat,
    Unknown(String),
}

impl From<HyperliquidWsMessage> for MarketDataType {
    fn from(msg: HyperliquidWsMessage) -> Self {
        match msg {
            HyperliquidWsMessage::Ticker(ticker) => Self::Ticker(ticker),
            HyperliquidWsMessage::OrderBook(orderbook) => Self::OrderBook(orderbook),
            HyperliquidWsMessage::Trade(trade) => Self::Trade(trade),
            HyperliquidWsMessage::Kline(kline) => Self::Kline(kline),
            // For heartbeat and unknown messages, we'll create a dummy ticker
            HyperliquidWsMessage::Heartbeat | HyperliquidWsMessage::Unknown(_) => {
                Self::Ticker(Ticker {
                    symbol: conversion::string_to_symbol("HEARTBEAT"),
                    price: conversion::string_to_price("0"),
                    price_change: conversion::string_to_price("0"),
                    price_change_percent: conversion::string_to_decimal("0"),
                    high_price: conversion::string_to_price("0"),
                    low_price: conversion::string_to_price("0"),
                    volume: conversion::string_to_volume("0"),
                    quote_volume: conversion::string_to_volume("0"),
                    open_time: 0,
                    close_time: 0,
                    count: 0,
                })
            }
        }
    }
}

/// Hyperliquid WebSocket codec
pub struct HyperliquidCodec;

impl HyperliquidCodec {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HyperliquidCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl WsCodec for HyperliquidCodec {
    type Message = HyperliquidWsMessage;

    fn encode_subscription(&self, streams: &[impl AsRef<str>]) -> Result<Message, ExchangeError> {
        // For Hyperliquid, we need to parse the stream format to determine subscription type
        // Expected format: "symbol@type" or just "type" for global subscriptions
        let subscriptions: Vec<Value> = streams
            .iter()
            .map(|stream| {
                let stream_str = stream.as_ref();
                if stream_str.contains('@') {
                    let parts: Vec<&str> = stream_str.split('@').collect();
                    if parts.len() == 2 {
                        let symbol = parts[0];
                        let sub_type = parts[1];

                        match sub_type {
                            "ticker" => {
                                json!({
                                    "method": "subscribe",
                                    "subscription": {
                                        "type": "allMids"
                                    }
                                })
                            }
                            "orderbook" => {
                                json!({
                                    "method": "subscribe",
                                    "subscription": {
                                        "type": "l2Book",
                                        "coin": symbol
                                    }
                                })
                            }
                            "trade" => {
                                json!({
                                    "method": "subscribe",
                                    "subscription": {
                                        "type": "trades",
                                        "coin": symbol
                                    }
                                })
                            }
                            "kline" => {
                                json!({
                                    "method": "subscribe",
                                    "subscription": {
                                        "type": "candle",
                                        "coin": symbol,
                                        "interval": "1m"
                                    }
                                })
                            }
                            _ => {
                                json!({
                                    "method": "subscribe",
                                    "subscription": {
                                        "type": sub_type,
                                        "coin": symbol
                                    }
                                })
                            }
                        }
                    } else {
                        // Invalid format, create a generic subscription
                        json!({
                            "method": "subscribe",
                            "subscription": {
                                "type": stream_str
                            }
                        })
                    }
                } else {
                    // Global subscription without symbol
                    json!({
                        "method": "subscribe",
                        "subscription": {
                            "type": stream_str
                        }
                    })
                }
            })
            .collect();

        // For now, just send the first subscription
        // TODO: Handle multiple subscriptions properly
        if let Some(subscription) = subscriptions.first() {
            let msg_text = serde_json::to_string(subscription).map_err(ExchangeError::JsonError)?;
            Ok(Message::Text(msg_text))
        } else {
            Err(ExchangeError::InvalidParameters(
                "No valid streams provided".to_string(),
            ))
        }
    }

    fn encode_unsubscription(&self, streams: &[impl AsRef<str>]) -> Result<Message, ExchangeError> {
        // Similar to subscription but with "unsubscribe" method
        let unsubscriptions: Vec<Value> = streams
            .iter()
            .map(|stream| {
                let stream_str = stream.as_ref();
                if stream_str.contains('@') {
                    let parts: Vec<&str> = stream_str.split('@').collect();
                    if parts.len() == 2 {
                        let symbol = parts[0];
                        let sub_type = parts[1];

                        match sub_type {
                            "ticker" => {
                                json!({
                                    "method": "unsubscribe",
                                    "subscription": {
                                        "type": "allMids"
                                    }
                                })
                            }
                            "orderbook" => {
                                json!({
                                    "method": "unsubscribe",
                                    "subscription": {
                                        "type": "l2Book",
                                        "coin": symbol
                                    }
                                })
                            }
                            "trade" => {
                                json!({
                                    "method": "unsubscribe",
                                    "subscription": {
                                        "type": "trades",
                                        "coin": symbol
                                    }
                                })
                            }
                            "kline" => {
                                json!({
                                    "method": "unsubscribe",
                                    "subscription": {
                                        "type": "candle",
                                        "coin": symbol,
                                        "interval": "1m"
                                    }
                                })
                            }
                            _ => {
                                json!({
                                    "method": "unsubscribe",
                                    "subscription": {
                                        "type": sub_type,
                                        "coin": symbol
                                    }
                                })
                            }
                        }
                    } else {
                        json!({
                            "method": "unsubscribe",
                            "subscription": {
                                "type": stream_str
                            }
                        })
                    }
                } else {
                    json!({
                        "method": "unsubscribe",
                        "subscription": {
                            "type": stream_str
                        }
                    })
                }
            })
            .collect();

        if let Some(unsubscription) = unsubscriptions.first() {
            let msg_text =
                serde_json::to_string(unsubscription).map_err(ExchangeError::JsonError)?;
            Ok(Message::Text(msg_text))
        } else {
            Err(ExchangeError::InvalidParameters(
                "No valid streams provided".to_string(),
            ))
        }
    }

    fn decode_message(&self, msg: Message) -> Result<Option<Self::Message>, ExchangeError> {
        match msg {
            Message::Text(text) => {
                let parsed: Value =
                    serde_json::from_str(&text).map_err(ExchangeError::JsonError)?;

                // Check if it's a heartbeat or system message
                if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                    match channel {
                        "pong" => return Ok(Some(HyperliquidWsMessage::Heartbeat)),
                        "subscriptionResponse" => {
                            // This is a subscription confirmation message, ignore it
                            return Ok(None);
                        }
                        _ => {} // Continue processing
                    }
                }

                // Process market data messages
                if let Some(data) = parsed.get("data") {
                    if let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) {
                        match channel {
                            "allMids" => {
                                if let Some(ticker) = self.convert_ticker_data(data, "global") {
                                    return Ok(Some(HyperliquidWsMessage::Ticker(ticker)));
                                }
                            }
                            "l2Book" => {
                                if let Some(symbol) = data.get("coin").and_then(|c| c.as_str()) {
                                    if let Some(orderbook) =
                                        self.convert_orderbook_data(data, symbol)
                                    {
                                        return Ok(Some(HyperliquidWsMessage::OrderBook(
                                            orderbook,
                                        )));
                                    }
                                }
                            }
                            "trades" => {
                                if let Some(symbol) = data.get("coin").and_then(|c| c.as_str()) {
                                    if let Some(trade) = self.convert_trade_data(data, symbol) {
                                        return Ok(Some(HyperliquidWsMessage::Trade(trade)));
                                    }
                                }
                            }
                            "candle" => {
                                if let Some(symbol) = data.get("coin").and_then(|c| c.as_str()) {
                                    if let Some(kline) = self.convert_kline_data(data, symbol) {
                                        return Ok(Some(HyperliquidWsMessage::Kline(kline)));
                                    }
                                }
                            }
                            _ => {
                                warn!("Unknown channel: {}", channel);
                                return Ok(Some(HyperliquidWsMessage::Unknown(text)));
                            }
                        }
                    }
                }

                Ok(Some(HyperliquidWsMessage::Unknown(text)))
            }
            Message::Binary(_) => {
                // Hyperliquid doesn't typically use binary messages
                Ok(None)
            }
            Message::Ping(_) | Message::Pong(_) => Ok(Some(HyperliquidWsMessage::Heartbeat)),
            Message::Close(_) | Message::Frame(_) => Ok(None),
        }
    }
}

impl HyperliquidCodec {
    fn convert_ticker_data(&self, data: &Value, _symbol: &str) -> Option<Ticker> {
        // Implementation for ticker data conversion
        if let Some(mids) = data.as_object() {
            for (sym, price) in mids {
                if let Some(price_str) = price.as_str() {
                    if let Ok(_price_f64) = price_str.parse::<f64>() {
                        return Some(Ticker {
                            symbol: conversion::string_to_symbol(sym),
                            price: conversion::string_to_price(price_str),
                            price_change: conversion::string_to_price("0"),
                            price_change_percent: conversion::string_to_decimal("0"),
                            high_price: conversion::string_to_price(price_str),
                            low_price: conversion::string_to_price(price_str),
                            volume: conversion::string_to_volume("0"),
                            quote_volume: conversion::string_to_volume("0"),
                            open_time: chrono::Utc::now().timestamp_millis(),
                            close_time: chrono::Utc::now().timestamp_millis(),
                            count: 1,
                        });
                    }
                }
            }
        }
        None
    }

    fn convert_orderbook_data(&self, data: &Value, symbol: &str) -> Option<OrderBook> {
        let levels = data.get("levels").and_then(|l| l.as_array());
        if let Some(levels) = levels {
            let mut bids = Vec::new();
            let mut asks = Vec::new();

            for level in levels {
                if let Some(level_data) = level.as_array() {
                    if level_data.len() >= 3 {
                        let price = level_data[0]
                            .get("px")
                            .and_then(|p| p.as_str())
                            .and_then(|p| p.parse::<f64>().ok());
                        let quantity = level_data[0]
                            .get("sz")
                            .and_then(|s| s.as_str())
                            .and_then(|s| s.parse::<f64>().ok());
                        let side = level_data[0].get("side").and_then(|s| s.as_str());

                        if let (Some(price), Some(quantity), Some(side)) = (price, quantity, side) {
                            let entry = OrderBookEntry {
                                price: conversion::string_to_price(&price.to_string()),
                                quantity: conversion::string_to_quantity(&quantity.to_string()),
                            };

                            if side == "B" {
                                bids.push(entry);
                            } else if side == "A" {
                                asks.push(entry);
                            }
                        }
                    }
                }
            }

            return Some(OrderBook {
                symbol: conversion::string_to_symbol(symbol),
                bids,
                asks,
                last_update_id: chrono::Utc::now().timestamp_millis(),
            });
        }
        None
    }

    fn convert_trade_data(&self, data: &Value, symbol: &str) -> Option<Trade> {
        // Implementation for trade data conversion
        if let Some(trades) = data.as_array() {
            for trade in trades {
                if let (Some(price), Some(quantity), Some(timestamp)) = (
                    trade
                        .get("px")
                        .and_then(|p| p.as_str())
                        .and_then(|p| p.parse::<f64>().ok()),
                    trade
                        .get("sz")
                        .and_then(|s| s.as_str())
                        .and_then(|s| s.parse::<f64>().ok()),
                    trade.get("time").and_then(|t| t.as_i64()),
                ) {
                    let side = trade
                        .get("side")
                        .and_then(|s| s.as_str())
                        .unwrap_or("unknown");

                    return Some(Trade {
                        symbol: conversion::string_to_symbol(symbol),
                        id: trade.get("tid").and_then(|t| t.as_i64()).unwrap_or(0),
                        price: conversion::string_to_price(&price.to_string()),
                        quantity: conversion::string_to_quantity(&quantity.to_string()),
                        time: timestamp,
                        is_buyer_maker: side == "B",
                    });
                }
            }
        }
        None
    }

    fn convert_kline_data(&self, data: &Value, symbol: &str) -> Option<Kline> {
        // Implementation for kline/candle data conversion
        if let (Some(open), Some(high), Some(low), Some(close), Some(volume), Some(timestamp)) = (
            data.get("o")
                .and_then(|o| o.as_str())
                .and_then(|o| o.parse::<f64>().ok()),
            data.get("h")
                .and_then(|h| h.as_str())
                .and_then(|h| h.parse::<f64>().ok()),
            data.get("l")
                .and_then(|l| l.as_str())
                .and_then(|l| l.parse::<f64>().ok()),
            data.get("c")
                .and_then(|c| c.as_str())
                .and_then(|c| c.parse::<f64>().ok()),
            data.get("v")
                .and_then(|v| v.as_str())
                .and_then(|v| v.parse::<f64>().ok()),
            data.get("t").and_then(|t| t.as_i64()),
        ) {
            return Some(Kline {
                symbol: conversion::string_to_symbol(symbol),
                open_time: timestamp,
                close_time: timestamp,
                interval: KlineInterval::Minutes1.to_binance_format(),
                open_price: conversion::string_to_price(&open.to_string()),
                high_price: conversion::string_to_price(&high.to_string()),
                low_price: conversion::string_to_price(&low.to_string()),
                close_price: conversion::string_to_price(&close.to_string()),
                volume: conversion::string_to_volume(&volume.to_string()),
                number_of_trades: 1,
                final_bar: true,
            });
        }
        None
    }
}
