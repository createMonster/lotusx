use crate::core::errors::ExchangeError;
use crate::core::kernel::WsCodec;
use crate::exchanges::bybit::types::{
    BybitKlineData, BybitWebSocketKline, BybitWebSocketOrderBook, BybitWebSocketTicker,
    BybitWebSocketTrade,
};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use tokio_tungstenite::tungstenite::Message;

/// Bybit WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "topic")]
pub enum BybitWsEvent {
    Ticker {
        data: BybitWebSocketTicker,
    },
    OrderBook {
        data: BybitWebSocketOrderBook,
    },
    Trade {
        data: BybitWebSocketTrade,
    },
    Kline {
        data: BybitWebSocketKline,
    },
    Pong {
        req_id: String,
    },
    #[serde(other)]
    Unknown,
}

/// Bybit subscription request structure
#[derive(Debug, Serialize)]
struct BybitSubscription {
    op: String,
    args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    req_id: Option<String>,
}

/// Bybit WebSocket codec implementation
pub struct BybitCodec;

impl WsCodec for BybitCodec {
    type Message = BybitWsEvent;

    fn encode_subscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError> {
        let stream_strings: Vec<String> = streams.iter().map(|s| s.as_ref().to_string()).collect();
        let subscription = BybitSubscription {
            op: "subscribe".to_string(),
            args: stream_strings,
            req_id: None,
        };

        let json_str = serde_json::to_string(&subscription).map_err(|e| {
            ExchangeError::SerializationError(format!("Failed to encode subscription: {}", e))
        })?;

        Ok(Message::Text(json_str))
    }

    fn encode_unsubscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError> {
        let stream_strings: Vec<String> = streams.iter().map(|s| s.as_ref().to_string()).collect();
        let unsubscription = BybitSubscription {
            op: "unsubscribe".to_string(),
            args: stream_strings,
            req_id: None,
        };

        let json_str = serde_json::to_string(&unsubscription).map_err(|e| {
            ExchangeError::SerializationError(format!("Failed to encode unsubscription: {}", e))
        })?;

        Ok(Message::Text(json_str))
    }

    fn decode_message(&self, message: Message) -> Result<Option<Self::Message>, ExchangeError> {
        match message {
            Message::Text(text) => {
                // Handle ping messages - respond with pong
                if text.contains("\"op\":\"ping\"") {
                    return Ok(Some(BybitWsEvent::Pong {
                        req_id: "pong".to_string(),
                    }));
                }

                // Try to parse as JSON for topic-based routing
                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                    if let Some(topic) = value.get("topic").and_then(|t| t.as_str()) {
                        if let Some(data) = value.get("data") {
                            match topic {
                                t if t.starts_with("tickers.") => {
                                    if let Ok(ticker) =
                                        serde_json::from_value::<BybitWebSocketTicker>(data.clone())
                                    {
                                        return Ok(Some(BybitWsEvent::Ticker { data: ticker }));
                                    }
                                }
                                t if t.starts_with("orderbook.") => {
                                    if let Ok(orderbook) =
                                        serde_json::from_value::<BybitWebSocketOrderBook>(
                                            data.clone(),
                                        )
                                    {
                                        return Ok(Some(BybitWsEvent::OrderBook {
                                            data: orderbook,
                                        }));
                                    }
                                }
                                t if t.starts_with("publicTrade.") => {
                                    let trade_data = data
                                        .as_array()
                                        .and_then(|items| items.first())
                                        .unwrap_or(data);
                                    if let Ok(trade) = serde_json::from_value::<BybitWebSocketTrade>(
                                        trade_data.clone(),
                                    ) {
                                        return Ok(Some(BybitWsEvent::Trade { data: trade }));
                                    }
                                }
                                t if t.starts_with("kline.") => {
                                    let kline_data = data
                                        .as_array()
                                        .and_then(|items| items.first())
                                        .unwrap_or(data);
                                    if let Ok(kline) =
                                        serde_json::from_value::<BybitKlineData>(kline_data.clone())
                                    {
                                        let symbol = t.rsplit('.').next().unwrap_or("").to_string();
                                        return Ok(Some(BybitWsEvent::Kline {
                                            data: BybitWebSocketKline { symbol, kline },
                                        }));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // If we can't parse it, return Unknown
                Ok(Some(BybitWsEvent::Unknown))
            }
            Message::Binary(_) => {
                // Bybit uses text messages, so binary messages are ignored
                Ok(None)
            }
            _ => {
                // Ignore other message types (ping, pong, close)
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::kernel::WsCodec;
    use tokio_tungstenite::tungstenite::Message;

    #[test]
    fn decodes_bybit_ticker_message() {
        let message = Message::Text(
            r#"{"topic":"tickers.BTCUSDT","type":"snapshot","ts":1710000000000,"data":{"symbol":"BTCUSDT","lastPrice":"43000","price24hPcnt":"0.01","highPrice24h":"44000","lowPrice24h":"42000","volume24h":"100","turnover24h":"4300000"}}"#
                .to_string(),
        );

        let decoded = BybitCodec.decode_message(message).unwrap();

        match decoded {
            Some(BybitWsEvent::Ticker { data }) => {
                assert_eq!(data.symbol, "BTCUSDT");
                assert_eq!(data.price, "43000");
            }
            other => panic!("expected ticker event, got {other:?}"),
        }
    }
}
