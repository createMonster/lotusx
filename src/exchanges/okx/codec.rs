use crate::core::errors::ExchangeError;
use crate::core::kernel::codec::WsCodec;
use crate::core::types::SubscriptionType;
use crate::exchanges::okx::types::{OkxWsChannel, OkxWsRequest};
use serde_json::Value;
use std::collections::HashMap;
use tokio_tungstenite::tungstenite::Message;

/// OKX WebSocket message types
#[derive(Debug, Clone)]
pub enum OkxMessage {
    /// Subscription confirmation
    Subscribe {
        channel: String,
        inst_id: Option<String>,
    },
    /// Market data update
    Data {
        channel: String,
        inst_id: Option<String>,
        data: Value,
    },
    /// Error message
    Error { code: String, message: String },
    /// Pong response
    Pong,
    /// Login response
    Login { success: bool, message: String },
}

/// OKX WebSocket codec implementation
pub struct OkxCodec {
    /// Channel subscriptions
    #[allow(dead_code)]
    subscriptions: HashMap<String, SubscriptionType>,
}

impl OkxCodec {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }

    /// Create subscription request for OKX WebSocket
    fn create_subscription_request(
        channels: Vec<OkxWsChannel>,
        operation: &str,
    ) -> Result<String, ExchangeError> {
        let request = OkxWsRequest {
            op: operation.to_string(),
            args: channels,
        };

        serde_json::to_string(&request)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))
    }

    /// Parse OKX channel name and instrument ID from subscription
    fn parse_channel_info(channel: &str) -> (String, Option<String>) {
        // OKX channels often have format like "tickers:BTC-USDT" or "books:BTC-USDT"
        channel.find(':').map_or_else(
            || (channel.to_string(), None),
            |pos| {
                let channel_name = channel[..pos].to_string();
                let inst_id = Some(channel[pos + 1..].to_string());
                (channel_name, inst_id)
            },
        )
    }
}

impl Default for OkxCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl WsCodec for OkxCodec {
    type Message = OkxMessage;

    fn encode_subscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError> {
        let mut channels = Vec::new();

        for stream in streams {
            let stream_str = stream.as_ref();
            let (channel_name, inst_id) = Self::parse_channel_info(stream_str);

            let channel = OkxWsChannel {
                channel: channel_name,
                inst_type: Some("SPOT".to_string()),
                inst_family: None,
                inst_id,
            };

            channels.push(channel);
        }

        let message_str = Self::create_subscription_request(channels, "subscribe")?;
        Ok(Message::Text(message_str))
    }

    fn encode_unsubscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError> {
        let mut channels = Vec::new();

        for stream in streams {
            let stream_str = stream.as_ref();
            let (channel_name, inst_id) = Self::parse_channel_info(stream_str);

            let channel = OkxWsChannel {
                channel: channel_name,
                inst_type: Some("SPOT".to_string()),
                inst_family: None,
                inst_id,
            };

            channels.push(channel);
        }

        let message_str = Self::create_subscription_request(channels, "unsubscribe")?;
        Ok(Message::Text(message_str))
    }

    fn decode_message(&self, message: Message) -> Result<Option<Self::Message>, ExchangeError> {
        let text = match message {
            Message::Text(text) => text,
            Message::Binary(data) => String::from_utf8(data).map_err(|e| {
                ExchangeError::ParseError(format!("Invalid UTF-8 in binary message: {}", e))
            })?,
            Message::Pong(_) => return Ok(Some(OkxMessage::Pong)),
            _ => return Ok(None), // Ignore other message types
        };

        // Handle simple text messages
        if text == "pong" {
            return Ok(Some(OkxMessage::Pong));
        }

        // Try to parse as JSON
        let value: Value = serde_json::from_str(&text)
            .map_err(|e| ExchangeError::ParseError(format!("Failed to parse JSON: {}", e)))?;

        // Handle different message types
        if let Some(event) = value.get("event").and_then(|v| v.as_str()) {
            match event {
                "subscribe" => {
                    let arg = value
                        .get("arg")
                        .and_then(|v| serde_json::from_value::<OkxWsChannel>(v.clone()).ok());

                    if let Some(channel_info) = arg {
                        return Ok(Some(OkxMessage::Subscribe {
                            channel: channel_info.channel,
                            inst_id: channel_info.inst_id,
                        }));
                    }
                }
                "error" => {
                    let code = value
                        .get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let msg = value
                        .get("msg")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown error")
                        .to_string();

                    return Ok(Some(OkxMessage::Error { code, message: msg }));
                }
                "login" => {
                    let code = value.get("code").and_then(|v| v.as_str()).unwrap_or("1");
                    let success = code == "0";
                    let msg = value
                        .get("msg")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    return Ok(Some(OkxMessage::Login {
                        success,
                        message: msg,
                    }));
                }
                _ => {}
            }
        }

        // Handle data messages
        if let Some(arg) = value.get("arg") {
            let channel_info: OkxWsChannel = serde_json::from_value(arg.clone()).map_err(|e| {
                ExchangeError::ParseError(format!("Failed to parse channel: {}", e))
            })?;

            let data = value
                .get("data")
                .ok_or_else(|| ExchangeError::ParseError("Missing data field".to_string()))?
                .clone();

            return Ok(Some(OkxMessage::Data {
                channel: channel_info.channel,
                inst_id: channel_info.inst_id,
                data,
            }));
        }

        Err(ExchangeError::ParseError(format!(
            "Unknown message format: {}",
            text
        )))
    }
}

/// Helper function to create OKX WebSocket stream identifiers
pub fn create_okx_stream_identifiers(
    symbols: &[String],
    subscription_types: &[SubscriptionType],
) -> Vec<String> {
    let mut identifiers = Vec::new();

    for symbol in symbols {
        for sub_type in subscription_types {
            let channel = match sub_type {
                SubscriptionType::Ticker => "tickers",
                SubscriptionType::OrderBook { depth: _ } => "books",
                SubscriptionType::Trades => "trades",
                SubscriptionType::Klines { interval: _ } => "candle1m",
            };

            identifiers.push(format!("{}:{}", channel, symbol));
        }
    }

    identifiers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_subscribe() {
        let codec = OkxCodec::new();
        let streams = vec!["tickers:BTC-USDT"];

        let result = codec.encode_subscription(&streams);
        assert!(result.is_ok());

        if let Message::Text(text) = result.unwrap() {
            assert!(text.contains("subscribe"));
            assert!(text.contains("tickers"));
            assert!(text.contains("BTC-USDT"));
        } else {
            panic!("Expected text message");
        }
    }

    #[test]
    fn test_decode_pong() {
        let codec = OkxCodec::new();
        let result = codec.decode_message(Message::Text("pong".to_string()));
        assert!(result.is_ok());

        if matches!(result.unwrap(), Some(OkxMessage::Pong)) {
            // Test passed
        } else {
            panic!("Expected pong message");
        }
    }

    #[test]
    fn test_decode_error() {
        let codec = OkxCodec::new();
        let error_msg = r#"{"event":"error","code":"60012","msg":"Invalid request"}"#;
        let result = codec.decode_message(Message::Text(error_msg.to_string()));
        assert!(result.is_ok());

        if let Some(OkxMessage::Error { code, message }) = result.unwrap() {
            assert_eq!(code, "60012");
            assert_eq!(message, "Invalid request");
        } else {
            panic!("Expected error message");
        }
    }

    #[test]
    fn test_stream_identifiers() {
        let symbols = vec!["BTC-USDT".to_string(), "ETH-USDT".to_string()];
        let subscription_types = vec![
            SubscriptionType::Ticker,
            SubscriptionType::OrderBook { depth: None },
        ];

        let identifiers = create_okx_stream_identifiers(&symbols, &subscription_types);

        assert_eq!(identifiers.len(), 4);
        assert!(identifiers.contains(&"tickers:BTC-USDT".to_string()));
        assert!(identifiers.contains(&"books:BTC-USDT".to_string()));
        assert!(identifiers.contains(&"tickers:ETH-USDT".to_string()));
        assert!(identifiers.contains(&"books:ETH-USDT".to_string()));
    }
}
