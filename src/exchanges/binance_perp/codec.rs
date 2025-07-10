use crate::core::errors::ExchangeError;
use crate::core::kernel::WsCodec;
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone)]
pub enum BinancePerpMessage {
    Ticker(super::types::BinancePerpWebSocketTicker),
    OrderBook(super::types::BinancePerpWebSocketOrderBook),
    Trade(super::types::BinancePerpWebSocketTrade),
    Kline(super::types::BinancePerpWebSocketKline),
    FundingRate(super::types::BinancePerpFundingRate),
    Unknown,
}

pub struct BinancePerpCodec;

impl WsCodec for BinancePerpCodec {
    type Message = BinancePerpMessage;

    fn encode_subscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError> {
        let stream_refs: Vec<&str> = streams.iter().map(|s| s.as_ref()).collect();
        let subscription = json!({
            "method": "SUBSCRIBE",
            "params": stream_refs,
            "id": 1
        });
        Ok(Message::Text(subscription.to_string()))
    }

    fn encode_unsubscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError> {
        let stream_refs: Vec<&str> = streams.iter().map(|s| s.as_ref()).collect();
        let unsubscription = json!({
            "method": "UNSUBSCRIBE",
            "params": stream_refs,
            "id": 1
        });
        Ok(Message::Text(unsubscription.to_string()))
    }

    fn decode_message(&self, message: Message) -> Result<Option<Self::Message>, ExchangeError> {
        let text = match message {
            Message::Text(text) => text,
            Message::Binary(data) => String::from_utf8(data).map_err(|e| {
                ExchangeError::DeserializationError(format!(
                    "Invalid UTF-8 in binary message: {}",
                    e
                ))
            })?,
            _ => return Ok(None), // Ignore other message types
        };

        let value: Value = serde_json::from_str(&text).map_err(|e| {
            ExchangeError::DeserializationError(format!("Failed to parse JSON: {}", e))
        })?;

        // Handle combined stream format
        if let Some(stream) = value.get("stream").and_then(|s| s.as_str()) {
            let data = value.get("data").ok_or_else(|| {
                ExchangeError::DeserializationError(
                    "Missing data field in stream message".to_string(),
                )
            })?;

            return self.decode_stream_data(stream, data).map(Some);
        }

        // Handle direct stream format or error messages
        if let Some(event_type) = value.get("e").and_then(|e| e.as_str()) {
            return self.decode_event_data(event_type, &value).map(Some);
        }

        // Handle subscription confirmations and errors
        if value.get("result").is_some() || value.get("error").is_some() {
            return Ok(Some(BinancePerpMessage::Unknown));
        }

        Ok(Some(BinancePerpMessage::Unknown))
    }
}

impl BinancePerpCodec {
    fn decode_stream_data(
        &self,
        stream: &str,
        data: &Value,
    ) -> Result<BinancePerpMessage, ExchangeError> {
        if stream.contains("@ticker") {
            let ticker: super::types::BinancePerpWebSocketTicker =
                serde_json::from_value(data.clone()).map_err(|e| {
                    ExchangeError::DeserializationError(format!("Failed to parse ticker: {}", e))
                })?;
            Ok(BinancePerpMessage::Ticker(ticker))
        } else if stream.contains("@depth") {
            let orderbook: super::types::BinancePerpWebSocketOrderBook =
                serde_json::from_value(data.clone()).map_err(|e| {
                    ExchangeError::DeserializationError(format!("Failed to parse orderbook: {}", e))
                })?;
            Ok(BinancePerpMessage::OrderBook(orderbook))
        } else if stream.contains("@trade") {
            let trade: super::types::BinancePerpWebSocketTrade =
                serde_json::from_value(data.clone()).map_err(|e| {
                    ExchangeError::DeserializationError(format!("Failed to parse trade: {}", e))
                })?;
            Ok(BinancePerpMessage::Trade(trade))
        } else if stream.contains("@kline") {
            let kline: super::types::BinancePerpWebSocketKline =
                serde_json::from_value(data.clone()).map_err(|e| {
                    ExchangeError::DeserializationError(format!("Failed to parse kline: {}", e))
                })?;
            Ok(BinancePerpMessage::Kline(kline))
        } else if stream.contains("@markPrice") {
            // Handle funding rate data if it contains funding rate info
            #[allow(clippy::option_if_let_else)]
            if let Ok(funding_rate) =
                serde_json::from_value::<super::types::BinancePerpFundingRate>(data.clone())
            {
                Ok(BinancePerpMessage::FundingRate(funding_rate))
            } else {
                Ok(BinancePerpMessage::Unknown)
            }
        } else {
            Ok(BinancePerpMessage::Unknown)
        }
    }

    fn decode_event_data(
        &self,
        event_type: &str,
        data: &Value,
    ) -> Result<BinancePerpMessage, ExchangeError> {
        match event_type {
            "24hrTicker" => {
                let ticker: super::types::BinancePerpWebSocketTicker =
                    serde_json::from_value(data.clone()).map_err(|e| {
                        ExchangeError::DeserializationError(format!(
                            "Failed to parse ticker: {}",
                            e
                        ))
                    })?;
                Ok(BinancePerpMessage::Ticker(ticker))
            }
            "depthUpdate" => {
                let orderbook: super::types::BinancePerpWebSocketOrderBook =
                    serde_json::from_value(data.clone()).map_err(|e| {
                        ExchangeError::DeserializationError(format!(
                            "Failed to parse orderbook: {}",
                            e
                        ))
                    })?;
                Ok(BinancePerpMessage::OrderBook(orderbook))
            }
            "trade" => {
                let trade: super::types::BinancePerpWebSocketTrade =
                    serde_json::from_value(data.clone()).map_err(|e| {
                        ExchangeError::DeserializationError(format!("Failed to parse trade: {}", e))
                    })?;
                Ok(BinancePerpMessage::Trade(trade))
            }
            "kline" => {
                let kline: super::types::BinancePerpWebSocketKline =
                    serde_json::from_value(data.clone()).map_err(|e| {
                        ExchangeError::DeserializationError(format!("Failed to parse kline: {}", e))
                    })?;
                Ok(BinancePerpMessage::Kline(kline))
            }
            "markPriceUpdate" => {
                #[allow(clippy::option_if_let_else)]
                if let Ok(funding_rate) =
                    serde_json::from_value::<super::types::BinancePerpFundingRate>(data.clone())
                {
                    Ok(BinancePerpMessage::FundingRate(funding_rate))
                } else {
                    Ok(BinancePerpMessage::Unknown)
                }
            }
            _ => Ok(BinancePerpMessage::Unknown),
        }
    }
}

/// Create Binance Perpetual stream identifiers for WebSocket subscriptions
pub fn create_binance_perp_stream_identifiers(
    symbols: &[String],
    subscription_types: &[crate::core::types::SubscriptionType],
) -> Vec<String> {
    let mut streams = Vec::new();

    for symbol in symbols {
        let lower_symbol = symbol.to_lowercase();
        for sub_type in subscription_types {
            match sub_type {
                crate::core::types::SubscriptionType::Ticker => {
                    streams.push(format!("{}@ticker", lower_symbol));
                }
                crate::core::types::SubscriptionType::OrderBook { depth } => {
                    if let Some(d) = depth {
                        streams.push(format!("{}@depth{}@100ms", lower_symbol, d));
                    } else {
                        streams.push(format!("{}@depth@100ms", lower_symbol));
                    }
                }
                crate::core::types::SubscriptionType::Trades => {
                    streams.push(format!("{}@trade", lower_symbol));
                }
                crate::core::types::SubscriptionType::Klines { interval } => {
                    streams.push(format!(
                        "{}@kline_{}",
                        lower_symbol,
                        interval.to_binance_format()
                    ));
                }
            }
        }
    }

    // Add funding rate streams for perpetual futures
    for symbol in symbols {
        let lower_symbol = symbol.to_lowercase();
        streams.push(format!("{}@markPrice", lower_symbol));
    }

    streams
}
