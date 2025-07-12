use crate::core::errors::ExchangeError;
use crate::core::kernel::codec::WsCodec;
use crate::core::types::conversion;
use crate::core::types::{
    Kline, MarketDataType, OrderBook, OrderBookEntry, SubscriptionType, Ticker, Trade,
};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;

/// Paradex WebSocket events
#[derive(Debug, Clone)]
pub enum ParadexWsEvent {
    Ticker(Ticker),
    OrderBook(OrderBook),
    Trade(Trade),
    Kline(Kline),
    SubscriptionConfirmation(Value),
    Heartbeat,
    Error(String),
}

/// Paradex WebSocket codec implementation
pub struct ParadexCodec;

impl WsCodec for ParadexCodec {
    type Message = ParadexWsEvent;

    fn encode_subscription(&self, streams: &[impl AsRef<str>]) -> Result<Message, ExchangeError> {
        let subscribe_msg = json!({
            "method": "subscribe",
            "params": streams.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
            "id": 1
        });

        Ok(Message::Text(subscribe_msg.to_string()))
    }

    fn encode_unsubscription(&self, streams: &[impl AsRef<str>]) -> Result<Message, ExchangeError> {
        let unsubscribe_msg = json!({
            "method": "unsubscribe",
            "params": streams.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
            "id": 2
        });

        Ok(Message::Text(unsubscribe_msg.to_string()))
    }

    fn decode_message(&self, msg: Message) -> Result<Option<Self::Message>, ExchangeError> {
        match msg {
            Message::Text(text) => {
                let parsed: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| ExchangeError::Other(format!("Failed to parse JSON: {}", e)))?;

                Ok(self.parse_message(parsed))
            }
            Message::Binary(data) => {
                // Some exchanges use binary compression
                let text = String::from_utf8(data)
                    .map_err(|e| ExchangeError::Other(format!("Failed to decode binary: {}", e)))?;

                let parsed: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| ExchangeError::Other(format!("Failed to parse JSON: {}", e)))?;

                Ok(self.parse_message(parsed))
            }
            _ => Ok(None), // Ignore other message types
        }
    }
}

impl ParadexCodec {
    #[allow(clippy::too_many_lines)]
    fn parse_message(&self, data: serde_json::Value) -> Option<ParadexWsEvent> {
        // Handle different message types based on the channel or message structure
        if let Some(channel) = data.get("channel").and_then(|c| c.as_str()) {
            match channel {
                "ticker" => {
                    let ticker = Ticker {
                        symbol: data
                            .get("symbol")
                            .and_then(|s| s.as_str())
                            .map(conversion::string_to_symbol)
                            .unwrap_or_default(),
                        price: data
                            .get("price")
                            .and_then(|p| p.as_str())
                            .map(conversion::string_to_price)
                            .unwrap_or_default(),
                        volume: data
                            .get("volume")
                            .and_then(|v| v.as_str())
                            .map(conversion::string_to_volume)
                            .unwrap_or_default(),
                        price_change: data
                            .get("price_change")
                            .and_then(|pc| pc.as_str())
                            .map(conversion::string_to_price)
                            .unwrap_or_default(),
                        price_change_percent: data
                            .get("price_change_percent")
                            .and_then(|pcp| pcp.as_str())
                            .map(conversion::string_to_decimal)
                            .unwrap_or_default(),
                        high_price: data
                            .get("high_price")
                            .and_then(|hp| hp.as_str())
                            .map(conversion::string_to_price)
                            .unwrap_or_default(),
                        low_price: data
                            .get("low_price")
                            .and_then(|lp| lp.as_str())
                            .map(conversion::string_to_price)
                            .unwrap_or_default(),
                        quote_volume: data
                            .get("quote_volume")
                            .and_then(|qv| qv.as_str())
                            .map(conversion::string_to_volume)
                            .unwrap_or_default(),
                        open_time: data
                            .get("open_time")
                            .and_then(|ot| ot.as_i64())
                            .unwrap_or_default(),
                        close_time: data
                            .get("close_time")
                            .and_then(|ct| ct.as_i64())
                            .unwrap_or_default(),
                        count: data
                            .get("count")
                            .and_then(|c| c.as_i64())
                            .unwrap_or_default(),
                    };
                    Some(ParadexWsEvent::Ticker(ticker))
                }
                "orderbook" => {
                    let orderbook = OrderBook {
                        symbol: data
                            .get("symbol")
                            .and_then(|s| s.as_str())
                            .map(conversion::string_to_symbol)
                            .unwrap_or_default(),
                        bids: data
                            .get("bids")
                            .and_then(|b| b.as_array())
                            .map(|bids| {
                                bids.iter()
                                    .filter_map(|bid| {
                                        bid.as_array().and_then(|bid_array| {
                                            if bid_array.len() >= 2 {
                                                Some(OrderBookEntry {
                                                    price: bid_array[0]
                                                        .as_str()
                                                        .map(conversion::string_to_price)
                                                        .unwrap_or_default(),
                                                    quantity: bid_array[1]
                                                        .as_str()
                                                        .map(conversion::string_to_quantity)
                                                        .unwrap_or_default(),
                                                })
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        asks: data
                            .get("asks")
                            .and_then(|a| a.as_array())
                            .map(|asks| {
                                asks.iter()
                                    .filter_map(|ask| {
                                        ask.as_array().and_then(|ask_array| {
                                            if ask_array.len() >= 2 {
                                                Some(OrderBookEntry {
                                                    price: ask_array[0]
                                                        .as_str()
                                                        .map(conversion::string_to_price)
                                                        .unwrap_or_default(),
                                                    quantity: ask_array[1]
                                                        .as_str()
                                                        .map(conversion::string_to_quantity)
                                                        .unwrap_or_default(),
                                                })
                                            } else {
                                                None
                                            }
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        last_update_id: data
                            .get("last_update_id")
                            .and_then(|id| id.as_i64())
                            .unwrap_or_default(),
                    };
                    Some(ParadexWsEvent::OrderBook(orderbook))
                }
                "trade" => {
                    let trade = Trade {
                        symbol: data
                            .get("symbol")
                            .and_then(|s| s.as_str())
                            .map(conversion::string_to_symbol)
                            .unwrap_or_default(),
                        id: data.get("id").and_then(|i| i.as_i64()).unwrap_or_default(),
                        price: data
                            .get("price")
                            .and_then(|p| p.as_str())
                            .map(conversion::string_to_price)
                            .unwrap_or_default(),
                        quantity: data
                            .get("quantity")
                            .and_then(|q| q.as_str())
                            .map(conversion::string_to_quantity)
                            .unwrap_or_default(),
                        time: data
                            .get("time")
                            .and_then(|t| t.as_i64())
                            .unwrap_or_default(),
                        is_buyer_maker: data
                            .get("is_buyer_maker")
                            .and_then(|b| b.as_bool())
                            .unwrap_or_default(),
                    };
                    Some(ParadexWsEvent::Trade(trade))
                }
                "kline" => {
                    let kline = Kline {
                        symbol: data
                            .get("symbol")
                            .and_then(|s| s.as_str())
                            .map(conversion::string_to_symbol)
                            .unwrap_or_default(),
                        open_time: data
                            .get("open_time")
                            .and_then(|ot| ot.as_i64())
                            .unwrap_or_default(),
                        close_time: data
                            .get("close_time")
                            .and_then(|ct| ct.as_i64())
                            .unwrap_or_default(),
                        interval: data
                            .get("interval")
                            .and_then(|i| i.as_str())
                            .unwrap_or("1m")
                            .to_string(),
                        open_price: data
                            .get("open_price")
                            .and_then(|o| o.as_str())
                            .map(conversion::string_to_price)
                            .unwrap_or_default(),
                        high_price: data
                            .get("high_price")
                            .and_then(|h| h.as_str())
                            .map(conversion::string_to_price)
                            .unwrap_or_default(),
                        low_price: data
                            .get("low_price")
                            .and_then(|l| l.as_str())
                            .map(conversion::string_to_price)
                            .unwrap_or_default(),
                        close_price: data
                            .get("close_price")
                            .and_then(|c| c.as_str())
                            .map(conversion::string_to_price)
                            .unwrap_or_default(),
                        volume: data
                            .get("volume")
                            .and_then(|v| v.as_str())
                            .map(conversion::string_to_volume)
                            .unwrap_or_default(),
                        number_of_trades: data
                            .get("number_of_trades")
                            .and_then(|n| n.as_i64())
                            .unwrap_or_default(),
                        final_bar: data
                            .get("final_bar")
                            .and_then(|f| f.as_bool())
                            .unwrap_or(true),
                    };
                    Some(ParadexWsEvent::Kline(kline))
                }
                _ => None, // Unknown channel
            }
        } else {
            // Handle subscription confirmations and other messages
            if data.get("result").is_some() {
                Some(ParadexWsEvent::SubscriptionConfirmation(data))
            } else {
                None
            }
        }
    }
}

/// Helper function to create subscription channels for Paradex WebSocket
pub fn create_subscription_channel(symbol: &str, subscription_type: &SubscriptionType) -> String {
    match subscription_type {
        SubscriptionType::Ticker => format!("ticker@{}", symbol),
        SubscriptionType::OrderBook { depth } => depth.as_ref().map_or_else(
            || format!("depth@{}", symbol),
            |depth| format!("depth{}@{}", depth, symbol),
        ),
        SubscriptionType::Trades => format!("trade@{}", symbol),
        SubscriptionType::Klines { interval } => {
            format!("kline_{}@{}", interval.to_binance_format(), symbol)
        }
    }
}

impl From<ParadexWsEvent> for Option<MarketDataType> {
    fn from(event: ParadexWsEvent) -> Self {
        match event {
            ParadexWsEvent::Ticker(ticker) => Some(MarketDataType::Ticker(ticker)),
            ParadexWsEvent::OrderBook(orderbook) => Some(MarketDataType::OrderBook(orderbook)),
            ParadexWsEvent::Trade(trade) => Some(MarketDataType::Trade(trade)),
            ParadexWsEvent::Kline(kline) => Some(MarketDataType::Kline(kline)),
            ParadexWsEvent::SubscriptionConfirmation(_)
            | ParadexWsEvent::Error(_)
            | ParadexWsEvent::Heartbeat => None,
        }
    }
}
