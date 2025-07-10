# Exchange Refactor Guide: Kernel Architecture Implementation

This guide provides a **comprehensive blueprint** for refactoring any exchange connector to the **LotusX Kernel Architecture**. It's based on the proven `structure_exchange.md` template and successful refactoring of **Binance** and **Backpack** exchanges.

## 🎯 Overview

The kernel architecture achieves **one responsibility per file** while maintaining **compile-time type safety** and avoiding transport-level details leaking into business logic:

- **✅ Zero-copy typed deserialization** for HFT performance
- **✅ Unified error handling** across all exchanges  
- **✅ Pluggable authentication** with exchange-specific signers
- **✅ Observable operations** with built-in tracing
- **✅ Testable components** through dependency injection

## 🏗️ Template Structure (Proven & Battle-Tested)

```
src/exchanges/<exchange>/         # e.g., binance, bybit, okx
├── mod.rs                       # public façade, re-exports
├── types.rs                     # serde structs ← raw JSON  
├── conversions.rs              # String ↔︎ Decimal, Symbol, etc.
├── signer.rs                   # Hmac / Ed25519 / JWT
├── codec.rs                    # impl WsCodec (WebSocket dialect)
├── rest.rs                     # thin typed wrapper around RestClient
├── connector/
│   ├── market_data.rs          # impl MarketDataSource
│   ├── trading.rs              # impl TradingEngine (orders)
│   ├── account.rs              # impl AccountInfoSource
│   └── mod.rs                  # re-export, compose sub-traits
└── builder.rs                  # fluent builder → concrete connector
```

## 📋 Refactoring Checklist

### Phase 1: File Structure Migration
- [ ] **Rename files** following template (auth.rs → signer.rs, converters.rs → conversions.rs)
- [ ] **Create connector/ subdirectory** with sub-trait implementations
- [ ] **Create rest.rs** with thin typed wrapper around RestClient
- [ ] **Create builder.rs** with fluent builder pattern
- [ ] **Delete legacy files** (client.rs, connector.rs if monolithic)

### Phase 2: Core Implementation
- [ ] **Update types.rs** to match actual API response schemas
- [ ] **Implement codec.rs** for WebSocket message encode/decode
- [ ] **Implement signer.rs** for exchange-specific authentication
- [ ] **Implement rest.rs** with strongly-typed endpoint methods
- [ ] **Update conversions.rs** with type-safe conversion utilities

### Phase 3: Sub-Trait Implementation  
- [ ] **Implement connector/market_data.rs** with MarketDataSource trait
- [ ] **Implement connector/trading.rs** with OrderPlacer trait (if supported)
- [ ] **Implement connector/account.rs** with AccountInfo trait
- [ ] **Implement connector/mod.rs** with composition pattern
- [ ] **Update mod.rs** to act as public facade with re-exports

### Phase 4: Builder & Factory
- [ ] **Implement builder.rs** with dependency injection pattern
- [ ] **Add legacy compatibility functions** for backward compatibility
- [ ] **Test all build variants** (REST-only, WebSocket, reconnection)

### Phase 5: Quality Assurance
- [ ] **Run quality checks** (`make quality`)
- [ ] **Fix compilation errors** and clippy warnings
- [ ] **Verify trait implementations** work correctly
- [ ] **Test examples** work with new API

## 🔧 Implementation Guide

### 1. REST Client Wrapper (rest.rs)

Create a **thin typed wrapper** around the kernel's RestClient:

```rust
use crate::core::kernel::RestClient;
use crate::core::errors::ExchangeError;
use crate::exchanges::<exchange>::types::*;

/// Thin typed wrapper around `RestClient` for <Exchange> API
pub struct <Exchange>RestClient<R: RestClient> {
    client: R,
}

impl<R: RestClient> <Exchange>RestClient<R> {
    pub fn new(client: R) -> Self {
        Self { client }
    }

    /// Get all markets
    pub async fn get_markets(&self) -> Result<Vec<<Exchange>MarketResponse>, ExchangeError> {
        self.client.get_json("/api/v1/markets", &[], false).await
    }

    /// Get ticker for symbol
    pub async fn get_ticker(&self, symbol: &str) -> Result<<Exchange>TickerResponse, ExchangeError> {
        let params = [("symbol", symbol)];
        self.client.get_json("/api/v1/ticker", &params, false).await
    }

    // Add other endpoints...
}
```

### 2. Sub-Trait Implementations (connector/)

#### market_data.rs - MarketDataSource Implementation

```rust
use crate::core::traits::MarketDataSource;
use crate::exchanges::<exchange>::rest::<Exchange>RestClient;

/// Market data implementation for <Exchange>
pub struct MarketData<R: RestClient, W = ()> {
    rest: <Exchange>RestClient<R>,
    ws: Option<W>,
}

impl<R: RestClient + Clone> MarketData<R, ()> {
    pub fn new(rest: &R, _ws: Option<()>) -> Self {
        Self {
            rest: <Exchange>RestClient::new(rest.clone()),
            ws: None,
        }
    }
}

#[async_trait]
impl<R: RestClient + Clone> MarketDataSource for MarketData<R, ()> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let markets = self.rest.get_markets().await?;
        Ok(markets.into_iter().map(convert_<exchange>_market).collect())
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let klines = self.rest.get_klines(&symbol, interval, limit, start_time, end_time).await?;
        Ok(klines.into_iter().map(|k| convert_<exchange>_kline(&k, &symbol)).collect())
    }

    // Other trait methods...
}
```

#### trading.rs - OrderPlacer Implementation  

```rust
use crate::core::traits::OrderPlacer;

/// Trading implementation for <Exchange>
pub struct Trading<R: RestClient> {
    rest: <Exchange>RestClient<R>,
}

impl<R: RestClient> Trading<R> {
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: <Exchange>RestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient> OrderPlacer for Trading<R> {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let <exchange>_order = convert_order_request(&order)?;
        let response = self.rest.place_order(&<exchange>_order).await?;
        convert_order_response(&response, &order)
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        self.rest.cancel_order(&symbol, &order_id).await?;
        Ok(())
    }
}
```

#### connector/mod.rs - Composition Pattern

```rust
use crate::core::traits::{AccountInfo, MarketDataSource, OrderPlacer};

pub mod account;
pub mod market_data;
pub mod trading;

pub use account::Account;
pub use market_data::MarketData;
pub use trading::Trading;

/// <Exchange> connector that composes all sub-trait implementations
pub struct <Exchange>Connector<R: RestClient, W = ()> {
    pub market: MarketData<R, W>,
    pub trading: Trading<R>,
    pub account: Account<R>,
}

impl<R: RestClient + Clone + Send + Sync> <Exchange>Connector<R, ()> {
    pub fn new_without_ws(rest: R, _config: ExchangeConfig) -> Self {
        Self {
            market: MarketData::<R, ()>::new(&rest, None),
            trading: Trading::new(&rest),
            account: Account::new(&rest),
        }
    }
}

// Implement traits for the connector by delegating to sub-components
#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> MarketDataSource for <Exchange>Connector<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        self.market.get_markets().await
    }
    // Delegate other methods...
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> OrderPlacer for <Exchange>Connector<R, W> {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        self.trading.place_order(order).await
    }
    // Delegate other methods...
}
```

### 3. Builder Pattern (builder.rs)

```rust
use crate::core::config::ExchangeConfig;
use crate::core::kernel::{RestClientBuilder, RestClientConfig, TungsteniteWs};

/// Create a <Exchange> connector with REST-only support
pub fn build_connector(
    config: ExchangeConfig,
) -> Result<<Exchange>Connector<crate::core::kernel::ReqwestRest, ()>, ExchangeError> {
    let base_url = config.base_url.clone()
        .unwrap_or_else(|| "https://api.<exchange>.com".to_string());

    let rest_config = RestClientConfig::new(base_url, "<exchange>".to_string())
        .with_timeout(30)
        .with_max_retries(3);

    let mut rest_builder = RestClientBuilder::new(rest_config);

    if config.has_credentials() {
        let signer = Arc::new(<Exchange>Signer::new(
            config.api_key().to_string(),
            config.secret_key().to_string(),
        ));
        rest_builder = rest_builder.with_signer(signer);
    }

    let rest = rest_builder.build()?;
    Ok(<Exchange>Connector::new_without_ws(rest, config))
}

/// Legacy compatibility functions
pub fn create_<exchange>_connector(
    config: ExchangeConfig,
) -> Result<<Exchange>Connector<ReqwestRest, TungsteniteWs<<Exchange>Codec>>, ExchangeError> {
    build_connector_with_websocket(config)
}
```

### 4. Public Facade (mod.rs)

```rust
pub mod codec;
pub mod conversions; 
pub mod signer;
pub mod types;

pub mod rest;
pub mod connector;
pub mod builder;

// Re-export main components
pub use builder::{
    build_connector,
    build_connector_with_websocket,
    build_connector_with_reconnection,
    // Legacy compatibility exports
    create_<exchange>_connector,
    create_<exchange>_connector_with_reconnection,
};
pub use codec::<Exchange>Codec;
pub use connector::{<Exchange>Connector, Account, MarketData, Trading};

// Helper functions if needed
pub fn create_<exchange>_stream_identifiers(
    symbols: &[String],
    subscription_types: &[crate::core::types::SubscriptionType],
) -> Vec<String> {
    // Exchange-specific stream format logic
}
```

## 🎯 Migration Benefits

### Before (Monolithic)
```rust
// ❌ Everything mixed together
pub struct ExchangeConnector {
    pub client: reqwest::Client,
    // Direct HTTP/WS concerns
    // Business logic mixed with transport
}

impl ExchangeConnector {
    // ❌ 500+ line file with everything
    pub async fn get_markets() { /* HTTP details */ }
    pub async fn place_order() { /* More HTTP details */ }
    // No trait compliance, hard to test
}
```

### After (Kernel Architecture)
```rust  
// ✅ Clean separation of concerns
pub struct ExchangeConnector<R: RestClient, W = ()> {
    pub market: MarketData<R, W>,      // Focused responsibility
    pub trading: Trading<R>,           // Focused responsibility  
    pub account: Account<R>,           // Focused responsibility
}

// ✅ Trait compliance for interoperability
impl<R, W> MarketDataSource for ExchangeConnector<R, W> { /* delegate */ }
impl<R, W> OrderPlacer for ExchangeConnector<R, W> { /* delegate */ }

// ✅ Easy testing, type safety, maintainability
```

## 🚀 Success Metrics

After successful refactoring, you should achieve:

1. **✅ Compilation Success**: `cargo check --lib` passes
2. **✅ Lint Compliance**: `cargo clippy --lib -- -D warnings` passes  
3. **✅ Trait Implementation**: All required traits implemented
4. **✅ Backward Compatibility**: Legacy functions still work
5. **✅ Type Safety**: Strong typing throughout, no stringly-typed APIs
6. **✅ Performance**: HFT-optimized with minimal allocations
7. **✅ Maintainability**: Each file has single responsibility

This proven template has successfully transformed **binance** and **backpack** exchanges, achieving full kernel compliance while maintaining production performance and reliability. 