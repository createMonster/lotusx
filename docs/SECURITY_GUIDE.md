# Security Guide

LotusX handles exchange credentials through `ExchangeConfig`. Keep credentials at the process boundary and prefer read-only configuration for public market data.

## Configuration Methods

Read-only market data:

```rust
use lotusx::core::config::ExchangeConfig;

let config = ExchangeConfig::read_only().testnet(true);
```

Environment variables:

```rust
use lotusx::core::config::ExchangeConfig;

let config = ExchangeConfig::from_env("BINANCE")?;
```

`.env` file:

```rust
use lotusx::core::config::ExchangeConfig;

let config = ExchangeConfig::from_env_file("BINANCE")?;
```

Direct configuration:

```rust
use lotusx::core::config::ExchangeConfig;

let config = ExchangeConfig::new(api_key, secret_key).testnet(true);
```

## Environment Variables

`ExchangeConfig::from_env("BINANCE")` reads:

```bash
BINANCE_API_KEY=...
BINANCE_SECRET_KEY=...
BINANCE_TESTNET=true
BINANCE_BASE_URL=...
```

The same pattern applies to `BYBIT`, `BINANCE_PERP`, `BACKPACK`, `HYPERLIQUID`, `OKX`, and `PARADEX`.

OKX authenticated builders also need `OKX_PASSPHRASE`.

## Built-In Protections

- `ExchangeConfig` stores credentials with `secrecy::Secret<String>`.
- Serialization redacts `api_key` and `secret_key`.
- `.env` is ignored by git.
- `ExchangeConfig::read_only()` avoids dummy secrets for public endpoints.

## Local Practices

- Never commit `.env`.
- Keep `.env.template` empty of real secrets.
- Use `chmod 600 .env` on shared machines.
- Use testnet first.
- Grant API keys the smallest permissions needed.
- Do not enable withdrawals on keys used for local development.

## Trading Safety

Before uncommenting order-placement examples, verify:

- exchange and account
- testnet or production mode
- symbol
- side
- quantity
- price
- order type

Credential safety is only half the problem; intentional order parameters are the other half.
