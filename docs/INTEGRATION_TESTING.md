# Integration Testing

LotusX has unit tests, public API integration tests, ignored credential tests, and doctests.

## Main Commands

```bash
cargo test --all-features
cargo test --test simple_integration_tests -- --nocapture
cargo test --test binance_integration_tests -- --nocapture
cargo test --test bybit_integration_tests -- --nocapture
```

`cargo test --all-features` is the default confidence check. It currently runs public market-data paths and skips credential-only tests.

## Credential Tests

Some tests are marked `#[ignore = "Requires valid API credentials"]`. Run them only when credentials and testnet settings are intentional:

```bash
cargo test --test binance_integration_tests -- --ignored --nocapture
cargo test --test bybit_integration_tests -- --ignored --nocapture
```

Common environment variables:

```bash
BINANCE_API_KEY=...
BINANCE_SECRET_KEY=...
BINANCE_TESTNET=true

BINANCE_PERP_API_KEY=...
BINANCE_PERP_SECRET_KEY=...
BINANCE_PERP_TESTNET=true

BYBIT_API_KEY=...
BYBIT_SECRET_KEY=...
BYBIT_TESTNET=true
```

`ExchangeConfig::from_env_file("BINANCE")` and similar helpers also read `.env`.

## Test Files

- `tests/simple_integration_tests.rs`: safe public market-data smoke tests.
- `tests/binance_integration_tests.rs`: Binance spot and perpetual tests.
- `tests/bybit_integration_tests.rs`: Bybit spot and perpetual tests.
- `tests/integration_test_config.rs`: shared test helpers.

## Safety

- Use testnet credentials for live tests.
- Keep order-placement tests ignored unless the account, symbol, size, and price are intentional.
- Network-backed tests can fail because of API availability, rate limits, or local connectivity.
