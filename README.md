<div align="center">
  <img src="assets/images/lotusx_logo.png" alt="LotusX Logo" width="200" height="200">
  
  # LotusX - Crypto Exchange Connectors
  
  <p><em>A Rust library for connecting to cryptocurrency exchanges for API trading and real-time market data</em></p>
  
  [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/yourusername/lotusx)
  
</div>

---

A Rust library for connecting to cryptocurrency exchanges for API trading and real-time market data streaming. Currently supports Binance Spot and Binance Perpetual Futures with a minimal but extensible architecture.

## Features

- ✅ **Binance Spot Integration**: Get markets, place orders, and stream market data
- ✅ **Binance Perpetual Futures**: Full support for futures trading and data streaming
- ✅ **Real-time WebSocket Streaming**: Live market data with auto-reconnection
- ✅ **Async/Await Support**: Built with tokio for high performance
- ✅ **Type Safety**: Strong typing for all API responses
- ✅ **Error Handling**: Comprehensive error types
- ✅ **Testnet Support**: Safe testing environment
- 🔄 **Extensible**: Easy to add more exchanges

## Quick Start

### Add to Cargo.toml

```toml
[dependencies]
lotusx = { path = "." }
tokio = { version = "1.0", features = ["full"] }
```

### Basic Trading Usage

```rust
use lotusx::{BinanceConnector, ExchangeConnector, OrderRequest, OrderSide, OrderType};
use lotusx::core::config::ExchangeConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure with your API credentials
    let config = ExchangeConfig::new(
        "your_api_key".to_string(),
        "your_secret_key".to_string(),
    ).testnet(true); // Use testnet for safety

    let binance = BinanceConnector::new(config);

    // Get all available markets
    let markets = binance.get_markets().await?;
    println!("Found {} markets", markets.len());

    // Place a limit order
    let order = OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: "0.001".to_string(),
        price: Some("30000.0".to_string()),
        time_in_force: Some(TimeInForce::GTC),
        stop_price: None,
    };

    let response = binance.place_order(order).await?;
    println!("Order placed: {}", response.order_id);

    Ok(())
}
```

### WebSocket Market Data Streaming

```rust
use lotusx::core::{
    config::ExchangeConfig,
    traits::ExchangeConnector,
    types::*,
};
use lotusx::exchanges::{
    binance::BinanceConnector,
    binance_perp::BinancePerpConnector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExchangeConfig {
        api_key: "your_api_key".to_string(),
        secret_key: "your_secret_key".to_string(),
        base_url: None,
        testnet: true,
    };

    // Create connectors
    let binance_spot = BinanceConnector::new(config.clone());
    let binance_perp = BinancePerpConnector::new(config);

    // Configure what data to stream
    let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::OrderBook { depth: Some(5) },
        SubscriptionType::Trades,
        SubscriptionType::Klines { interval: "1m".to_string() },
    ];

    // WebSocket configuration
    let ws_config = WebSocketConfig {
        auto_reconnect: true,
        ping_interval: Some(30),
        max_reconnect_attempts: Some(5),
    };

    // Start streaming market data
    let mut receiver = binance_spot
        .subscribe_market_data(symbols, subscription_types, Some(ws_config))
        .await?;

    // Process incoming data
    while let Some(data) = receiver.recv().await {
        match data {
            MarketDataType::Ticker(ticker) => {
                println!("Ticker: {} - Price: {} ({}%)", 
                    ticker.symbol, ticker.price, ticker.price_change_percent);
            }
            MarketDataType::OrderBook(orderbook) => {
                println!("OrderBook: {} - Best Bid: {}, Best Ask: {}", 
                    orderbook.symbol,
                    orderbook.bids.first().map(|b| &b.price).unwrap_or(&"N/A".to_string()),
                    orderbook.asks.first().map(|a| &a.price).unwrap_or(&"N/A".to_string())
                );
            }
            MarketDataType::Trade(trade) => {
                println!("Trade: {} - Price: {}, Qty: {}", 
                    trade.symbol, trade.price, trade.quantity);
            }
            MarketDataType::Kline(kline) => {
                println!("Kline: {} - OHLC: {}/{}/{}/{}", 
                    kline.symbol, kline.open_price, kline.high_price, 
                    kline.low_price, kline.close_price);
            }
        }
    }

    Ok(())
}
```

## Configuration

### Environment Variables

You can set your API credentials using environment variables:

```bash
export BINANCE_API_KEY="your_api_key"
export BINANCE_SECRET_KEY="your_secret_key"
```

### Testnet

Always use testnet when testing:

```rust
let config = ExchangeConfig::new(api_key, secret_key).testnet(true);
```

## Supported Operations

### Get Markets

```rust
let markets = binance.get_markets().await?;
for market in markets {
    println!("Symbol: {}, Status: {}", market.symbol.symbol, market.status);
}
```

### Place Orders

```rust
let order = OrderRequest {
    symbol: "BTCUSDT".to_string(),
    side: OrderSide::Buy,
    order_type: OrderType::Limit,
    quantity: "0.001".to_string(),
    price: Some("30000.0".to_string()),
    time_in_force: Some(TimeInForce::GTC),
    stop_price: None,
};

let response = binance.place_order(order).await?;
```

### WebSocket Market Data

The library supports real-time market data streaming with the following features:

#### Subscription Types

- **Ticker**: 24hr ticker statistics
- **OrderBook**: Real-time order book updates with configurable depth
- **Trades**: Individual trade executions
- **Klines**: Candlestick/OHLC data with configurable intervals

#### WebSocket Configuration

```rust
let ws_config = WebSocketConfig {
    auto_reconnect: true,           // Automatically reconnect on disconnection
    ping_interval: Some(30),        // Send ping every 30 seconds
    max_reconnect_attempts: Some(5), // Maximum reconnection attempts
};
```

#### Supported Exchanges

- **Binance Spot**: `BinanceConnector`
- **Binance Perpetual Futures**: `BinancePerpConnector`

## Running Examples

```bash
# Run the basic trading example
cargo run --example basic_usage

# Run the WebSocket streaming example
cargo run --example websocket_example

# Run the main example
cargo run
```

## Architecture

The library is designed with extensibility in mind:

- **Core Traits**: `ExchangeConnector` trait for unified interface
- **WebSocket Manager**: Reusable WebSocket handling with auto-reconnection
- **Type Safety**: Strong typing for all data structures
- **Error Handling**: Comprehensive error types with proper error propagation
- **Async First**: All operations are async for better performance

### WebSocket Features

- **Auto-reconnection**: Automatically reconnects on connection loss
- **Ping/Pong Handling**: Built-in heartbeat mechanism
- **Message Parsing**: Type-safe parsing of exchange-specific messages
- **Error Recovery**: Robust error handling and recovery mechanisms
- **Configurable**: Flexible configuration for different use cases

## Safety Notes

⚠️ **Important**: 
- Always test with testnet first
- Double-check all order parameters
- Start with small amounts
- The library handles API authentication and signatures automatically
- WebSocket connections are automatically managed but monitor for any connection issues

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is open source. Please review the code before using in production. 