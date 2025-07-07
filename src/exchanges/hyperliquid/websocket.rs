use super::client::HyperliquidClient;
use crate::core::errors::ExchangeError;
use crate::core::types::{
    conversion, Kline, MarketDataType, OrderBook, OrderBookEntry, SubscriptionType, Ticker, Trade,
    WebSocketConfig,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Public function to handle WebSocket market data subscription
/// This is called by the `MarketDataSource` trait implementation
pub async fn subscribe_market_data_impl(
    client: &HyperliquidClient,
    symbols: Vec<String>,
    subscription_types: Vec<SubscriptionType>,
    config: Option<WebSocketConfig>,
) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
    let ws_url = client.get_websocket_url();
    let (ws_stream, _) = connect_async(&ws_url)
        .await
        .map_err(|e| ExchangeError::NetworkError(format!("WebSocket connection failed: {}", e)))?;

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, rx) = mpsc::channel(1000);

    // Handle auto-reconnection if configured
    let auto_reconnect = config.as_ref().map_or(true, |c| c.auto_reconnect);
    let _max_reconnect_attempts = config
        .as_ref()
        .and_then(|c| c.max_reconnect_attempts)
        .unwrap_or(5);

    // Send all subscriptions using a flattened approach
    send_subscriptions(&mut ws_sender, &symbols, &subscription_types).await?;

    // Spawn task to handle incoming messages
    let tx_clone = tx.clone();
    let symbols_clone = symbols.clone();

    tokio::spawn(async move {
        let mut heartbeat_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                msg = ws_receiver.next() => {
                    if handle_websocket_message(msg, &tx_clone, &symbols_clone, auto_reconnect).await {
                        break;
                    }
                }
                // Send periodic heartbeat
                _ = heartbeat_interval.tick() => {
                    if send_heartbeat(&mut ws_sender).await {
                        break;
                    }
                }
            }
        }
    });

    Ok(rx)
}

// Helper function to send all WebSocket subscriptions
async fn send_subscriptions(
    ws_sender: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    symbols: &[String],
    subscription_types: &[SubscriptionType],
) -> Result<(), ExchangeError> {
    // Create all subscription combinations using iterator chains to avoid nested loops
    let subscriptions: Vec<_> = symbols
        .iter()
        .flat_map(|symbol| {
            subscription_types
                .iter()
                .map(|sub_type| (symbol.as_str(), sub_type))
        })
        .map(|(symbol, sub_type)| create_subscription_message(symbol, sub_type))
        .collect();

    // Send all subscriptions
    for subscription in subscriptions {
        let msg = Message::Text(subscription.to_string());
        ws_sender.send(msg).await.map_err(|e| {
            ExchangeError::NetworkError(format!("Failed to send subscription: {}", e))
        })?;
    }

    Ok(())
}

// Helper function to create subscription message
fn create_subscription_message(symbol: &str, sub_type: &SubscriptionType) -> Value {
    match sub_type {
        SubscriptionType::Ticker => {
            json!({
                "method": "subscribe",
                "subscription": {
                    "type": "allMids"
                }
            })
        }
        SubscriptionType::OrderBook { depth: _ } => {
            json!({
                "method": "subscribe",
                "subscription": {
                    "type": "l2Book",
                    "coin": symbol
                }
            })
        }
        SubscriptionType::Trades => {
            json!({
                "method": "subscribe",
                "subscription": {
                    "type": "trades",
                    "coin": symbol
                }
            })
        }
        SubscriptionType::Klines { interval } => {
            json!({
                "method": "subscribe",
                "subscription": {
                    "type": "candle",
                    "coin": symbol,
                    "interval": interval.to_hyperliquid_format()
                }
            })
        }
    }
}

// Helper function to handle WebSocket messages
async fn handle_websocket_message(
    msg: Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
    tx: &mpsc::Sender<MarketDataType>,
    symbols: &[String],
    auto_reconnect: bool,
) -> bool {
    match msg {
        Some(Ok(Message::Text(text))) => process_text_message(&text, tx, symbols).await,
        Some(Ok(Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Frame(_))) => {
            // Handle binary, ping, pong, and frame messages - all return false to continue
            false
        }
        Some(Ok(Message::Close(_))) => {
            tracing::info!("WebSocket connection closed by server");
            true
        }
        Some(Err(e)) => {
            tracing::error!("WebSocket error: {}", e);
            if auto_reconnect {
                tracing::info!("Attempting to reconnect...");
            }
            true
        }
        None => {
            tracing::info!("WebSocket stream ended");
            true
        }
    }
}

// Helper function to process text messages
async fn process_text_message(
    text: &str,
    tx: &mpsc::Sender<MarketDataType>,
    symbols: &[String],
) -> bool {
    let Ok(parsed) = serde_json::from_str::<Value>(text) else {
        return false;
    };

    let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) else {
        return false;
    };

    let Some(data) = parsed.get("data") else {
        return false;
    };

    // Process data for each subscribed symbol using iterator to avoid nested loops
    for symbol in symbols {
        if let Some(market_data) = convert_ws_data_static(channel, data, symbol) {
            if tx.send(market_data).await.is_err() {
                tracing::warn!("Receiver dropped, stopping WebSocket task");
                return true;
            }
        }
    }

    false
}

// Helper function to send heartbeat
async fn send_heartbeat(
    ws_sender: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
) -> bool {
    let ping_msg = Message::Ping(vec![]);
    if let Err(e) = ws_sender.send(ping_msg).await {
        tracing::error!("Failed to send ping: {}", e);
        return true;
    }
    false
}

// Static version of convert_ws_data for use in async task
fn convert_ws_data_static(channel: &str, data: &Value, symbol: &str) -> Option<MarketDataType> {
    match channel {
        "allMids" => convert_all_mids_data(data, symbol),
        "l2Book" => convert_orderbook_data(data, symbol),
        "trades" => convert_trades_data(data, symbol),
        "candle" => convert_candle_data(data, symbol),
        _ => None,
    }
}

// Helper function to convert allMids data
fn convert_all_mids_data(data: &Value, symbol: &str) -> Option<MarketDataType> {
    let mids = data.get("mids")?.as_object()?;
    let price = mids.get(symbol)?.as_str()?;

    Some(MarketDataType::Ticker(Ticker {
        symbol: conversion::string_to_symbol(symbol),
        price: conversion::string_to_price(price),
        price_change: conversion::string_to_price("0"),
        price_change_percent: conversion::string_to_decimal("0"),
        high_price: conversion::string_to_price("0"),
        low_price: conversion::string_to_price("0"),
        volume: conversion::string_to_volume("0"),
        quote_volume: conversion::string_to_volume("0"),
        open_time: 0,
        close_time: 0,
        count: 0,
    }))
}

// Helper function to convert orderbook data
fn convert_orderbook_data(data: &Value, symbol: &str) -> Option<MarketDataType> {
    let coin = data.get("coin")?.as_str()?;
    let levels = data.get("levels")?.as_array()?;
    let time = data.get("time")?.as_i64()?;

    if coin != symbol || levels.len() < 2 {
        return None;
    }

    let bids = extract_order_book_levels(levels.first()?)?;
    let asks = extract_order_book_levels(levels.get(1)?)?;

    Some(MarketDataType::OrderBook(OrderBook {
        symbol: conversion::string_to_symbol(coin),
        bids,
        asks,
        last_update_id: time,
    }))
}

// Helper function to extract order book levels
fn extract_order_book_levels(level_data: &Value) -> Option<Vec<OrderBookEntry>> {
    let levels = level_data.as_array()?;
    let mut entries = Vec::new();

    for level in levels {
        let px = level.get("px")?.as_str()?;
        let sz = level.get("sz")?.as_str()?;
        entries.push(OrderBookEntry {
            price: conversion::string_to_price(px),
            quantity: conversion::string_to_quantity(sz),
        });
    }

    Some(entries)
}

// Helper function to convert trades data
fn convert_trades_data(data: &Value, symbol: &str) -> Option<MarketDataType> {
    let trades = data.as_array()?;

    for trade in trades {
        let coin = trade.get("coin")?.as_str()?;
        if coin != symbol {
            continue;
        }

        let side = trade.get("side")?.as_str()?;
        let px = trade.get("px")?.as_str()?;
        let sz = trade.get("sz")?.as_str()?;
        let time = trade.get("time")?.as_i64()?;
        let tid = trade.get("tid")?.as_i64()?;

        return Some(MarketDataType::Trade(Trade {
            symbol: conversion::string_to_symbol(coin),
            id: tid,
            price: conversion::string_to_price(px),
            quantity: conversion::string_to_quantity(sz),
            time,
            is_buyer_maker: side == "B",
        }));
    }

    None
}

// Helper function to convert candle data
fn convert_candle_data(data: &Value, symbol: &str) -> Option<MarketDataType> {
    let candles = data.as_array()?;

    for candle in candles {
        let coin = candle.get("s")?.as_str()?;
        if coin != symbol {
            continue;
        }

        let open_time = candle.get("t")?.as_i64()?;
        let close_time = candle.get("T")?.as_i64()?;
        let open = candle.get("o")?.as_f64()?;
        let close = candle.get("c")?.as_f64()?;
        let high = candle.get("h")?.as_f64()?;
        let low = candle.get("l")?.as_f64()?;
        let volume = candle.get("v")?.as_f64()?;

        return Some(MarketDataType::Kline(Kline {
            symbol: conversion::string_to_symbol(coin),
            open_time,
            close_time,
            interval: "1m".to_string(),
            open_price: conversion::string_to_price(&open.to_string()),
            high_price: conversion::string_to_price(&high.to_string()),
            low_price: conversion::string_to_price(&low.to_string()),
            close_price: conversion::string_to_price(&close.to_string()),
            volume: conversion::string_to_volume(&volume.to_string()),
            number_of_trades: candle.get("n").and_then(|n| n.as_i64()).unwrap_or(0),
            final_bar: true,
        }));
    }

    None
}
