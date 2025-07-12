use crate::core::errors::ExchangeError;
use crate::core::kernel::WsCodec;
use crate::core::types::MarketDataType;
use crate::exchanges::bybit_perp::conversions::parse_websocket_message;
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

/// WebSocket events for Bybit Perpetual
#[derive(Debug, Clone)]
pub enum BybitPerpWsEvent {
    MarketData(MarketDataType),
    Ping,
    Pong,
    Error(String),
    Other(Value),
}

/// Bybit Perpetual WebSocket codec
pub struct BybitPerpCodec;

impl Default for BybitPerpCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl BybitPerpCodec {
    pub fn new() -> Self {
        Self
    }
}

impl WsCodec for BybitPerpCodec {
    type Message = BybitPerpWsEvent;

    fn encode_subscription(&self, streams: &[impl AsRef<str>]) -> Result<Message, ExchangeError> {
        let topics: Vec<String> = streams.iter().map(|s| s.as_ref().to_string()).collect();

        let subscribe_message = json!({
            "op": "subscribe",
            "args": topics
        });

        let message_str = serde_json::to_string(&subscribe_message)
            .map_err(|e| ExchangeError::Other(format!("Failed to encode subscription: {}", e)))?;

        Ok(Message::Text(message_str))
    }

    fn encode_unsubscription(&self, streams: &[impl AsRef<str>]) -> Result<Message, ExchangeError> {
        let topics: Vec<String> = streams.iter().map(|s| s.as_ref().to_string()).collect();

        let unsubscribe_message = json!({
            "op": "unsubscribe",
            "args": topics
        });

        let message_str = serde_json::to_string(&unsubscribe_message)
            .map_err(|e| ExchangeError::Other(format!("Failed to encode unsubscription: {}", e)))?;

        Ok(Message::Text(message_str))
    }

    fn decode_message(&self, msg: Message) -> Result<Option<Self::Message>, ExchangeError> {
        match msg {
            Message::Text(text) => {
                // Handle ping messages
                if text.trim() == "ping" {
                    return Ok(Some(BybitPerpWsEvent::Ping));
                }

                // Try to parse as JSON
                let value: Value = serde_json::from_str(&text).map_err(|e| {
                    ExchangeError::Other(format!("Failed to parse WebSocket message: {}", e))
                })?;

                // Handle different message types
                if let Some(op) = value.get("op").and_then(|v| v.as_str()) {
                    match op {
                        "ping" => Ok(Some(BybitPerpWsEvent::Ping)),
                        "pong" => Ok(Some(BybitPerpWsEvent::Pong)),
                        "subscribe" | "unsubscribe" => {
                            // Subscription confirmation, not market data
                            Ok(None)
                        }
                        _ => Ok(Some(BybitPerpWsEvent::Other(value))),
                    }
                } else if value.get("topic").is_some() {
                    // This is market data
                    parse_websocket_message(value.clone()).map_or_else(
                        || Ok(Some(BybitPerpWsEvent::Other(value))),
                        |market_data| Ok(Some(BybitPerpWsEvent::MarketData(market_data))),
                    )
                } else if let Some(ret_msg) = value.get("ret_msg").and_then(|v| v.as_str()) {
                    // Error response
                    Ok(Some(BybitPerpWsEvent::Error(ret_msg.to_string())))
                } else {
                    // Unknown message format
                    Ok(Some(BybitPerpWsEvent::Other(value)))
                }
            }
            Message::Ping(data) => {
                // WebSocket ping frame
                let _ = data; // Suppress unused variable warning
                Ok(Some(BybitPerpWsEvent::Ping))
            }
            Message::Pong(data) => {
                // WebSocket pong frame
                let _ = data; // Suppress unused variable warning
                Ok(Some(BybitPerpWsEvent::Pong))
            }
            Message::Binary(_) => {
                // Bybit doesn't typically use binary messages for market data
                Ok(None)
            }
            Message::Close(_) => {
                // Connection closed
                Ok(None)
            }
            Message::Frame(_) => {
                // Raw frame, not typically handled directly
                Ok(None)
            }
        }
    }
}

/// Helper functions for creating stream identifiers
pub fn create_bybit_perp_stream_identifiers(
    symbols: &[String],
    subscription_types: &[crate::core::types::SubscriptionType],
) -> Vec<String> {
    let mut streams = Vec::new();

    for symbol in symbols {
        for sub_type in subscription_types {
            match sub_type {
                crate::core::types::SubscriptionType::Ticker => {
                    streams.push(format!("tickers.{}", symbol));
                }
                crate::core::types::SubscriptionType::OrderBook { depth } => {
                    if let Some(d) = depth {
                        streams.push(format!("orderbook.{}.{}", d, symbol));
                    } else {
                        streams.push(format!("orderbook.1.{}", symbol));
                    }
                }
                crate::core::types::SubscriptionType::Trades => {
                    streams.push(format!("publicTrade.{}", symbol));
                }
                crate::core::types::SubscriptionType::Klines { interval } => {
                    let interval_str = match interval {
                        crate::core::types::KlineInterval::Minutes3 => "3",
                        crate::core::types::KlineInterval::Minutes5 => "5",
                        crate::core::types::KlineInterval::Minutes15 => "15",
                        crate::core::types::KlineInterval::Minutes30 => "30",
                        crate::core::types::KlineInterval::Hours1 => "60",
                        crate::core::types::KlineInterval::Hours2 => "120",
                        crate::core::types::KlineInterval::Hours4 => "240",
                        crate::core::types::KlineInterval::Hours6 => "360",
                        crate::core::types::KlineInterval::Hours12 => "720",
                        crate::core::types::KlineInterval::Days1 => "D",
                        crate::core::types::KlineInterval::Weeks1 => "W",
                        crate::core::types::KlineInterval::Months1 => "M",
                        _ => "1", // Default to 1 minute (including Minutes1)
                    };
                    streams.push(format!("kline.{}.{}", interval_str, symbol));
                }
            }
        }
    }

    streams
}
