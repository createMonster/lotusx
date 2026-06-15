# Adding A New Exchange

Use this guide when adding a new exchange connector to LotusX. Keep the first version small: market data first, then account, then trading, then optional WebSocket and funding-rate support.

## Current Pattern

Every active exchange module follows this structure:

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

Responsibilities:

- `types.rs`: exchange API request/response structs.
- `conversions.rs`: exchange types to core types.
- `rest.rs`: typed wrapper around `core::kernel::RestClient`.
- `signer.rs`: exchange authentication through `core::kernel::Signer`.
- `codec.rs`: WebSocket subscription encoding and message decoding through `WsCodec`.
- `connector/market_data.rs`: `MarketDataSource`.
- `connector/trading.rs`: `OrderPlacer`.
- `connector/account.rs`: `AccountInfo`.
- `connector/mod.rs`: final connector composition and trait delegation.
- `builder.rs`: public constructor functions and builder type.
- `mod.rs`: module exports.

## Minimal Steps

1. Create `src/exchanges/<exchange>/` and `src/exchanges/<exchange>/connector/`.
2. Add `types.rs` for raw API structs.
3. Add `conversions.rs` for mapping into `Symbol`, `Price`, `Quantity`, `Market`, `OrderRequest`, and other core types.
4. Add `rest.rs` with only the endpoints needed for the first working slice.
5. Add `signer.rs` if authenticated endpoints are needed.
6. Add `codec.rs` if WebSocket support is needed.
7. Implement `connector/market_data.rs` first.
8. Add `connector/trading.rs` and `connector/account.rs` only when needed.
9. Compose the connector in `connector/mod.rs`.
10. Add public constructors in `builder.rs`.
11. Export the module in `src/exchanges/mod.rs`.
12. Optionally add the exchange to `src/utils/exchange_factory.rs` for latency tests.

## Trait Targets

Use the smallest trait that matches the capability:

- Market data: `MarketDataSource`
- Orders: `OrderPlacer`
- Balances and positions: `AccountInfo`
- Perpetual funding rates: `FundingRateSource`

Do not force a spot exchange to implement funding rates.

## Verification

Run these before calling the exchange usable:

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo test --all-features
```

For public market data, add a small integration test that can tolerate network failures if the endpoint is not reliable in CI. For authenticated trading, keep tests ignored by default and require explicit credentials.
