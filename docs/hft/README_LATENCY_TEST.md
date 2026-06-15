# Latency Testing

LotusX includes latency-test examples for comparing exchange market-data behavior.

## Commands

Quick run:

```bash
cargo run --example latency_test -- --quick
```

Default run:

```bash
cargo run --example latency_test
```

Larger sample run:

```bash
cargo run --example latency_test -- --comprehensive
```

Include credential-gated exchanges when credentials are present:

```bash
cargo run --example latency_test -- --all
```

Custom exchange selection example:

```bash
cargo run --example custom_latency_test
```

## Default Exchanges

The default latency config currently includes:

- Binance Spot
- Binance Perp
- Bybit Spot
- Bybit Perp
- Hyperliquid
- Paradex

Backpack is only added when `BACKPACK_API_KEY` and `BACKPACK_SECRET_KEY` are available.

## What Is Measured

The latency tester can measure:

- market-list request latency
- kline request latency
- WebSocket connection and first-message timing
- simulated tick-to-trade timing
- simple reliability, jitter, market-impact, and liquidity-score summaries

These measurements are useful for local comparisons, not exchange guarantees. Public internet routes, API load, rate limits, and local network conditions all affect results.
