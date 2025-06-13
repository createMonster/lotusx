<div align="center">
  <img src="assets/images/lotusx_logo.png" alt="LotusX Logo" width="200" height="200">
  
  # LotusX - Crypto Exchange Connectors
  
  <p><em>A secure, async Rust library for cryptocurrency exchange APIs and real-time market data</em></p>
  
  [![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/yourusername/lotusx)
  
</div>

---

## 📊 **Supported Exchanges**

| Exchange | Market Data | WebSocket | Trading | Account | Klines | Testnet | Status |
|----------|-------------|-----------|---------|---------|--------|---------|--------|
| **Binance Spot** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Complete** |
| **Binance Perpetual** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **Complete** |
| **Hyperliquid** | ✅ | ✅ | ✅ | ✅ | ❌* | ✅ | **Complete** |
| **Backpack** | ✅ | ✅ | ✅ | ❌ | ✅ | ❌ | **In Progress** |

**Legend:**
- ✅ Fully implemented and tested
- ❌ Not available (exchange limitation)
- ❌* Feature not supported by exchange
- 🚧 Work in progress

### Performance Benchmarks
| Exchange | Markets Load | Klines Avg | WebSocket Connect |
|----------|--------------|------------|-------------------|
| Binance Spot | ~4.0s (1,445 markets) | ~214ms | <100ms |
| Binance Perpetual | ~1.4s (509 markets) | ~234ms | <100ms |
| Hyperliquid | ~399ms (199 markets) | N/A | <100ms |
| Backpack | TBD | TBD | <100ms |

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

- **🏗️ Modular Architecture**: Clean, consistent structure across all exchanges
- **🔒 Secure**: Memory-protected credentials with automatic redaction
- **⚡ Async**: Built with tokio for high performance
- **🔗 WebSocket**: Real-time market data streaming with auto-reconnection
- **🛡️ Type Safe**: Strong typing for all API responses
- **🧪 Testnet**: Full testnet support for safe development
- **📊 Multi-Exchange**: Binance Spot & Futures, Hyperliquid, Backpack (more coming soon)
- **🔧 Maintainable**: Single-responsibility modules for easy development
- **📈 Performance Monitoring**: Built-in latency testing and benchmarking tools
- **🌐 Cross-Platform**: Reliable TLS implementation using rustls for consistent connections
- **🎯 Quality Assured**: All code passes strict clippy linting with zero warnings

## 🏛️ **Architecture Highlights**

### Consistent Modular Design
All exchanges follow the same proven structure:

```
exchanges/{exchange}/
├── client.rs       # Core client (~30 lines, was 500+)
├── account.rs      # Account functions (AccountInfo trait)
├── trading.rs      # Trading functions (OrderPlacer trait)  
├── market_data.rs  # Market data (MarketDataSource trait)
├── converters.rs   # Data conversion & parsing
├── auth.rs         # Authentication & signing
├── types.rs        # Exchange-specific types
└── mod.rs          # Module exports
```

### Benefits
- **🎯 95% Code Reduction**: Massive client files reduced to focused modules
- **🔄 Code Reuse**: Shared components between related exchanges
- **🧩 Single Responsibility**: Each module has one clear purpose
- **📈 Maintainability**: Easy to locate and modify specific functionality
- **🎨 Consistency**: Identical patterns across all implementations

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

### Performance Benchmarking

```rust
// Test latency across all exchanges
cargo run --example latency_test

// Results show:
// - get_markets: 200ms-4s (varies by exchange)
// - get_klines: 150-250ms (where supported)
// - WebSocket connection: <100ms
// - Comprehensive statistics (min/max/avg/median/std dev)
```

## 🏃 **Run Examples**

```bash
# Basic trading example
cargo run --example basic_usage

# WebSocket streaming (now with reliable TLS!)
cargo run --example backpack_streams_example

# Performance benchmarking
cargo run --example latency_test

# Configuration examples
cargo run --example secure_config_example --features env-file

# Hyperliquid integration
cargo run --example hyperliquid_example
```

## 📚 **Documentation**

- **[Security Guide](docs/SECURITY_GUIDE.md)** - Credential handling best practices
- **[Technical Progress](docs/TECHNICAL_PROGRESS.md)** - Implementation status and roadmap
- **[Changelog](docs/changelog.md)** - Recent updates and improvements
- **[Examples](examples/)** - Working code examples

## ⚠️ **Safety First**

- Always test with testnet first: `BINANCE_TESTNET=true`
- Start with small amounts
- Review all order parameters carefully
- Keep your API keys secure

## 🏗️ **For Developers**

### Adding New Exchanges
Thanks to our standardized modular architecture, adding new exchanges is straightforward:

1. Copy the module structure from an existing exchange
2. Implement the three core traits: `MarketDataSource`, `OrderPlacer`, `AccountInfo`
3. Create exchange-specific types and authentication
4. Follow the established patterns for consistency

### Module Responsibilities
- **client.rs**: Basic setup and core HTTP helpers
- **auth.rs**: Exchange-specific authentication logic
- **types.rs**: All exchange-specific data structures
- **converters.rs**: Convert between exchange and core types
- **market_data.rs**: Implement MarketDataSource trait
- **trading.rs**: Implement OrderPlacer trait
- **account.rs**: Implement AccountInfo trait

### Code Quality Standards
- All code must pass `cargo clippy --all-targets --all-features -- -D warnings`
- Use rustls for WebSocket TLS connections (cross-platform reliability)
- Follow established error handling and type conversion patterns
- Maintain consistent module structure across exchanges

## 🤝 **Contributing**

Contributions welcome! Please see our [technical progress](docs/TECHNICAL_PROGRESS.md) for current status and planned features.

The modular architecture makes it easy to:
- Add new exchange integrations
- Improve existing functionality
- Add new features to specific exchanges
- Maintain code quality and consistency

### Quality Assurance
- All PRs must pass clippy with zero warnings
- WebSocket connections use reliable rustls TLS implementation
- Comprehensive testing across supported platforms
- Performance benchmarking for new exchange integrations

## 📄 **License**

Open source project. Please review the code before using in production. 