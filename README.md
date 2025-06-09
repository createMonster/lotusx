<div align="center">
  <img src="assets/images/lotusx_logo.png" alt="LotusX Logo" width="200" height="200">
  
  # LotusX - Crypto Exchange Connectors
  
  <p><em>A Rust library for connecting to cryptocurrency exchanges for API trading</em></p>
  
  [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/yourusername/lotusx)
  
</div>

---

A Rust library for connecting to cryptocurrency exchanges for API trading. Currently supports Binance with a minimal but extensible architecture.

## Features

- ✅ **Binance Integration**: Get markets and place orders
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

### Basic Usage

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

## Running Examples

```bash
# Run the basic example
cargo run --example basic_usage

# Run the main example
cargo run
```

## Architecture

The library is designed with extensibility in mind:

- **Core Traits**: `ExchangeConnector` trait for unified interface
- **Type Safety**: Strong typing for all data structures
- **Error Handling**: Comprehensive error types with proper error propagation
- **Async First**: All operations are async for better performance

## Safety Notes

⚠️ **Important**: 
- Always test with testnet first
- Double-check all order parameters
- Start with small amounts
- The library handles API authentication and signatures automatically

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is open source. Please review the code before using in production. 