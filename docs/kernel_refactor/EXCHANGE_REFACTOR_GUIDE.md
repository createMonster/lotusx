# Exchange Refactor Guide: Migrating to Kernel Architecture

This guide provides a comprehensive blueprint for refactoring any exchange connector to use the LotusX Kernel Architecture, based on the successful Backpack migration. The kernel provides a unified, type-safe, and observable foundation for all exchange integrations.

## 🎯 Overview

The kernel architecture separates **transport concerns** from **exchange-specific logic**, enabling:
- **Zero-copy typed deserialization** for optimal performance
- **Unified error handling** across all exchanges
- **Pluggable authentication** with exchange-specific signers
- **Observable operations** with built-in tracing
- **Testable components** through dependency injection

## 🏗️ Architecture Principles

### 1. Separation of Concerns
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Application   │◄──►│    Connector     │◄──►│   Kernel        │
│   (Traits)      │    │  (Exchange-Specific) │    │   (Transport)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

- **Kernel**: Transport, authentication, observability (exchange-agnostic)
- **Connector**: Field mapping, endpoint configuration (exchange-specific)  
- **Application**: Business logic via traits (exchange-agnostic)

### 2. Key Kernel Components

| Component | Purpose | Exchange Implementation Required |
|-----------|---------|----------------------------------|
| `RestClient` | HTTP transport with signing | ❌ (provided by kernel) |
| `WsSession` | WebSocket transport | ❌ (provided by kernel) |
| `WsCodec` | Message encoding/decoding | ✅ (exchange-specific) |
| `Signer` | Request authentication | ✅ (exchange-specific) |

## 📋 Migration Checklist

### Phase 1: Kernel Integration
- [ ] Create exchange-specific `WsCodec` implementation
- [ ] Create exchange-specific `Signer` implementation  
- [ ] Refactor connector to use kernel `RestClient`
- [ ] Refactor connector to use kernel `WsSession`
- [ ] Update all methods to return strongly-typed responses

### Phase 2: Trait Compliance
- [ ] Implement `MarketDataSource` trait
- [ ] Implement `AccountInfo` trait
- [ ] Implement `OrderPlacer` trait (if supported)
- [ ] Implement `FundingRateSource` trait (if supported)

### Phase 3: Quality & Testing
- [ ] Add comprehensive error handling
- [ ] Add tracing instrumentation
- [ ] Create factory functions
- [ ] Update examples and tests
- [ ] Verify performance benchmarks

## 🔧 Step-by-Step Refactoring

### Step 1: Define Exchange Types

Create strongly-typed response structures in `types.rs`:

```rust
// Response types matching API schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeMarketResponse {
    pub symbol: String,
    pub base_asset: String,
    pub quote_asset: String,
    // ... other fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeTickerResponse {
    pub symbol: String,
    pub price: String,
    pub volume: String,
    // ... other fields
}
```

### Step 2: Implement WsCodec

Create `codec.rs` with exchange-specific message handling:

```rust
use crate::core::kernel::WsCodec;

pub struct ExchangeCodec;

impl WsCodec for ExchangeCodec {
    type Message = ExchangeMessage;

    fn encode_subscribe(&self, streams: &[String]) -> Result<String, ExchangeError> {
        let subscription = json!({
            "method": "SUBSCRIBE",
            "params": streams,
            "id": 1
        });
        Ok(subscription.to_string())
    }

    fn encode_unsubscribe(&self, streams: &[String]) -> Result<String, ExchangeError> {
        let unsubscription = json!({
            "method": "UNSUBSCRIBE", 
            "params": streams,
            "id": 1
        });
        Ok(unsubscription.to_string())
    }

    fn decode_message(&self, text: &str) -> Result<Self::Message, ExchangeError> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        
        // Parse exchange-specific message format
        if let Some(event_type) = value.get("e").and_then(|e| e.as_str()) {
            match event_type {
                "ticker" => Ok(ExchangeMessage::Ticker(serde_json::from_value(value)?)),
                "trade" => Ok(ExchangeMessage::Trade(serde_json::from_value(value)?)),
                _ => Ok(ExchangeMessage::Unknown),
            }
        } else {
            Ok(ExchangeMessage::Unknown)
        }
    }
}
```

### Step 3: Implement Signer

Create exchange-specific authentication in `auth.rs`:

```rust
use crate::core::kernel::Signer;

pub struct ExchangeSigner {
    api_key: String,
    secret_key: String,
}

impl Signer for ExchangeSigner {
    fn sign_request(
        &self,
        method: &str,
        endpoint: &str,
        query_string: &str,
        body: &[u8],
        timestamp: u64,
    ) -> Result<(HashMap<String, String>, Vec<(String, String)>), ExchangeError> {
        // Exchange-specific signing logic
        let signature = self.create_signature(method, endpoint, query_string, body, timestamp)?;
        
        let mut headers = HashMap::new();
        headers.insert("X-API-KEY".to_string(), self.api_key.clone());
        
        let mut params = vec![];
        params.push(("signature".to_string(), signature));
        params.push(("timestamp".to_string(), timestamp.to_string()));
        
        Ok((headers, params))
    }
}
```

### Step 4: Refactor Connector

Transform the connector to use kernel components:

```rust
use crate::core::kernel::{RestClient, WsSession};

pub struct ExchangeConnector<R: RestClient, W: WsSession<ExchangeCodec>> {
    rest: R,
    ws: Option<W>,
    config: ExchangeConfig,
}

impl<R: RestClient, W: WsSession<ExchangeCodec>> ExchangeConnector<R, W> {
    pub fn new(rest: R, ws: Option<W>, config: ExchangeConfig) -> Self {
        Self { rest, ws, config }
    }

    // Use strongly-typed responses
    pub async fn get_markets(&self) -> Result<Vec<ExchangeMarketResponse>, ExchangeError> {
        self.rest.get_json("/api/v1/markets", &[], false).await
    }

    pub async fn get_ticker(&self, symbol: &str) -> Result<ExchangeTickerResponse, ExchangeError> {
        let params = [("symbol", symbol)];
        self.rest.get_json("/api/v1/ticker", &params, false).await
    }
}
```

### Step 5: Implement Traits

Implement standard traits for interoperability:

```rust
#[async_trait]
impl<R: RestClient, W: WsSession<ExchangeCodec>> MarketDataSource for ExchangeConnector<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let markets: Vec<ExchangeMarketResponse> = self.get_markets().await?;
        
        Ok(markets.into_iter().map(|m| Market {
            symbol: Symbol {
                base: m.base_asset,
                quote: m.quote_asset,
            },
            status: m.status,
            // ... field mapping
        }).collect())
    }
}
```

### Step 6: Create Factory Functions

Provide convenient constructors in `mod.rs`:

```rust
pub fn create_exchange_connector(
    config: ExchangeConfig,
    with_websocket: bool,
) -> Result<ExchangeConnector<ReqwestRest, Option<TungsteniteWs<ExchangeCodec>>>, ExchangeError> {
    // Build REST client
    let rest_config = RestClientConfig::new(
        "https://api.exchange.com".to_string(),
        "exchange".to_string(),
    );
    
    let mut rest_builder = RestClientBuilder::new(rest_config);
    
    if config.has_credentials() {
        let signer = Arc::new(ExchangeSigner::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }
    
    let rest = rest_builder.build()?;
    
    // Build WebSocket client (optional)
    let ws = if with_websocket {
        let ws_config = WsConfig::new("wss://stream.exchange.com".to_string());
        let codec = ExchangeCodec;
        Some(TungsteniteWs::new(ws_config, codec)?)
    } else {
        None
    };
    
    Ok(ExchangeConnector::new(rest, ws, config))
}
```

## 🎯 Best Practices

### 1. Strongly-Typed Responses

**❌ Before (manual parsing):**
```rust
pub async fn get_ticker(&self, symbol: &str) -> Result<serde_json::Value, ExchangeError> {
    let response: serde_json::Value = self.rest.get("/api/ticker", &params, false).await?;
    let ticker: TickerResponse = serde_json::from_value(response).map_err(|e| {
        ExchangeError::DeserializationError(format!("Failed to parse ticker: {}", e))
    })?;
    // ... manual conversion
}
```

**✅ After (zero-copy typed):**
```rust
pub async fn get_ticker(&self, symbol: &str) -> Result<TickerResponse, ExchangeError> {
    let params = [("symbol", symbol)];
    self.rest.get_json("/api/ticker", &params, false).await
}
```

### 2. Error Handling

Use consistent error types and tracing:

```rust
#[instrument(skip(self), fields(exchange = "exchange_name", symbol = %symbol))]
pub async fn get_ticker(&self, symbol: &str) -> Result<TickerResponse, ExchangeError> {
    self.rest.get_json("/api/ticker", &[("symbol", symbol)], false).await
}
```

### 3. Configuration Management

Separate configuration from business logic:

```rust
pub struct ExchangeConfig {
    api_key: String,
    secret_key: String,
    testnet: bool,
    base_url: Option<String>,
}

impl ExchangeConfig {
    pub fn has_credentials(&self) -> bool {
        !self.api_key.is_empty() && !self.secret_key.is_empty()
    }
}
```

### 4. WebSocket Stream Helpers

Provide utility functions for stream management:

```rust
pub fn create_exchange_stream_identifiers(
    symbols: &[String],
    subscription_types: &[SubscriptionType],
) -> Vec<String> {
    let mut streams = Vec::new();
    
    for symbol in symbols {
        for sub_type in subscription_types {
            match sub_type {
                SubscriptionType::Ticker => streams.push(format!("{}@ticker", symbol.to_lowercase())),
                SubscriptionType::Trades => streams.push(format!("{}@trade", symbol.to_lowercase())),
                SubscriptionType::OrderBook { depth } => {
                    let depth_str = depth.map_or("".to_string(), |d| format!("@{}", d));
                    streams.push(format!("{}@depth{}", symbol.to_lowercase(), depth_str));
                }
            }
        }
    }
    
    streams
}
```

## 🔍 Migration Validation

### Compilation Checks
```bash
# Verify clean compilation
cargo check --all-features

# Run clippy for best practices
cargo clippy --all-targets --all-features -- -D warnings

# Ensure formatting consistency
cargo fmt --all
```

### Functional Testing
```bash
# Run existing tests to ensure compatibility
cargo test

# Run exchange-specific integration tests  
cargo test --test exchange_integration_tests

# Verify examples still work
cargo run --example exchange_example
```

### Performance Validation
```bash
# Run latency benchmarks
cargo run --example latency_test

# Compare memory usage before/after
cargo run --example memory_benchmark
```

## 📊 Expected Outcomes

### Before Refactor
- ❌ Manual JSON parsing with error-prone `serde_json::from_value`
- ❌ Inconsistent error handling across methods
- ❌ Mixed transport and business logic
- ❌ Difficult testing due to tight coupling
- ❌ No observability or tracing

### After Refactor  
- ✅ **Zero-copy typed deserialization** for optimal performance
- ✅ **Consistent error handling** with proper error propagation
- ✅ **Clean separation** of transport vs business logic
- ✅ **Testable components** through dependency injection
- ✅ **Full observability** with structured tracing
- ✅ **Type safety** with compile-time guarantees
- ✅ **Reduced code complexity** (~60% less boilerplate)

## 🚀 Exchange-Specific Considerations

### Authentication Patterns

**HMAC-SHA256 (Binance, Bybit):**
```rust
impl Signer for HmacSigner {
    fn sign_request(&self, method: &str, endpoint: &str, query_string: &str, body: &[u8], timestamp: u64) -> Result<...> {
        let payload = format!("{}{}{}timestamp={}", method, endpoint, query_string, timestamp);
        let signature = hmac_sha256(&self.secret_key, payload.as_bytes());
        // ... return headers and params
    }
}
```

**Ed25519 (Backpack, dYdX):**
```rust
impl Signer for Ed25519Signer {
    fn sign_request(&self, method: &str, endpoint: &str, query_string: &str, body: &[u8], timestamp: u64) -> Result<...> {
        let instruction = format!("instruction={}¶ms={}", endpoint, query_string);
        let signature = self.signing_key.sign(instruction.as_bytes());
        // ... return headers and params
    }
}
```

### WebSocket Message Formats

**Standard JSON (Most exchanges):**
```rust
fn decode_message(&self, text: &str) -> Result<Self::Message, ExchangeError> {
    let value: serde_json::Value = serde_json::from_str(text)?;
    // Parse based on event type or stream name
}
```

**Binary/Compressed (Some exchanges):**
```rust
fn decode_message(&self, text: &str) -> Result<Self::Message, ExchangeError> {
    // Handle compression/decompression if needed
    let decompressed = decompress_if_needed(text)?;
    let value: serde_json::Value = serde_json::from_str(&decompressed)?;
    // ... parse message
}
```

## 📝 Summary

This kernel architecture refactor delivers:

1. **Performance**: Zero-copy deserialization and reduced allocations
2. **Maintainability**: Clear separation of concerns and reduced complexity  
3. **Reliability**: Type safety and comprehensive error handling
4. **Observability**: Built-in tracing and metrics collection
5. **Testability**: Dependency injection enables comprehensive testing
6. **Scalability**: Consistent patterns across all exchanges

Follow this guide to migrate any exchange to the kernel architecture, ensuring consistent quality and performance across the entire LotusX ecosystem. 