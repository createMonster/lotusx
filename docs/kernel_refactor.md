# LotusX Unified REST / WebSocket Kernel – Best-Practice Guide

> **Goal**
> Evolve LotusX from “every connector rolls its own HTTP & WS client” into a **single, composable kernel** that handles signing, rate-limiting, retries and telemetry, so each exchange connector focuses only on *end-points & field mapping*.

---

## 1  Layered Architecture

```
lotusx
├── core
│   ├── kernel          # ★ NEW: shared transport layer
│   │   ├── rest.rs     # RestClient trait + ReqwestRest impl
│   │   ├── ws.rs       # WsSession trait + TungsteniteWs impl
│   │   └── signer.rs   # Signer trait + Hmac / Ed25519 / …
│   ├── types.rs
│   ├── errors.rs
│   └── traits.rs       # ExchangeConnector trait
└── exchanges
    └── binance / bybit / …
```

*All cross-cutting concerns live once in `kernel`, connectors just compose.*

---

## 2  `RestClient` Design

| Target                 | Practice                                                                                                                                                                                                                                                                                            |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Unified API**        | `rust<br/>#[async_trait]<br/>pub trait RestClient {<br/>  async fn get&lt;T: DeserializeOwned&gt;(&self, ep:&str, qs:&[(&str,&str)]) -> Result&lt;T&gt;;<br/>  async fn post&lt;T: DeserializeOwned&gt;(&self, ep:&str, body:Option&lt;Value&gt;) -> Result&lt;T&gt;;<br/>  // … delete, put<br/>}` |
| **Pluggable signing**  | `Signer` trait → impls `BinanceHmac`, `BybitHash`, `ParadexEd25519` …                                                                                                                                                                                                                               |
| **Rate-limit & retry** | `tower::ServiceBuilder` -→ `Retry ∘ RateLimit ∘ Tracing ∘ ReqwestTransport`                                                                                                                                                                                                                         |
| **Observability**      | `tracing` spans: `rest_call.exchange="binance" path="/api/v3/order" …`                                                                                                                                                                                                                              |
| **Testing**            | `RestClientMock` returns local JSON; unit-tests assert *signature & URL*, never hit the wire                                                                                                                                                                                                        |

---

## 3  `WsSession` Design

| Target                    | Practice                                                                                                                                                                                                                                                                                                              |
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Lifecycle**             | `rust<br/>#[async_trait]<br/>pub trait WsSession {<br/>  async fn connect(&mut self)       -> Result<()>;<br/>  async fn send(&mut self, msg:&WsMsg) -> Result<()>;<br/>  async fn next<'a>(&'a mut self)      -> Option&lt;Result&lt;WsMsg&gt;&gt;;<br/>  async fn close(&mut self)            -> Result<()>;<br/>}` |
| **Heartbeat & reconnect** | Wrap with `ReconnectWs<T>` (auto `ping/pong`, exponential back-off, resubscribe)                                                                                                                                                                                                                                      |
| **Middleware chain**      | `Deflate ∘ Dechunk ∘ Parse ∘ UserParser`                                                                                                                                                                                                                                                                              |
| **Protocol quirks**       | Connector just calls `build_subscribe(["ticker","depth"])`; session sends frames                                                                                                                                                                                                                                      |

---

## 4  Connector Refactor Pattern

```rust
pub struct BinanceConnector<R: RestClient, W: WsSession> {
    rest: R,
    ws:   W,
    base: String,
}

#[async_trait]
impl<R, W> ExchangeConnector for BinanceConnector<R, W>
where
    R: RestClient + Send + Sync,
    W: WsSession  + Send + Sync,
{
    async fn place_order(&self, req: NewOrder) -> Result<Order> {
        self.rest.post("/api/v3/order", &req).await
    }
    async fn subscribe_market_data(&mut self, streams: Vec<&str>) -> Result<()> {
        self.ws.subscribe(streams).await
    }
}
```

Dependency injection keeps the connector agnostic of transport details:

```rust
let rest = ReqwestRest::builder()
    .signer(BinanceHmac::new(key, secret))
    .rate_limiter(Limiter::binance())
    .build();

let ws = TungsteniteWs::new(url).with_signer(...);
let binance = BinanceConnector::new(rest, ws);
```

---

## 5  Observability & Quality Gates

* **Metrics**: export `latency_ms`, `retry_count`, `rate_limited_total` to Prometheus.
* **Coverage & benches**: `cargo-tarpaulin` ≥ 90 %, `criterion` p99 latency vs legacy target ≤ 1.2×.
* **LLM-powered code audit**: bot comments on PR for deadlocks / UB / race conditions.

---

## 6  Feature Flags & Extensibility

```toml
[features]
default = ["binance", "bybit"]
binance  = []
bybit    = []
```

Future HTTP (hyper, http/2) or QUIC (`quinn`) transports slide underneath `RestClient` / `WsSession` unchanged.

---

## 7  Migration Roadmap (≤ 3 months)

| Phase                 | Weeks     | Deliverables                                                                                                                      |
| --------------------- | --------- | --------------------------------------------------------------------------------------------------------------------------------- |
| **Kernel Extraction** | **W 1-2** | • Create `core/kernel` <br/>• Move existing WS logic, purge exchange-specific hacks <br/>• Add `RestClient` trait + `ReqwestRest` |
| **REST Swap-in**      | **W 3-4** | • Port **GET** market-data for Binance & Bybit <br/>• Unit-tests for signature correctness                                        |
| **WebSocket Swap-in** | **W 5-6** | • Replace `WebSocketManager` with `TungsteniteWs` <br/>• Support `SUBSCRIBE/UNSUBSCRIBE`                                          |
| **Private Endpoints** | **W 7-8** | • Orders / balances / withdrawals via new kernel <br/>• End-to-end tests green                                                    |
| **Unified Telemetry** | **W 9**   | • `tracing` + Prometheus metrics <br/>• Dashboards for latency & error budgets                                                    |
| **Docs & Scaffold**   | **W 10**  | • Update `README` <br/>• `lotusx new-exchange foo` CLI scaffold generator                                                         |

---

### 💡 Outcome

* **-40-60 % connector code**; adding a new exchange ≈ 1 day.
* Standardised retries & rate limits → stronger production stability.
* Clear separation of concerns → ready for AI-generated connector blueprints.

---

Happy refactoring — and may LotusX grow into a **high-performance, hot-swappable connection hub** for all your market-making adventures!
