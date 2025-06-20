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

| Exchange | Market Data | WebSocket | Trading | Account | Status |
|----------|-------------|-----------|---------|---------|--------|
| **Binance Spot** | ✅ | ✅ | ✅ | ✅ | Complete |
| **Binance Perpetual** | ✅ | ✅ | ✅ | ✅ | Complete |
| **Bybit Spot** | ✅ | ✅ | ✅ | ✅ | Complete |
| **Bybit Perpetual** | ✅ | ✅ | ✅ | ✅ | Complete |
| **Hyperliquid** | ✅ | ✅ | ✅ | ✅ | Complete |
| **Backpack** | ✅ | ✅ | ✅ | ❌ | In Progress |

## 🚀 **Quick Start**

### Installation

```toml
[dependencies]
lotusx = { path = ".", features = ["env-file"] }
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use lotusx::{BinanceConnector, BybitConnector};
use lotusx::core::config::ExchangeConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment
    let config = ExchangeConfig::from_env("BINANCE")?;
    let binance = BinanceConnector::new(config);

    // Get markets
    let markets = binance.get_markets().await?;
    println!("Found {} markets", markets.len());

    Ok(())
}
```

### Environment Configuration

Create a `.env` file:

```bash
# Binance
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET_KEY=your_secret_key_here
BINANCE_TESTNET=true

# Bybit  
BYBIT_API_KEY=your_bybit_api_key_here
BYBIT_SECRET_KEY=your_bybit_secret_key_here
BYBIT_TESTNET=true
```

## ✨ **Key Features**

- **🏗️ Multi-Exchange**: Unified API across 6 major exchanges
- **⚡ Async**: Built with tokio for high performance
- **🔒 Secure**: Memory-protected credentials with automatic redaction
- **🔗 WebSocket**: Real-time market data streaming with auto-reconnection
- **🧪 Testnet**: Full testnet support for safe development
- **📊 Performance Testing**: Built-in latency analysis and HFT metrics
- **🛡️ Type Safe**: Strong typing for all API responses

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

### WebSocket Streaming

```rust
let symbols = vec!["BTCUSDT".to_string()];
let subscription_types = vec![SubscriptionType::Ticker];

let mut receiver = binance
    .subscribe_market_data(symbols, subscription_types, None)
    .await?;

while let Some(data) = receiver.recv().await {
    match data {
        MarketDataType::Ticker(ticker) => {
            println!("{}@{} ({}%)", ticker.symbol, ticker.price, ticker.price_change_percent);
        }
        _ => {}
    }
}
```

### Latency Testing

Run comprehensive latency analysis across all exchanges:

```bash
# Quick test (reduced sample size)
cargo run --example latency_test -- --quick

# Full analysis with all exchanges
cargo run --example latency_test -- --all

# Comprehensive testing (larger sample sizes)
cargo run --example latency_test -- --comprehensive
```

**Example Output:**
```
🚀 HFT Exchange Latency Analysis
================================
📊 Testing 5 exchanges...

📊 CRITICAL PERFORMANCE METRICS
--------------------------------------------------------------------------------
Exchange        P99 (μs)   P95 (μs)   Mean (μs)  Jitter (μs) Reliability (%)
--------------------------------------------------------------------------------
Binance Spot    620116     606453     551639     34336      93.8           
Binance Perp    67895      46585      43988      6055       86.2           
Bybit Spot      541927     502592     226296     165441     26.9           
Hyperliquid     313108     39672      27732      67718      100.0          

⚡ HFT-SPECIFIC METRICS
--------------------------------------------------------------------------------
Exchange        Tick-to-Trade (μs) Market Impact (bps) Liquidity Score 
--------------------------------------------------------------------------------
Binance Perp    51030           1.38            13.6            
Hyperliquid     11480           5.00            1.4             
```

### Custom Latency Testing

```rust
use lotusx::utils::exchange_factory::*;
use lotusx::utils::latency_testing::*;

// Build custom test configuration
let configs = ExchangeTestConfigBuilder::new()
    .add_exchange(ExchangeType::Binance, false)
    .add_exchange(ExchangeType::Hyperliquid, false)
    .build();

// Run tests with custom configuration
let tester = LatencyTester::with_config(LatencyTestConfig::comprehensive());
// ... run tests
```

## 🏃 **Run Examples**

```bash
# Basic usage
cargo run --example basic_usage

# Exchange-specific examples
cargo run --example bybit_example
cargo run --example hyperliquid_example
cargo run --example backpack_streams_example

# Performance testing
cargo run --example latency_test
cargo run --example custom_latency_test

# WebSocket streaming
cargo run --example websocket_example
```

## ⚠️ **Safety First**

- Always test with testnet first: `BINANCE_TESTNET=true`
- Start with small amounts in production
- Keep your API keys secure
- Review all order parameters carefully

## 🤝 **Contributing**

Contributions welcome! The modular architecture makes it easy to add new exchanges or improve existing functionality.

### Quality Standards
- All code must pass `cargo clippy` with zero warnings
- Comprehensive testing across supported platforms
- Follow established patterns for consistency

## 📄 **License**

MIT License. Please review the code before using in production. 