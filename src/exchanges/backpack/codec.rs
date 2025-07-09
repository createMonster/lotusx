use crate::core::errors::ExchangeError;
use crate::core::kernel::WsCodec;
use crate::exchanges::backpack::types::{
    BackpackWebSocketBookTicker, BackpackWebSocketKline, BackpackWebSocketLiquidation,
    BackpackWebSocketMarkPrice, BackpackWebSocketOpenInterest, BackpackWebSocketOrderBook,
    BackpackWebSocketRFQ, BackpackWebSocketRFQUpdate, BackpackWebSocketTicker,
    BackpackWebSocketTrade,
};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

/// Typed messages for Backpack WebSocket streams
#[derive(Debug, Clone)]
pub enum BackpackMessage {
    Ticker(BackpackWebSocketTicker),
    OrderBook(BackpackWebSocketOrderBook),
    Trade(BackpackWebSocketTrade),
    Kline(BackpackWebSocketKline),
    MarkPrice(BackpackWebSocketMarkPrice),
    OpenInterest(BackpackWebSocketOpenInterest),
    Liquidation(BackpackWebSocketLiquidation),
    BookTicker(BackpackWebSocketBookTicker),
    RFQ(BackpackWebSocketRFQ),
    RFQUpdate(BackpackWebSocketRFQUpdate),
    Ping {
        ping: i64,
    },
    Pong {
        pong: i64,
    },
    Subscription {
        status: String,
        params: Vec<String>,
        id: i64,
    },
    Unknown(Value),
}

/// Backpack WebSocket codec implementation
pub struct BackpackCodec;

impl BackpackCodec {
    /// Create a new Backpack codec
    pub fn new() -> Self {
        Self
    }

    /// Build a subscription message in Backpack format
    fn build_subscription_message(&self, streams: &[impl AsRef<str>]) -> Value {
        let params: Vec<String> = streams.iter().map(|s| s.as_ref().to_string()).collect();

        json!({
            "method": "SUBSCRIBE",
            "params": params,
            "id": 1
        })
    }

    /// Build an unsubscription message in Backpack format
    fn build_unsubscription_message(&self, streams: &[impl AsRef<str>]) -> Value {
        let params: Vec<String> = streams.iter().map(|s| s.as_ref().to_string()).collect();

        json!({
            "method": "UNSUBSCRIBE",
            "params": params,
            "id": 1
        })
    }

    /// Parse incoming WebSocket message into typed `BackpackMessage`
    fn parse_websocket_message(&self, value: &Value) -> BackpackMessage {
        // Handle subscription confirmations
        if let Some(result) = value.get("result") {
            if let Some(params) = result.as_array() {
                return BackpackMessage::Subscription {
                    status: "confirmed".to_string(),
                    params: params
                        .iter()
                        .filter_map(|p| p.as_str().map(|s| s.to_string()))
                        .collect(),
                    id: value.get("id").and_then(|id| id.as_i64()).unwrap_or(0),
                };
            }
        }

        // Handle ping/pong
        if let Some(ping) = value.get("ping") {
            if let Some(ping_val) = ping.as_i64() {
                return BackpackMessage::Ping { ping: ping_val };
            }
        }

        if let Some(pong) = value.get("pong") {
            if let Some(pong_val) = pong.as_i64() {
                return BackpackMessage::Pong { pong: pong_val };
            }
        }

        // Handle stream data - determine message type by event type
        if let Some(event_type) = value.get("e").and_then(|e| e.as_str()) {
            match event_type {
                "24hrTicker" => {
                    if let Ok(ticker) =
                        serde_json::from_value::<BackpackWebSocketTicker>(value.clone())
                    {
                        return BackpackMessage::Ticker(ticker);
                    }
                }
                "depthUpdate" => {
                    if let Ok(orderbook) =
                        serde_json::from_value::<BackpackWebSocketOrderBook>(value.clone())
                    {
                        return BackpackMessage::OrderBook(orderbook);
                    }
                }
                "trade" => {
                    if let Ok(trade) =
                        serde_json::from_value::<BackpackWebSocketTrade>(value.clone())
                    {
                        return BackpackMessage::Trade(trade);
                    }
                }
                "kline" => {
                    if let Ok(kline) =
                        serde_json::from_value::<BackpackWebSocketKline>(value.clone())
                    {
                        return BackpackMessage::Kline(kline);
                    }
                }
                "markPrice" => {
                    if let Ok(mark_price) =
                        serde_json::from_value::<BackpackWebSocketMarkPrice>(value.clone())
                    {
                        return BackpackMessage::MarkPrice(mark_price);
                    }
                }
                "openInterest" => {
                    if let Ok(open_interest) =
                        serde_json::from_value::<BackpackWebSocketOpenInterest>(value.clone())
                    {
                        return BackpackMessage::OpenInterest(open_interest);
                    }
                }
                "liquidation" => {
                    if let Ok(liquidation) =
                        serde_json::from_value::<BackpackWebSocketLiquidation>(value.clone())
                    {
                        return BackpackMessage::Liquidation(liquidation);
                    }
                }
                "bookTicker" => {
                    if let Ok(book_ticker) =
                        serde_json::from_value::<BackpackWebSocketBookTicker>(value.clone())
                    {
                        return BackpackMessage::BookTicker(book_ticker);
                    }
                }
                "rfq" => {
                    if let Ok(rfq) = serde_json::from_value::<BackpackWebSocketRFQ>(value.clone()) {
                        return BackpackMessage::RFQ(rfq);
                    }
                }
                "rfqUpdate" => {
                    if let Ok(rfq_update) =
                        serde_json::from_value::<BackpackWebSocketRFQUpdate>(value.clone())
                    {
                        return BackpackMessage::RFQUpdate(rfq_update);
                    }
                }
                _ => {}
            }
        }

        // Return unknown message if we can't parse it
        BackpackMessage::Unknown(value.clone())
    }
}

impl WsCodec for BackpackCodec {
    type Message = BackpackMessage;

    fn encode_subscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError> {
        let msg = self.build_subscription_message(streams);
        Ok(Message::Text(msg.to_string()))
    }

    fn encode_unsubscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError> {
        let msg = self.build_unsubscription_message(streams);
        Ok(Message::Text(msg.to_string()))
    }

    fn decode_message(&self, message: Message) -> Result<Option<Self::Message>, ExchangeError> {
        match message {
            Message::Text(text) => {
                let value: Value = serde_json::from_str(&text).map_err(|e| {
                    ExchangeError::DeserializationError(format!("JSON parse error: {}", e))
                })?;

                Ok(Some(self.parse_websocket_message(&value)))
            }
            _ => Ok(None), // Ignore non-text messages
        }
    }
}

impl Default for BackpackCodec {
    fn default() -> Self {
        Self::new()
    }
}
