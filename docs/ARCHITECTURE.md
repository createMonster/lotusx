# LotusX Architecture

**Updated:** 2026-06-15

## Summary

LotusX is organized as a Rust library crate with a small binary demo. The library exposes shared core types and traits, exchange connector modules, and latency-test utilities.

The main architectural idea is simple: the core owns generic transport, signing, configuration, errors, and shared financial types; each exchange owns only its API dialect, conversions, and connector behavior.

## Crate Shape

```text
src/
├── core/
│   ├── config.rs
│   ├── errors.rs
│   ├── kernel/
│   │   ├── codec.rs
│   │   ├── rest.rs
│   │   ├── signer.rs
│   │   └── ws.rs
│   ├── traits.rs
│   └── types.rs
├── exchanges/
├── utils/
├── lib.rs
└── main.rs
```

[src/lib.rs](/Users/larrycao/Desktop/projects/lotusx/src/lib.rs) exports `core`, `exchanges`, and `utils`. [src/main.rs](/Users/larrycao/Desktop/projects/lotusx/src/main.rs) is only a runnable market-data demo.

## Core Layer

[src/core/types.rs](/Users/larrycao/Desktop/projects/lotusx/src/core/types.rs) defines shared domain types such as `Symbol`, `Price`, `Quantity`, `Market`, `OrderRequest`, `Position`, and `FundingRate`. Monetary values use `rust_decimal::Decimal` through typed wrappers instead of floats.

[src/core/traits.rs](/Users/larrycao/Desktop/projects/lotusx/src/core/traits.rs) defines connector capabilities:

- `MarketDataSource`
- `OrderPlacer`
- `AccountInfo`
- `FundingRateSource`

[src/core/kernel](/Users/larrycao/Desktop/projects/lotusx/src/core/kernel/mod.rs) is the transport kernel:

- `RestClient` and `ReqwestRest` handle HTTP.
- `WsSession`, `TungsteniteWs`, and `ReconnectWs` handle WebSocket transport.
- `WsCodec` lets each exchange encode subscriptions and decode messages.
- `Signer` lets each exchange provide its own authentication scheme.

## Exchange Layer

Active exchange modules:

```text
backpack
binance
binance_perp
bybit
bybit_perp
hyperliquid
okx
paradex
```

Each active exchange uses this module template:

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

The connector composes the focused capability modules and delegates trait methods to them. For example, [src/exchanges/binance/connector/mod.rs](/Users/larrycao/Desktop/projects/lotusx/src/exchanges/binance/connector/mod.rs) wires `MarketData`, `Trading`, and `Account` into `BinanceConnector`.

## Utilities

[src/utils/exchange_factory.rs](/Users/larrycao/Desktop/projects/lotusx/src/utils/exchange_factory.rs) creates boxed market-data connectors for latency tests and demos.

[src/utils/latency_testing.rs](/Users/larrycao/Desktop/projects/lotusx/src/utils/latency_testing.rs) measures market-data, kline, WebSocket, and simulated tick-to-trade latency.

## Architecture Assessment

Strengths:

- Clear separation between transport kernel and exchange-specific API dialects.
- Consistent exchange module layout across all active connectors.
- Shared financial domain types reduce precision and parsing mistakes.
- Connector traits make market data, trading, account, and funding capabilities explicit.
- Tests currently compile and exercise public market-data paths.

Risks and debt:

- WebSocket support exists in the kernel and several modules, but default builders are not uniformly WebSocket-enabled. Call the explicit WebSocket builder when a module exposes one.
- `ExchangeConnector` is not the best documentation surface today; prefer the smaller capability traits when writing examples or new code.
- `src/lib.rs` re-exports connector types for all active exchanges. Use `lotusx::exchanges::<exchange>` for builders and exchange-specific API details.
- Some legacy compatibility constructors keep old names or parameters while delegating to the new builders. Prefer the direct `build_connector*` functions in new code.
- `ExchangeFactory` is for latency tests and demos. It now delegates Bybit and OKX creation through the exchange builders, but production code should still prefer exchange-specific builders when credentials, passphrases, or custom behavior matter.
- `RestClientConfig::max_retries` applies to retryable GET and DELETE failures. POST requests are not automatically retried to avoid repeating order-style side effects.
- `KlineInterval` is a shared semantic interval type; exchange-specific interval formatting belongs in exchange modules.
- `src/exchanges/okx/` is implemented, but `src/exchanges/okx_perp/` is an empty, unregistered directory. Treat OKX perpetual as not implemented.
- Some tests depend on public exchange APIs and can fail because of network or upstream API changes.
- Historical docs previously mixed old plans with current architecture. Current docs now keep those notes in `docs/archive/`.

## Running The Project

```bash
cargo check --all-targets --all-features
cargo test --all-features
cargo run
```

Current verification on 2026-06-15:

- `cargo check --all-targets --all-features`: passed
- `cargo test --all-features`: passed
- `cargo run`: fetched Binance perpetual markets successfully
