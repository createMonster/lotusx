use super::client::ParadexConnector;
use crate::core::errors::ExchangeError;
use crate::core::types::{
    conversion, Kline, MarketDataType, OrderBook, OrderBookEntry, SubscriptionType, Ticker, Trade,
    WebSocketConfig,
};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, instrument, warn};

/// Public function to handle WebSocket market data subscription
/// This is called by the `MarketDataSource` trait implementation
#[instrument(skip(client, config), fields(exchange = "paradex"))]
pub async fn subscribe_market_data_impl(
    client: &ParadexConnector,
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

    // Send all subscriptions
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
                // Send periodic heartbeat/ping
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
    let mut subscription_id = 1;

    // Create all subscription combinations
    for symbol in symbols {
        for sub_type in subscription_types {
            let channel = create_subscription_channel(symbol, sub_type);
            let subscription = json!({
                "method": "SUBSCRIBE",
                "params": [channel],
                "id": subscription_id
            });

            let msg = Message::Text(subscription.to_string());
            ws_sender.send(msg).await.map_err(|e| {
                ExchangeError::NetworkError(format!("Failed to send subscription: {}", e))
            })?;

            subscription_id += 1;
        }
    }

    Ok(())
}

// Helper function to create subscription channel name
fn create_subscription_channel(symbol: &str, sub_type: &SubscriptionType) -> String {
    match sub_type {
        SubscriptionType::Ticker => format!("ticker@{}", symbol),
        SubscriptionType::OrderBook { depth } => depth.as_ref().map_or_else(
            || format!("depth@{}", symbol),
            |d| format!("depth{}@{}", d, symbol),
        ),
        SubscriptionType::Trades => format!("trade@{}", symbol),
        SubscriptionType::Klines { interval } => {
            format!("kline_{}@{}", interval.to_string().to_lowercase(), symbol)
        }
    }
}

// Helper function to handle incoming WebSocket messages
async fn handle_websocket_message(
    msg: Option<Result<Message, tokio_tungstenite::tungstenite::Error>>,
    tx: &mpsc::Sender<MarketDataType>,
    symbols: &[String],
    auto_reconnect: bool,
) -> bool {
    match msg {
        Some(Ok(Message::Text(text))) => process_text_message(&text, tx, symbols).await,
        Some(Ok(Message::Binary(_) | Message::Ping(_) | Message::Pong(_) | Message::Frame(_))) => {
            // Handle binary, ping, pong, and frame messages - continue processing
            false
        }
        Some(Ok(Message::Close(_))) => {
            warn!("WebSocket connection closed by server");
            true
        }
        Some(Err(e)) => {
            error!("WebSocket error: {}", e);
            if auto_reconnect {
                warn!("Attempting to reconnect...");
            }
            true
        }
        None => {
            warn!("WebSocket stream ended");
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
        warn!("Failed to parse WebSocket message: {}", text);
        return false;
    };

    // Handle subscription confirmations
    if parsed.get("result").is_some() && parsed.get("id").is_some() {
        // This is a subscription confirmation, continue processing
        return false;
    }

    let Some(channel) = parsed.get("channel").and_then(|c| c.as_str()) else {
        return false;
    };

    let Some(data) = parsed.get("data") else {
        return false;
    };

    // Process data for relevant symbols
    for symbol in symbols {
        if channel.contains(symbol) {
            if let Some(market_data) = convert_ws_data(channel, data, symbol) {
                if tx.send(market_data).await.is_err() {
                    warn!("Receiver dropped, stopping WebSocket task");
                    return true;
                }
            }
        }
    }

    false
}

// Helper function to convert WebSocket data to MarketDataType
fn convert_ws_data(channel: &str, data: &Value, symbol: &str) -> Option<MarketDataType> {
    if channel.contains("ticker") {
        convert_ticker_data(data, symbol)
    } else if channel.contains("depth") {
        convert_orderbook_data(data, symbol)
    } else if channel.contains("trade") {
        convert_trade_data(data, symbol)
    } else if channel.contains("kline") {
        convert_kline_data(data, symbol)
    } else {
        None
    }
}

// Convert ticker data
fn convert_ticker_data(data: &Value, symbol: &str) -> Option<MarketDataType> {
    let ticker = Ticker {
        symbol: conversion::string_to_symbol(symbol),
        price: conversion::string_to_price(data.get("price")?.as_str()?),
        price_change: conversion::string_to_price(data.get("price_change")?.as_str()?),
        price_change_percent: conversion::string_to_decimal(
            data.get("price_change_percent")?.as_str()?,
        ),
        high_price: conversion::string_to_price(data.get("high")?.as_str()?),
        low_price: conversion::string_to_price(data.get("low")?.as_str()?),
        volume: conversion::string_to_volume(data.get("volume")?.as_str()?),
        quote_volume: conversion::string_to_volume(data.get("quote_volume")?.as_str()?),
        open_time: data.get("open_time")?.as_i64()?,
        close_time: data.get("close_time")?.as_i64()?,
        count: data.get("count")?.as_i64()?,
    };
    Some(MarketDataType::Ticker(ticker))
}

// Convert order book data
fn convert_orderbook_data(data: &Value, symbol: &str) -> Option<MarketDataType> {
    let bids = data
        .get("bids")?
        .as_array()?
        .iter()
        .filter_map(|bid| {
            if let [price, quantity] = bid.as_array()?.as_slice() {
                Some(OrderBookEntry {
                    price: conversion::string_to_price(price.as_str()?),
                    quantity: conversion::string_to_quantity(quantity.as_str()?),
                })
            } else {
                None
            }
        })
        .collect();

    let asks = data
        .get("asks")?
        .as_array()?
        .iter()
        .filter_map(|ask| {
            if let [price, quantity] = ask.as_array()?.as_slice() {
                Some(OrderBookEntry {
                    price: conversion::string_to_price(price.as_str()?),
                    quantity: conversion::string_to_quantity(quantity.as_str()?),
                })
            } else {
                None
            }
        })
        .collect();

    let order_book = OrderBook {
        symbol: conversion::string_to_symbol(symbol),
        bids,
        asks,
        last_update_id: data.get("last_update_id")?.as_i64()?,
    };

    Some(MarketDataType::OrderBook(order_book))
}

// Convert trade data
fn convert_trade_data(data: &Value, symbol: &str) -> Option<MarketDataType> {
    let trade = Trade {
        symbol: conversion::string_to_symbol(symbol),
        id: data.get("id")?.as_i64()?,
        price: conversion::string_to_price(data.get("price")?.as_str()?),
        quantity: conversion::string_to_quantity(data.get("quantity")?.as_str()?),
        time: data.get("time")?.as_i64()?,
        is_buyer_maker: data.get("is_buyer_maker")?.as_bool()?,
    };
    Some(MarketDataType::Trade(trade))
}

// Convert kline data
fn convert_kline_data(data: &Value, symbol: &str) -> Option<MarketDataType> {
    let kline = Kline {
        symbol: conversion::string_to_symbol(symbol),
        open_time: data.get("open_time")?.as_i64()?,
        close_time: data.get("close_time")?.as_i64()?,
        interval: data.get("interval")?.as_str()?.to_string(),
        open_price: conversion::string_to_price(data.get("open")?.as_str()?),
        high_price: conversion::string_to_price(data.get("high")?.as_str()?),
        low_price: conversion::string_to_price(data.get("low")?.as_str()?),
        close_price: conversion::string_to_price(data.get("close")?.as_str()?),
        volume: conversion::string_to_volume(data.get("volume")?.as_str()?),
        number_of_trades: data.get("trades")?.as_i64()?,
        final_bar: data.get("final")?.as_bool()?,
    };
    Some(MarketDataType::Kline(kline))
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
        error!("Failed to send heartbeat: {}", e);
        return true;
    }
    false
}
