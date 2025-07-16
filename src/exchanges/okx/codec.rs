use crate::core::errors::ExchangeError;
use crate::core::kernel::Codec;
use crate::core::types::SubscriptionType;
use crate::exchanges::okx::types::{OkxWsChannel, OkxWsRequest, OkxWsResponse};
use serde_json::Value;
use std::collections::HashMap;

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
        if let Some(pos) = channel.find(':') {
            let channel_name = channel[..pos].to_string();
            let inst_id = Some(channel[pos + 1..].to_string());
            (channel_name, inst_id)
        } else {
            (channel.to_string(), None)
        }
    }
}

impl Default for OkxCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Codec for OkxCodec {
    type Message = OkxMessage;

    fn encode_subscribe(
        &mut self,
        symbols: &[String],
        subscription_types: &[SubscriptionType],
    ) -> Result<Vec<String>, ExchangeError> {
        let mut messages = Vec::new();
        let mut channels = Vec::new();

        for symbol in symbols {
            for sub_type in subscription_types {
                let channel_name = match sub_type {
                    SubscriptionType::Ticker => "tickers",
                    SubscriptionType::OrderBook => "books",
                    SubscriptionType::Trade => "trades",
                    SubscriptionType::Kline => "candle1m", // Default to 1m candles
                    _ => continue,                         // Skip unsupported types
                };

                let channel = OkxWsChannel {
                    channel: channel_name.to_string(),
                    inst_type: Some("SPOT".to_string()),
                    inst_family: None,
                    inst_id: Some(symbol.clone()),
                };

                // Track subscription
                let subscription_key = format!("{}:{}", channel_name, symbol);
                self.subscriptions.insert(subscription_key, *sub_type);

                channels.push(channel);
            }
        }

        if !channels.is_empty() {
            let message = Self::create_subscription_request(channels, "subscribe")?;
            messages.push(message);
        }

        Ok(messages)
    }

    fn encode_unsubscribe(
        &mut self,
        symbols: &[String],
        subscription_types: &[SubscriptionType],
    ) -> Result<Vec<String>, ExchangeError> {
        let mut messages = Vec::new();
        let mut channels = Vec::new();

        for symbol in symbols {
            for sub_type in subscription_types {
                let channel_name = match sub_type {
                    SubscriptionType::Ticker => "tickers",
                    SubscriptionType::OrderBook => "books",
                    SubscriptionType::Trade => "trades",
                    SubscriptionType::Kline => "candle1m",
                    _ => continue,
                };

                let channel = OkxWsChannel {
                    channel: channel_name.to_string(),
                    inst_type: Some("SPOT".to_string()),
                    inst_family: None,
                    inst_id: Some(symbol.clone()),
                };

                // Remove from subscriptions
                let subscription_key = format!("{}:{}", channel_name, symbol);
                self.subscriptions.remove(&subscription_key);

                channels.push(channel);
            }
        }

        if !channels.is_empty() {
            let message = Self::create_subscription_request(channels, "unsubscribe")?;
            messages.push(message);
        }

        Ok(messages)
    }

    fn encode_ping(&self) -> Result<String, ExchangeError> {
        Ok("ping".to_string())
    }

    fn decode(&self, message: &str) -> Result<Self::Message, ExchangeError> {
        // Handle simple text messages
        if message == "pong" {
            return Ok(OkxMessage::Pong);
        }

        // Try to parse as JSON
        let value: Value = serde_json::from_str(message)
            .map_err(|e| ExchangeError::ParseError(format!("Failed to parse JSON: {}", e)))?;

        // Handle different message types
        if let Some(event) = value.get("event").and_then(|v| v.as_str()) {
            match event {
                "subscribe" => {
                    let arg = value
                        .get("arg")
                        .and_then(|v| serde_json::from_value::<OkxWsChannel>(v.clone()).ok());

                    if let Some(channel_info) = arg {
                        return Ok(OkxMessage::Subscribe {
                            channel: channel_info.channel,
                            inst_id: channel_info.inst_id,
                        });
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

                    return Ok(OkxMessage::Error { code, message: msg });
                }
                "login" => {
                    let code = value.get("code").and_then(|v| v.as_str()).unwrap_or("1");
                    let success = code == "0";
                    let msg = value
                        .get("msg")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    return Ok(OkxMessage::Login {
                        success,
                        message: msg,
                    });
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

            return Ok(OkxMessage::Data {
                channel: channel_info.channel,
                inst_id: channel_info.inst_id,
                data,
            });
        }

        Err(ExchangeError::ParseError(format!(
            "Unknown message format: {}",
            message
        )))
    }

    fn get_subscription_type(&self, message: &Self::Message) -> Option<SubscriptionType> {
        match message {
            OkxMessage::Data {
                channel, inst_id, ..
            } => {
                if let Some(inst_id) = inst_id {
                    let subscription_key = format!("{}:{}", channel, inst_id);
                    self.subscriptions.get(&subscription_key).copied()
                } else {
                    // Map channel name to subscription type
                    match channel.as_str() {
                        "tickers" => Some(SubscriptionType::Ticker),
                        "books" => Some(SubscriptionType::OrderBook),
                        "trades" => Some(SubscriptionType::Trade),
                        name if name.starts_with("candle") => Some(SubscriptionType::Kline),
                        _ => None,
                    }
                }
            }
            _ => None,
        }
    }

    fn is_error(&self, message: &Self::Message) -> bool {
        matches!(message, OkxMessage::Error { .. })
    }

    fn is_pong(&self, message: &Self::Message) -> bool {
        matches!(message, OkxMessage::Pong)
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
                SubscriptionType::OrderBook => "books",
                SubscriptionType::Trade => "trades",
                SubscriptionType::Kline => "candle1m",
                _ => continue,
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
        let mut codec = OkxCodec::new();
        let symbols = vec!["BTC-USDT".to_string()];
        let subscription_types = vec![SubscriptionType::Ticker];

        let result = codec.encode_subscribe(&symbols, &subscription_types);
        assert!(result.is_ok());

        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert!(messages[0].contains("subscribe"));
        assert!(messages[0].contains("tickers"));
        assert!(messages[0].contains("BTC-USDT"));
    }

    #[test]
    fn test_decode_pong() {
        let codec = OkxCodec::new();
        let result = codec.decode("pong");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), OkxMessage::Pong));
    }

    #[test]
    fn test_decode_error() {
        let codec = OkxCodec::new();
        let error_msg = r#"{"event":"error","code":"60012","msg":"Invalid request"}"#;
        let result = codec.decode(error_msg);
        assert!(result.is_ok());

        if let OkxMessage::Error { code, message } = result.unwrap() {
            assert_eq!(code, "60012");
            assert_eq!(message, "Invalid request");
        } else {
            panic!("Expected error message");
        }
    }

    #[test]
    fn test_stream_identifiers() {
        let symbols = vec!["BTC-USDT".to_string(), "ETH-USDT".to_string()];
        let subscription_types = vec![SubscriptionType::Ticker, SubscriptionType::OrderBook];

        let identifiers = create_okx_stream_identifiers(&symbols, &subscription_types);

        assert_eq!(identifiers.len(), 4);
        assert!(identifiers.contains(&"tickers:BTC-USDT".to_string()));
        assert!(identifiers.contains(&"books:BTC-USDT".to_string()));
        assert!(identifiers.contains(&"tickers:ETH-USDT".to_string()));
        assert!(identifiers.contains(&"books:ETH-USDT".to_string()));
    }
}
