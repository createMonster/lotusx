**LotusX Connector Layer — Cross-Exchange Arbitrage Readiness
(English Summary in Markdown)**

---

## 1. Scope

This document focuses **exclusively on the *connector layer*** of LotusX and outlines the technical work required to make it “arbitrage-ready”.

---

## 2. Capabilities Every Connector Must Expose

| Module               | Goal                                                                                                 | Typical API                                                                       |
| :------------------- | :--------------------------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------- |
| **Market Data**      | L2/L3 order-book, recent trades, candles, index price, funding rate, estimated liquidation price     | `get_orderbook(level)`, `get_trades`, `get_funding_rate`, `get_liquidation_price` |
| **Trading**          | Beyond plain *place/cancel*: batch orders, amend, batch cancel, order-type flags (IOC/FOK/Post-Only) | `place_batch_orders`, `amend_order`, `cancel_batch`                               |
| **Account / Assets** | Balances, positions, sub-accounts, leverage brackets, dynamic fee table                              | `get_balances`, `get_positions`, `get_fee_rates`, `get_leverage_bracket`          |
| **Funds Transfer**   | Internal transfers / on-chain withdrawals (stub if not yet implemented)                              | `internal_transfer`, `withdraw`, `deposit_history`                                |
| **System / Meta**    | Time sync, exchange status, rate limits                                                              | `sync_time`, `get_system_status`, `get_rate_limits`                               |
| **Observability**    | Tracing span + metrics for every REST/WS call                                                        | `instrumented_client.request(...)`                                                |

---

## 3. Code-Level Improvements Still Missing

| Topic                 | Current State                  | Action Items                                                                             |
| :-------------------- | :----------------------------- | :--------------------------------------------------------------------------------------- |
| **Unified Types**     | ✅ **COMPLETED** `price/qty` now use `rust_decimal::Decimal` with type-safe `Symbol` | ✅ All core types updated, Binance implemented, comprehensive tests added. See `UNIFIED_TYPES_IMPLEMENTATION.md` |
| **REST / WS Kernel**  | Each connector rolls its own   | Extract `RestClient` / `WsSession` traits handling signing, retries, rate limiting       |
| **Feature Gating**    | Single crate builds everything | Convert to Cargo **workspace** (`lotusx-core` + `connector-*`), enable with `--features` |
| **Error Granularity** | Generic `Other(String)`        | Use `thiserror` + fine-grained mapping of exchange error codes                           |
| **Testing**           | Mostly unit tests              | CI on testnet (live order → query → cancel) + `vcr-rs` playback                          |
| **Observability**     | Latency CLI only               | Integrate `tracing` + Prometheus (p95 RTT, WS drops, rate-limit hits)                    |

---

## 4. Next Exchanges to Add (Priority Order)

1. **OKX** (spot + perp, excellent sub-account support)
2. **Coinbase Advanced Trade API**
3. **Kraken** (spot/futures)
4. **Bitget** & **KuCoin**
5. **Gate.io** / **BingX** (Asia-focused backup)

---

## 5. Six-Week Connector Roadmap

| Week   | Milestone                     | Key Deliverables                                                                 |
| :----- | :---------------------------- | :------------------------------------------------------------------------------- |
| **W1** | Workspace & Abstractions      | `lotusx-core`, `connector-binance`…; implement `RestClient` + `WsSession` traits |
| **W2** | Type Safety Upgrade           | Replace strings with `Decimal`, normalize `Symbol`                               |
| **W3** | Rate-Limit & Retry Middleware | Unified `RateLimiter` & `RetryPolicy`, dynamic quotas                            |
| **W4** | **OKX Connector MVP**         | Full market, trade, account coverage; CI on OKX testnet                          |
| **W5** | Batch / Amend Support         | `place_batch`, `amend_order` for Binance & Bybit; benchmark latency              |
| **W6** | Observability & Docs          | `tracing` + Prometheus metrics, README with Grafana dashboard sample             |

---

## 6. Summary

Strengthening the connector layer along these axes—**capabilities, type safety, shared infrastructure, observability, and additional exchanges**—will transform LotusX into a plug-and-play, production-grade foundation for any cross-exchange arbitrage engine.
