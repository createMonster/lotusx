# LotusX Kernel Codec Architecture - Usage Guide

## Overview

The LotusX kernel follows **strict separation of concerns**:

- **Transport Layer** (`core/kernel/ws.rs`): Handles TCP/TLS, connection management, ping/pong, reconnection
- **Codec Interface** (`core/kernel/codec.rs`): Defines the `WsCodec` trait only
- **Exchange Codecs** (`exchanges/*/codec.rs`): Exchange-specific message formatting implementations  
- **Application Layer** (`exchanges/*/connector.rs`): Exchange connectors focus on business logic

**❌ CRITICAL RULE**: The kernel contains NO exchange-specific code. All exchange formatting lives in `exchanges/` folders.

## Architecture Diagram

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │    │   Codec Layer   │    │ Transport Layer │
│   (Connector)   │◄──►│  (Exchange      │◄──►│   (Network)     │
│                 │    │   Specific)     │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
      Business              Message               Connection
       Logic                Formatting            Management
```

## Directory Structure

```
src/
├── core/
│   └── kernel/
│       ├── codec.rs      # WsCodec trait ONLY (NO exchange-specific code)
│       ├── ws.rs         # Transport layer (TungsteniteWs, ReconnectWs)
│       ├── rest.rs       # REST client (ReqwestRest, builders)
│       └── signer.rs     # Authentication (HmacSigner, Ed25519Signer, JwtSigner)
└── exchanges/
    ├── binance/
    │   ├── codec.rs      # BinanceCodec + BinanceMessage (ALL formatting logic)
    │   └── connector.rs  # Business logic
    ├── bybit/
    │   ├── codec.rs      # BybitCodec + BybitMessage (ALL formatting logic)
    │   └── connector.rs  # Business logic
    └── hyperliquid/
        ├── codec.rs      # HyperliquidCodec + HyperliquidMessage (ALL formatting logic)
        └── connector.rs  # Business logic
```

## Core Traits (in kernel)

### WsCodec Trait

```rust
// core/kernel/codec.rs - ONLY the trait definition
pub trait WsCodec: Send + Sync + 'static {
    type Message: Send + Sync;

    fn encode_subscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError>;
    
    fn encode_unsubscription(
        &self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<Message, ExchangeError>;
    
    fn decode_message(&self, message: Message) -> Result<Option<Self::Message>, ExchangeError>;
}
```

**Key Changes:**
- ❌ **Removed**: `is_control_message()` and `create_pong_response()` - handled at transport level
- ❌ **Removed**: `SubscriptionBuilder` - each codec builds messages internally  
- ✅ **Improved**: Uses `&[impl AsRef<str>]` to avoid unnecessary string allocations

### WsSession Trait

```rust
// core/kernel/ws.rs
pub trait WsSession<C: WsCodec>: Send + Sync {
    async fn connect(&mut self) -> Result<(), ExchangeError>;
    async fn send_raw(&mut self, msg: Message) -> Result<(), ExchangeError>;
    async fn next_raw(&mut self) -> Option<Result<Message, ExchangeError>>;
    async fn next_message(&mut self) -> Option<Result<C::Message, ExchangeError>>;
    
    async fn subscribe(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError>;
    
    async fn unsubscribe(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError>;
    
    async fn close(&mut self) -> Result<(), ExchangeError>;
    fn is_connected(&self) -> bool;
}
```

**Key Changes:**
- ✅ **Transport handles all control messages** (ping/pong/close) automatically
- ✅ **String slices** instead of owned strings for better performance

## Implementation Examples

### 1. Binance Codec (in `exchanges/binance/codec.rs`)

```rust
use lotusx::core::kernel::WsCodec;
use lotusx::core::errors::ExchangeError;
use serde_json::{json, Map, Value};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone)]
pub enum BinanceMessage {
    Ticker { symbol: String, price: String },
    OrderBook { symbol: String, bids: Vec<(String, String)>, asks: Vec<(String, String)> },
    Trade { symbol: String, price: String, quantity: String },
    Subscription { status: String, id: Option<u64> },
    Unknown(Value),
}

pub struct BinanceCodec;

impl BinanceCodec {
    pub fn new() -> Self {
        Self
    }

    // Internal helper - builds Binance-specific subscription format
    fn build_subscription_message(&self, streams: &[impl AsRef<str>]) -> Value {
        let mut msg = Map::new();
        msg.insert("method".to_string(), Value::String("SUBSCRIBE".to_string()));
        msg.insert("params".to_string(), Value::Array(
            streams.iter().map(|s| Value::String(s.as_ref().to_string())).collect()
        ));
        msg.insert("id".to_string(), Value::Number(1.into()));
        Value::Object(msg)
    }

    fn build_unsubscription_message(&self, streams: &[impl AsRef<str>]) -> Value {
        let mut msg = Map::new();
        msg.insert("method".to_string(), Value::String("UNSUBSCRIBE".to_string()));
        msg.insert("params".to_string(), Value::Array(
            streams.iter().map(|s| Value::String(s.as_ref().to_string())).collect()
        ));
        msg.insert("id".to_string(), Value::Number(1.into()));
        Value::Object(msg)
    }
}

impl WsCodec for BinanceCodec {
    type Message = BinanceMessage;

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
                let value: Value = serde_json::from_str(&text)
                    .map_err(|e| ExchangeError::DeserializationError(format!("JSON parse error: {}", e)))?;
                
                // Handle subscription confirmations
                if let Some(result) = value.get("result") {
                    if result.is_null() {
                        return Ok(Some(BinanceMessage::Subscription {
                            status: "confirmed".to_string(),
                            id: value.get("id").and_then(|id| id.as_u64()),
                        }));
                    }
                }

                // Handle stream data
                if let Some(stream) = value.get("stream").and_then(|s| s.as_str()) {
                    if let Some(data) = value.get("data") {
                        return Ok(Some(self.parse_stream_data(stream, data)));
                    }
                }

                Ok(Some(BinanceMessage::Unknown(value)))
            }
            _ => Ok(None), // Ignore non-text messages
        }
    }
}

impl BinanceCodec {
    fn parse_stream_data(&self, stream: &str, data: &Value) -> BinanceMessage {
        if stream.contains("@ticker") {
            BinanceMessage::Ticker {
                symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                price: data.get("c").and_then(|c| c.as_str()).unwrap_or("0").to_string(),
            }
        } else if stream.contains("@depth") {
            BinanceMessage::OrderBook {
                symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                bids: self.parse_order_book_side(data.get("b")),
                asks: self.parse_order_book_side(data.get("a")),
            }
        } else {
            BinanceMessage::Unknown(data.clone())
        }
    }

    fn parse_order_book_side(&self, side: Option<&Value>) -> Vec<(String, String)> {
        side.and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let arr = item.as_array()?;
                        let price = arr.first()?.as_str()?;
                        let qty = arr.get(1)?.as_str()?;
                        Some((price.to_string(), qty.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
```

### 2. Bybit Codec (in `exchanges/bybit/codec.rs`)

```rust
use lotusx::core::kernel::WsCodec;
use lotusx::core::errors::ExchangeError;
use serde_json::{json, Map, Value};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone)]
pub enum BybitMessage {
    Ticker { symbol: String, price: String },
    OrderBook { symbol: String, bids: Vec<(String, String)>, asks: Vec<(String, String)> },
    Subscription { status: String },
    Heartbeat,
    Unknown(Value),
}

pub struct BybitCodec;

impl BybitCodec {
    pub fn new() -> Self {
        Self
    }

    // Internal helper - builds Bybit-specific subscription format
    fn build_subscription_message(&self, streams: &[impl AsRef<str>]) -> Value {
        let mut msg = Map::new();
        msg.insert("op".to_string(), Value::String("subscribe".to_string()));
        msg.insert("args".to_string(), Value::Array(
            streams.iter().map(|s| Value::String(s.as_ref().to_string())).collect()
        ));
        Value::Object(msg)
    }

    fn build_unsubscription_message(&self, streams: &[impl AsRef<str>]) -> Value {
        let mut msg = Map::new();
        msg.insert("op".to_string(), Value::String("unsubscribe".to_string()));
        msg.insert("args".to_string(), Value::Array(
            streams.iter().map(|s| Value::String(s.as_ref().to_string())).collect()
        ));
        Value::Object(msg)
    }
}

impl WsCodec for BybitCodec {
    type Message = BybitMessage;

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
                let value: Value = serde_json::from_str(&text)
                    .map_err(|e| ExchangeError::DeserializationError(format!("JSON parse error: {}", e)))?;
                
                // Handle subscription confirmations
                if let Some(op) = value.get("op").and_then(|o| o.as_str()) {
                    if op == "subscribe" {
                        return Ok(Some(BybitMessage::Subscription {
                            status: if value.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
                                "confirmed".to_string()
                            } else {
                                "failed".to_string()
                            },
                        }));
                    }
                }

                // Handle pong responses
                if value.get("op").and_then(|o| o.as_str()) == Some("pong") {
                    return Ok(Some(BybitMessage::Heartbeat));
                }

                // Handle topic data
                if let Some(topic) = value.get("topic").and_then(|t| t.as_str()) {
                    if let Some(data) = value.get("data") {
                        return Ok(Some(self.parse_topic_data(topic, data)));
                    }
                }

                Ok(Some(BybitMessage::Unknown(value)))
            }
            _ => Ok(None), // Ignore non-text messages
        }
    }
}

impl BybitCodec {
    fn parse_topic_data(&self, topic: &str, data: &Value) -> BybitMessage {
        if topic.starts_with("tickers") {
            BybitMessage::Ticker {
                symbol: data.get("symbol").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                price: data.get("lastPrice").and_then(|p| p.as_str()).unwrap_or("0").to_string(),
            }
        } else if topic.starts_with("orderbook") {
            BybitMessage::OrderBook {
                symbol: data.get("s").and_then(|s| s.as_str()).unwrap_or("").to_string(),
                bids: self.parse_order_book_side(data.get("b")),
                asks: self.parse_order_book_side(data.get("a")),
            }
        } else {
            BybitMessage::Unknown(data.clone())
        }
    }

    fn parse_order_book_side(&self, side: Option<&Value>) -> Vec<(String, String)> {
        side.and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let arr = item.as_array()?;
                        let price = arr.first()?.as_str()?;
                        let qty = arr.get(1)?.as_str()?;
                        Some((price.to_string(), qty.to_string()))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
```

## Usage Examples

### 1. Using Binance Codec

```rust
use lotusx::core::kernel::{TungsteniteWs, WsSession};
use your_project::exchanges::binance::codec::{BinanceCodec, BinanceMessage};

#[tokio::main]
async fn main() -> Result<(), ExchangeError> {
    // Create Binance codec (lives in exchanges/binance/codec.rs)
    let codec = BinanceCodec::new();
    
    // Create WebSocket session with codec
    let mut ws = TungsteniteWs::new(
        "wss://stream.binance.com/ws".to_string(),
        "binance".to_string(),
        codec
    );
    
    // Connect and use (note: using string slices, not owned strings)
    ws.connect().await?;
    ws.subscribe(&["btcusdt@ticker", "ethusdt@ticker"]).await?;
    
    // Process exchange-specific messages
    while let Some(result) = ws.next_message().await {
        match result? {
            BinanceMessage::Ticker { symbol, price } => {
                println!("Binance Ticker: {} = {}", symbol, price);
            }
            BinanceMessage::Subscription { status, id } => {
                println!("Binance Subscription {}: {:?}", status, id);
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

### 2. Using Bybit Codec with Reconnection

```rust
use lotusx::core::kernel::{TungsteniteWs, ReconnectWs, WsSession};
use your_project::exchanges::bybit::codec::{BybitCodec, BybitMessage};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), ExchangeError> {
    // Create Bybit codec (lives in exchanges/bybit/codec.rs)
    let codec = BybitCodec::new();
    
    // Create WebSocket session with reconnection
    let base_ws = TungsteniteWs::new(
        "wss://stream.bybit.com/v5/public/spot".to_string(),
        "bybit".to_string(),
        codec
    );
    
    let mut ws = ReconnectWs::new(base_ws)
        .with_max_reconnect_attempts(10)
        .with_reconnect_delay(Duration::from_secs(2))
        .with_auto_resubscribe(true);
    
    ws.connect().await?;
    ws.subscribe(&["orderbook.1.BTCUSDT", "tickers.BTCUSDT"]).await?;
    
    // Process exchange-specific messages  
    while let Some(result) = ws.next_message().await {
        match result? {
            BybitMessage::Ticker { symbol, price } => {
                println!("Bybit Ticker: {} = {}", symbol, price);
            }
            BybitMessage::OrderBook { symbol, bids, asks } => {
                println!("Bybit OrderBook {}: {} bids, {} asks", symbol, bids.len(), asks.len());
            }
            BybitMessage::Heartbeat => {
                println!("Bybit Heartbeat");
            }
            _ => {}
        }
    }
    
    Ok(())
}
```

### 3. Custom Exchange Codec (in `exchanges/myexchange/codec.rs`)

```rust
use lotusx::core::kernel::WsCodec;
use lotusx::core::errors::ExchangeError;
use serde_json::{json, Map, Value};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug, Clone)]
pub enum MyExchangeMessage {
    Price { symbol: String, value: f64 },
    Volume { symbol: String, amount: f64 },
    Status { message: String },
}

pub struct MyExchangeCodec;

impl MyExchangeCodec {
    pub fn new() -> Self {
        Self
    }

    // Internal helper - builds custom exchange subscription format
    fn build_subscription_message(&self, streams: &[impl AsRef<str>]) -> Value {
        let mut msg = Map::new();
        msg.insert("sub".to_string(), Value::String("data".to_string()));
        msg.insert("topics".to_string(), Value::Array(
            streams.iter().map(|s| Value::String(s.as_ref().to_string())).collect()
        ));
        msg.insert("req_id".to_string(), Value::Number(1.into()));
        Value::Object(msg)
    }

    fn build_unsubscription_message(&self, streams: &[impl AsRef<str>]) -> Value {
        let mut msg = Map::new();
        msg.insert("unsub".to_string(), Value::String("data".to_string()));
        msg.insert("topics".to_string(), Value::Array(
            streams.iter().map(|s| Value::String(s.as_ref().to_string())).collect()
        ));
        msg.insert("req_id".to_string(), Value::Number(1.into()));
        Value::Object(msg)
    }
}

impl WsCodec for MyExchangeCodec {
    type Message = MyExchangeMessage;

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
                let value: Value = serde_json::from_str(&text)
                    .map_err(|e| ExchangeError::DeserializationError(format!("JSON parse error: {}", e)))?;
                
                // Parse your exchange's specific format
                if let Some(data_type) = value.get("type").and_then(|t| t.as_str()) {
                    match data_type {
                        "price" => {
                            let symbol = value.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
                            let price = value.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0);
                            Ok(Some(MyExchangeMessage::Price { 
                                symbol: symbol.to_string(), 
                                value: price 
                            }))
                        }
                        "volume" => {
                            let symbol = value.get("symbol").and_then(|s| s.as_str()).unwrap_or("");
                            let amount = value.get("volume").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            Ok(Some(MyExchangeMessage::Volume { 
                                symbol: symbol.to_string(), 
                                amount 
                            }))
                        }
                        _ => Ok(Some(MyExchangeMessage::Status { 
                            message: format!("Unknown type: {}", data_type) 
                        }))
                    }
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None)
        }
    }
}
```

## Benefits of This Architecture

### ✅ **Proper Separation of Concerns**
- **Kernel**: Contains ONLY transport logic and generic interfaces
- **Exchange folders**: Contain ALL exchange-specific code including message formatting
- **Zero coupling**: Kernel compiles without knowing about any exchange

### ✅ **Single Responsibility Principle**
- **Transport** (`ws.rs`): Network connections, ping/pong, reconnection
- **Codec Interface** (`codec.rs`): Generic message formatting contract only
- **Exchange Codecs** (`exchanges/*/codec.rs`): Exchange-specific formatting logic
- **Connectors** (`exchanges/*/connector.rs`): Business logic

### ✅ **Open/Closed Principle**
- Adding new exchange = create new folder `exchanges/new_exchange/`
- Implement `WsCodec` trait for the new exchange with internal message building
- Zero modifications to kernel code

### ✅ **Performance Optimizations**
- String slices (`&str`) instead of owned strings where possible
- Raw bytes in signer API instead of JSON serialization
- Minimal allocations in hot paths

### ✅ **Testability**
```rust
// Test kernel transport in isolation
#[test]
fn test_transport_with_mock_codec() {
    struct MockCodec;
    impl WsCodec for MockCodec {
        type Message = String;
        fn encode_subscription(&self, streams: &[impl AsRef<str> + Send + Sync]) -> Result<Message, ExchangeError> {
            Ok(Message::Text(format!("mock_sub:{}", streams.len())))
        }
        // ... minimal mock implementation
    }
    
    let mock_codec = MockCodec;
    let mut ws = TungsteniteWs::new("ws://test", "test", mock_codec);
    // Test only transport functionality
}

// Test exchange codec in isolation
#[test]
fn test_binance_codec_decode() {
    let codec = BinanceCodec::new();
    let message = Message::Text(r#"{"stream":"btcusdt@ticker","data":{"s":"BTCUSDT","c":"50000"}}"#);
    let result = codec.decode_message(message).unwrap();
    // Test only codec functionality
}
```

### ✅ **Dependency Inversion**
- Transport depends on `WsCodec` trait, not concrete implementations
- Easy to swap codecs for testing or different exchanges
- Clear boundaries between layers

## Migration Guidelines

1. **Create exchange codec files**: Add `codec.rs` to each `exchanges/*/` folder
2. **Define message types**: Create exchange-specific message enums in each codec
3. **Implement WsCodec trait**: Each exchange implements message formatting with internal builders
4. **Update connectors**: Use the new codec-based WebSocket sessions
5. **Remove old code**: Delete legacy WebSocket managers with embedded formatting

## Key Rule: No Exchange Logic in Kernel!

The kernel is **transport-only**. All exchange-specific code lives in `exchanges/` folders. This ensures:
- Clean separation of concerns
- Easy testing of transport vs. formatting logic
- Simple addition of new exchanges
- Stable, reusable kernel foundation

## Current API Status

✅ **Kernel is Complete and Stable**  
✅ **Ready for Exchange Codec Implementation**  
✅ **All Quality Checks Passing**  
✅ **Performance Optimized**