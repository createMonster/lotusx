# LotusX

LotusX is an async Rust library for cryptocurrency exchange connectors. It provides a shared core for configuration, typed financial data, REST/WebSocket transport, signing, and exchange-specific connector modules.

The current repository is a library crate with a small binary demo in [src/main.rs](/Users/larrycao/Desktop/projects/lotusx/src/main.rs). The demo fetches Binance perpetual markets.

## Supported Modules

| Module | Market data | Trading | Account | Funding rates |
| --- | --- | --- | --- | --- |
| `binance` | yes | yes | yes | no |
| `binance_perp` | yes | yes | yes | yes |
| `bybit` | yes | yes | yes | no |
| `bybit_perp` | yes | yes | yes | yes |
| `backpack` | yes | yes | yes | no |
| `hyperliquid` | yes | yes | yes | no |
| `okx` | yes | yes | yes | no |
| `paradex` | yes | yes | yes | yes |

Each active exchange module follows the same shape:

```text
src/exchanges/<exchange>/
├── builder.rs
├── codec.rs
├── connector/
│   ├── account.rs
│   ├── market_data.rs
│   ├── mod.rs
│   └── trading.rs
├── conversions.rs
├── mod.rs
├── rest.rs
├── signer.rs
└── types.rs
```

## Quick Start

```bash
cargo check --all-targets --all-features
cargo test --all-features
cargo run
```

`cargo run` uses public Binance perpetual market data and requires network access.

Use the crate from another project:

```toml
[dependencies]
lotusx = { path = ".", features = ["env-file"] }
tokio = { version = "1.0", features = ["full"] }
```

Basic market-data usage:

```rust
use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::MarketDataSource;
use lotusx::exchanges::binance::build_connector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExchangeConfig::read_only().testnet(true);
    let binance = build_connector(config)?;

    let markets = binance.get_markets().await?;
    println!("Found {} markets", markets.len());

    Ok(())
}
```

Typed order request shape:

```rust
use lotusx::core::types::{
    conversion, OrderRequest, OrderSide, OrderType, TimeInForce,
};

let order = OrderRequest {
    symbol: conversion::string_to_symbol("BTCUSDT"),
    side: OrderSide::Buy,
    order_type: OrderType::Limit,
    quantity: conversion::string_to_quantity("0.001"),
    price: Some(conversion::string_to_price("30000")),
    time_in_force: Some(TimeInForce::GTC),
    stop_price: None,
};
```

## Environment

Configuration is read with `{EXCHANGE}_API_KEY`, `{EXCHANGE}_SECRET_KEY`, optional `{EXCHANGE}_TESTNET`, and optional `{EXCHANGE}_BASE_URL`.

```bash
BINANCE_API_KEY=...
BINANCE_SECRET_KEY=...
BINANCE_TESTNET=true

BYBIT_API_KEY=...
BYBIT_SECRET_KEY=...
BYBIT_TESTNET=true

BACKPACK_API_KEY=...
BACKPACK_SECRET_KEY=...

HYPERLIQUID_API_KEY=...
HYPERLIQUID_SECRET_KEY=...
HYPERLIQUID_TESTNET=true

OKX_API_KEY=...
OKX_SECRET_KEY=...
OKX_PASSPHRASE=...
OKX_TESTNET=true

PARADEX_API_KEY=...
PARADEX_SECRET_KEY=...
PARADEX_TESTNET=true
```

Keep real credentials in `.env`; the file is ignored by git.

## Examples

```bash
cargo run --example basic_usage
cargo run --example bybit_example
cargo run --example hyperliquid_example
cargo run --example okx_example
cargo run --example backpack_example
cargo run --example paradex_example
cargo run --example connection_test
cargo run --example latency_test -- --quick
cargo run --example custom_latency_test
```

## Documentation

- [Architecture](/Users/larrycao/Desktop/projects/lotusx/docs/ARCHITECTURE.md)
- [Adding a New Exchange](/Users/larrycao/Desktop/projects/lotusx/docs/ADDING_NEW_EXCHANGE.md)
- [Integration Testing](/Users/larrycao/Desktop/projects/lotusx/docs/INTEGRATION_TESTING.md)
- [Security Guide](/Users/larrycao/Desktop/projects/lotusx/docs/SECURITY_GUIDE.md)
- [Latency Testing](/Users/larrycao/Desktop/projects/lotusx/docs/hft/README_LATENCY_TEST.md)
- [Changelog](/Users/larrycao/Desktop/projects/lotusx/docs/changelog.md)

Historical plans, migrations, reports, and older analysis notes are under [docs/archive](/Users/larrycao/Desktop/projects/lotusx/docs/archive).

## Safety

Use testnet first, keep API keys out of commits, and leave order-placement examples commented until the target account, symbol, size, and price are intentional.
