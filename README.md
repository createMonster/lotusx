<div align="center">
  <img src="assets/images/lotusx_logo.png" alt="LotusX Logo" width="200" height="200">
  
  # LotusX - Crypto Exchange Connectors
  
  <p><em>A secure, async Rust library for cryptocurrency exchange APIs and real-time market data</em></p>
  
  [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/yourusername/lotusx)
  
</div>

---

## 🚀 **Quick Start**

### Add to your project

```toml
[dependencies]
lotusx = { path = ".", features = ["env-file"] }
tokio = { version = "1.0", features = ["full"] }
```

### Basic usage

```rust
use lotusx::{BinanceConnector, ExchangeConnector};
use lotusx::core::config::ExchangeConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load from .env file or environment variables
    let config = ExchangeConfig::from_env_file("BINANCE")?;
    let binance = BinanceConnector::new(config);

    // Get markets
    let markets = binance.get_markets().await?;
    println!("Found {} markets", markets.len());

    Ok(())
}
```

### Configuration

Create a `.env` file:

```bash
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET_KEY=your_secret_key_here
BINANCE_TESTNET=true
```

## ✨ **Features**

- **🔒 Secure**: Memory-protected credentials with automatic redaction
- **⚡ Async**: Built with tokio for high performance
- **🔗 WebSocket**: Real-time market data streaming with auto-reconnection
- **🛡️ Type Safe**: Strong typing for all API responses
- **🧪 Testnet**: Full testnet support for safe development
- **📊 Multi-Exchange**: Binance Spot & Futures (more coming soon)

## 📖 **Examples**

### Trading

```rust
use lotusx::core::types::*;

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

### Real-time Data Streaming

```rust
let symbols = vec!["BTCUSDT".to_string()];
let subscription_types = vec![
    SubscriptionType::Ticker,
    SubscriptionType::OrderBook { depth: Some(5) },
    SubscriptionType::Trades,
];

let mut receiver = binance
    .subscribe_market_data(symbols, subscription_types, None)
    .await?;

while let Some(data) = receiver.recv().await {
    match data {
        MarketDataType::Ticker(ticker) => {
            println!("{}@{} ({}%)", ticker.symbol, ticker.price, ticker.price_change_percent);
        }
        MarketDataType::Trade(trade) => {
            println!("Trade: {} @ {}", trade.quantity, trade.price);
        }
        _ => {}
    }
}
```

## 🏃 **Run Examples**

```bash
# Basic trading example
cargo run --example basic_usage

# WebSocket streaming
cargo run --example websocket_example

# Configuration examples
cargo run --example secure_config_example --features env-file
```

## 📚 **Documentation**

- **[Security Guide](docs/SECURITY_GUIDE.md)** - Credential handling best practices
- **[Technical Progress](docs/TECHNICAL_PROGRESS.md)** - Implementation status and roadmap
- **[Examples](examples/)** - Working code examples

## ⚠️ **Safety First**

- Always test with testnet first: `BINANCE_TESTNET=true`
- Start with small amounts
- Review all order parameters carefully
- Keep your API keys secure

## 🤝 **Contributing**

Contributions welcome! Please see our [technical progress](docs/TECHNICAL_PROGRESS.md) for current status and planned features.

## 📄 **License**

Open source project. Please review the code before using in production. 