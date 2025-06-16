# Adding New Exchange Guide

## Overview

This guide provides a comprehensive walkthrough for adding a new cryptocurrency exchange to the LotusTX trading system. It's designed to be clear, systematic, and includes common pitfalls to avoid.

## 🎯 Key Principles

1. **Consistency**: Follow the established patterns used by existing exchanges
2. **Modularity**: Each exchange is self-contained with clear interfaces
3. **Error Handling**: Robust error handling for all API interactions
4. **Type Safety**: Strong typing for all data structures
5. **Testability**: Code should be easily testable

## 📁 Directory Structure

Each exchange follows this standard structure:
```
src/exchanges/exchange_name/
├── mod.rs           # Module exports and re-exports
├── client.rs        # Main connector struct
├── types.rs         # All data types and structures
├── auth.rs          # Authentication logic
├── converters.rs    # Type conversions and utilities
├── market_data.rs   # Market data implementation
├── trading.rs       # Order placement and management
└── account.rs       # Account information queries
```

## 🚀 Step-by-Step Implementation

### Step 1: Create the Exchange Directory

```bash
mkdir src/exchanges/exchange_name
```

### Step 2: Define Core Types (`types.rs`)

Start with the API response structures:

```rust
use serde::{Deserialize, Serialize};

// Main API response wrapper
#[derive(Debug, Deserialize, Serialize)]
pub struct ExchangeApiResponse<T> {
    pub success: bool,
    pub result: T,
    pub error: Option<String>,
}

// Market/Symbol information
#[derive(Debug, Deserialize, Serialize)]
pub struct ExchangeMarket {
    pub symbol: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub status: String,
    // Add exchange-specific fields
}

// Order request/response types
#[derive(Debug, Serialize)]
pub struct ExchangeOrderRequest {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: String,
    pub price: Option<String>,
    // Add exchange-specific fields
}

#[derive(Debug, Deserialize)]
pub struct ExchangeOrderResponse {
    pub order_id: String,
    pub client_order_id: String,
    pub symbol: String,
    pub side: String,
    pub status: String,
    pub timestamp: i64,
    // Add exchange-specific fields
}

// Account balance types
#[derive(Debug, Deserialize)]
pub struct ExchangeBalance {
    pub currency: String,
    pub available: String,
    pub locked: String,
}

// WebSocket message types
#[derive(Debug, Deserialize)]
pub struct ExchangeWebSocketMessage {
    pub channel: String,
    pub data: serde_json::Value,
    pub timestamp: i64,
}
```

### Step 3: Create the Client (`client.rs`)

```rust
use crate::core::{config::ExchangeConfig, traits::ExchangeConnector};
use reqwest::Client;

pub struct ExchangeNameConnector {
    pub(crate) client: Client,
    pub(crate) config: ExchangeConfig,
    pub(crate) base_url: String,
}

impl ExchangeNameConnector {
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://api-testnet.exchange.com".to_string()
        } else {
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.exchange.com".to_string())
        };

        Self {
            client: Client::new(),
            config,
            base_url,
        }
    }
}

impl ExchangeConnector for ExchangeNameConnector {}
```

### Step 4: Implement Authentication (`auth.rs`)

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn sign_request(
    payload: &str,
    secret_key: &str,
    // Add other parameters as needed
) -> Result<String, crate::core::errors::ExchangeError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).map_err(|_| {
        crate::core::errors::ExchangeError::AuthError("Invalid secret key".to_string())
    })?;
    
    mac.update(payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    
    Ok(signature)
}
```

### Step 5: Create Converters (`converters.rs`)

```rust
use super::types::{ExchangeMarket, ExchangeOrderResponse};
use crate::core::types::{Market, Symbol, OrderSide, OrderType, TimeInForce};

/// Convert exchange market to core market type
pub fn convert_exchange_market(exchange_market: ExchangeMarket) -> Market {
    Market {
        symbol: Symbol {
            base: exchange_market.base_currency,
            quote: exchange_market.quote_currency,
            symbol: exchange_market.symbol.clone(),
        },
        status: exchange_market.status,
        base_precision: 8, // Parse from exchange data
        quote_precision: 8, // Parse from exchange data
        min_qty: None, // Parse from exchange data
        max_qty: None, // Parse from exchange data
        min_price: None, // Parse from exchange data
        max_price: None, // Parse from exchange data
    }
}

/// Convert order side to exchange format
pub fn convert_order_side(side: &OrderSide) -> String {
    match side {
        OrderSide::Buy => "buy".to_string(),
        OrderSide::Sell => "sell".to_string(),
    }
}

/// Convert order type to exchange format
pub fn convert_order_type(order_type: &OrderType) -> String {
    match order_type {
        OrderType::Market => "market".to_string(),
        OrderType::Limit => "limit".to_string(),
        // Add other types as supported by the exchange
        _ => "limit".to_string(),
    }
}

/// Convert time in force to exchange format
pub fn convert_time_in_force(tif: &TimeInForce) -> String {
    match tif {
        TimeInForce::GTC => "GTC".to_string(),
        TimeInForce::IOC => "IOC".to_string(),
        TimeInForce::FOK => "FOK".to_string(),
    }
}
```

### Step 6: Implement Market Data (`market_data.rs`)

```rust
use super::client::ExchangeNameConnector;
use super::converters::convert_exchange_market;
use super::types as exchange_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::MarketDataSource;
use crate::core::types::{Kline, Market, MarketDataType, SubscriptionType, WebSocketConfig};
use crate::core::websocket::WebSocketManager;
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
impl MarketDataSource for ExchangeNameConnector {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/api/v1/markets", self.base_url);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Markets request failed: {}",
                error_text
            )));
        }

        let api_response: exchange_types::ExchangeApiResponse<Vec<exchange_types::ExchangeMarket>> = 
            response.json().await?;

        if !api_response.success {
            return Err(ExchangeError::NetworkError(format!(
                "Exchange API error: {:?}",
                api_response.error
            )));
        }

        let markets = api_response
            .result
            .into_iter()
            .map(convert_exchange_market)
            .collect();

        Ok(markets)
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Build WebSocket subscription streams
        let mut streams = Vec::new();

        for symbol in &symbols {
            for sub_type in &subscription_types {
                match sub_type {
                    SubscriptionType::Ticker => {
                        streams.push(format!("ticker:{}", symbol));
                    }
                    SubscriptionType::OrderBook { depth } => {
                        let depth_str = depth.map_or("20".to_string(), |d| d.to_string());
                        streams.push(format!("orderbook:{}:{}", symbol, depth_str));
                    }
                    SubscriptionType::Trades => {
                        streams.push(format!("trades:{}", symbol));
                    }
                    SubscriptionType::Klines { interval } => {
                        streams.push(format!("klines:{}:{}", symbol, interval));
                    }
                }
            }
        }

        let ws_url = self.get_websocket_url();
        let ws_manager = WebSocketManager::new(ws_url);
        ws_manager.start_stream(parse_websocket_message).await
    }

    fn get_websocket_url(&self) -> String {
        if self.config.testnet {
            "wss://ws-testnet.exchange.com/v1/stream".to_string()
        } else {
            "wss://ws.exchange.com/v1/stream".to_string()
        }
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: String,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        // Implementation for getting historical kline data
        todo!("Implement klines fetching")
    }
}

fn parse_websocket_message(_value: serde_json::Value) -> Option<MarketDataType> {
    // Parse WebSocket messages and convert to MarketDataType
    // This is exchange-specific and needs to be implemented
    None
}
```

### Step 7: Implement Trading (`trading.rs`)

```rust
use super::auth;
use super::client::ExchangeNameConnector;
use super::converters::{convert_order_side, convert_order_type, convert_time_in_force};
use super::types as exchange_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse, OrderType};
use async_trait::async_trait;

#[async_trait]
impl OrderPlacer for ExchangeNameConnector {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let url = format!("{}/api/v1/orders", self.base_url);
        let timestamp = auth::get_timestamp();

        // Build request body
        let request_body = exchange_types::ExchangeOrderRequest {
            symbol: order.symbol.clone(),
            side: convert_order_side(&order.side),
            order_type: convert_order_type(&order.order_type),
            quantity: order.quantity.clone(),
            price: if matches!(order.order_type, OrderType::Limit) {
                order.price.clone()
            } else {
                None
            },
        };

        let body = serde_json::to_string(&request_body).map_err(|e| {
            ExchangeError::NetworkError(format!("Failed to serialize request: {}", e))
        })?;

        // Generate signature
        let signature = auth::sign_request(&body, self.config.secret_key())?;

        let response = self
            .client
            .post(&url)
            .header("API-KEY", self.config.api_key())
            .header("TIMESTAMP", timestamp.to_string())
            .header("SIGNATURE", &signature)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Order placement failed: {}",
                error_text
            )));
        }

        let response_text = response.text().await?;
        let api_response: exchange_types::ExchangeApiResponse<exchange_types::ExchangeOrderResponse> = 
            serde_json::from_str(&response_text).map_err(|e| {
                ExchangeError::NetworkError(format!(
                    "Failed to parse response: {}. Response was: {}",
                    e, response_text
                ))
            })?;

        if !api_response.success {
            return Err(ExchangeError::NetworkError(format!(
                "Exchange API error: {:?}",
                api_response.error
            )));
        }

        let exchange_response = api_response.result;
        Ok(OrderResponse {
            order_id: exchange_response.order_id,
            client_order_id: exchange_response.client_order_id,
            symbol: exchange_response.symbol,
            side: order.side,
            order_type: order.order_type,
            quantity: order.quantity,
            price: order.price,
            status: exchange_response.status,
            timestamp: exchange_response.timestamp,
        })
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let url = format!("{}/api/v1/orders/{}", self.base_url, order_id);
        let timestamp = auth::get_timestamp();

        let request_body = serde_json::json!({
            "symbol": symbol
        });

        let body = request_body.to_string();
        let signature = auth::sign_request(&body, self.config.secret_key())?;

        let response = self
            .client
            .delete(&url)
            .header("API-KEY", self.config.api_key())
            .header("TIMESTAMP", timestamp.to_string())
            .header("SIGNATURE", &signature)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Order cancellation failed: {}",
                error_text
            )));
        }

        Ok(())
    }
}
```

### Step 8: Implement Account Info (`account.rs`)

```rust
use super::auth;
use super::client::ExchangeNameConnector;
use super::types as exchange_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for ExchangeNameConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/api/v1/account/balance", self.base_url);
        let timestamp = auth::get_timestamp();

        let signature = auth::sign_request("", self.config.secret_key())?;

        let response = self
            .client
            .get(&url)
            .header("API-KEY", self.config.api_key())
            .header("TIMESTAMP", timestamp.to_string())
            .header("SIGNATURE", &signature)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Balance request failed: {}",
                error_text
            )));
        }

        let api_response: exchange_types::ExchangeApiResponse<Vec<exchange_types::ExchangeBalance>> = 
            response.json().await?;

        if !api_response.success {
            return Err(ExchangeError::NetworkError(format!(
                "Exchange API error: {:?}",
                api_response.error
            )));
        }

        let balances = api_response
            .result
            .into_iter()
            .map(|balance| Balance {
                asset: balance.currency,
                free: balance.available,
                locked: balance.locked,
            })
            .collect();

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        // For spot exchanges, return empty positions
        // For futures exchanges, implement position fetching
        Ok(vec![])
    }
}
```

### Step 9: Create Module File (`mod.rs`)

```rust
pub mod account;
pub mod auth;
pub mod client;
pub mod converters;
pub mod market_data;
pub mod trading;
pub mod types;

// Re-export main types for easier importing
pub use client::ExchangeNameConnector;
pub use types::{
    ExchangeApiResponse, ExchangeBalance, ExchangeMarket, ExchangeOrderRequest,
    ExchangeOrderResponse, ExchangeWebSocketMessage,
};
```

### Step 10: Update Main Exchanges Module

Add your exchange to `src/exchanges/mod.rs`:

```rust
pub mod exchange_name;
```

## ⚠️ Common Mistakes to Avoid

### 1. Authentication Errors
- **Wrong signature format**: Each exchange has unique signature requirements
- **Missing headers**: Check API documentation for required headers
- **Timestamp issues**: Some exchanges require precise timestamp formats
- **URL encoding**: Some exchanges require URL-encoded parameters in signatures

### 2. Data Type Mismatches
- **String vs Number**: Many exchanges return numbers as strings in JSON
- **Precision handling**: Different exchanges have different precision requirements
- **Field naming**: API field names often don't match Rust conventions

### 3. Error Handling
- **Not parsing API errors**: Always check and parse exchange-specific error responses
- **Incomplete error context**: Include the full response in error messages for debugging
- **Missing status checks**: Always verify response status codes

### 4. WebSocket Implementation
- **Message format**: Each exchange has different WebSocket message formats
- **Subscription format**: Subscription parameters vary greatly between exchanges
- **Reconnection logic**: Implement proper reconnection handling

### 5. Rate Limiting
- **Missing rate limits**: Implement proper rate limiting to avoid bans
- **Burst handling**: Some exchanges have burst limits vs sustained limits
- **Different endpoints**: Different endpoints may have different rate limits

## 🔑 Key Points to Remember

1. **Read the API Documentation Thoroughly**
   - Understand authentication requirements
   - Check rate limits and restrictions
   - Verify WebSocket message formats
   - Test with sandbox/testnet first

2. **Handle Edge Cases**
   - Network timeouts and retries
   - Invalid responses from the exchange
   - Authentication failures
   - Market closure scenarios

3. **Type Safety First**
   - Use strong typing for all data structures
   - Implement proper error types
   - Use `Option<T>` for optional fields
   - Parse numbers carefully (string vs numeric)

4. **Testing Strategy**
   - Unit tests for converters
   - Integration tests with testnet
   - Mock tests for edge cases
   - Load testing for performance

5. **Documentation**
   - Document exchange-specific quirks
   - Include example usage
   - Document rate limits and restrictions
   - Keep examples up to date

## 🧪 Testing Your Implementation

Create a simple test in `examples/`:

```rust
// examples/exchange_name_example.rs
use lotusx::{
    core::{config::ExchangeConfig, traits::{AccountInfo, MarketDataSource}},
    exchanges::exchange_name::ExchangeNameConnector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExchangeConfig::from_env("EXCHANGE_NAME")?;
    let connector = ExchangeNameConnector::new(config);

    // Test market data
    let markets = connector.get_markets().await?;
    println!("Found {} markets", markets.len());

    // Test account balance (requires API credentials)
    match connector.get_account_balance().await {
        Ok(balances) => {
            println!("Balances:");
            for balance in balances {
                println!("  {}: {} free, {} locked", 
                    balance.asset, balance.free, balance.locked);
            }
        }
        Err(e) => println!("Error getting balance: {}", e),
    }

    Ok(())
}
```

## 📋 Checklist

Before submitting your exchange implementation:

- [ ] All traits implemented (`MarketDataSource`, `OrderPlacer`, `AccountInfo`)
- [ ] Proper error handling with specific error messages
- [ ] Authentication working correctly
- [ ] Type conversions implemented and tested
- [ ] WebSocket message parsing implemented
- [ ] Rate limiting considered
- [ ] Example/test file created
- [ ] Documentation updated
- [ ] Code passes `cargo fmt` and `cargo clippy`
- [ ] Integration tests pass with testnet

## 🚀 Advanced Considerations

### Performance Optimization
- Connection pooling for HTTP clients
- WebSocket connection management
- Efficient JSON parsing
- Memory usage optimization

### Security
- Secure credential handling
- API key rotation support
- Request signing verification
- Rate limiting implementation

### Reliability
- Automatic reconnection for WebSockets
- Retry logic for failed requests
- Circuit breaker pattern
- Health check endpoints

### Monitoring
- Metrics collection
- Logging for debugging
- Performance monitoring
- Error tracking

This guide provides a solid foundation for adding any new exchange to the LotusTX system. Remember to always test thoroughly and handle edge cases gracefully! 