Below is a **template you can replicate for every venue** (Binance, Bybit, OKX, …) that sits cleanly on top of the new **kernel** while satisfying all the “umbrella traits” in `src/core/traits.rs` (and their sub-traits for market-data, trading, account, etc.).
It keeps **one responsibility per file**, enforces **compile-time type safety**, and avoids leaking transport-level details into business logic.

```
src/
└── exchanges/
    └── <exchange>/               # e.g. binance, bybit, okx
        ├── mod.rs                # public façade, builder helpers
        ├── types.rs              # serde structs <— raw JSON
        ├── conversions.rs        # String ↔︎ Decimal, Symbol, etc.
        ├── signer.rs             # Hmac / Ed25519 / JWT (only if needed)
        │
        ├── codec.rs              # impl WsCodec   (WebSocket dialect)
        ├── rest.rs               # thin typed wrapper around RestClient
        │
        ├── connector/
        │   ├── market_data.rs    # impl MarketDataSource
        │   ├── trading.rs        # impl TradingEngine   (orders)
        │   ├── account.rs        # impl AccountInfoSource
        │   └── mod.rs            # re-export, compose sub-traits
        ├── builder.rs            # fluent builder → concrete connector
```

---

## 1  Transport layer (**kernel-side**) remains generic

```rust
// kernel/ws.rs (already exists)
pub struct WsSession<C: WsCodec> { /* transport only */ }

// kernel/rest.rs (already exists)
pub struct ReqwestRest { /* transport only */ }
```

**`WsSession` and `ReqwestRest` know nothing about Binance or Bybit**.
Every exchange instead supplies:

* a **codec** (encode/decode frames)
* a **signer** (optional HMAC / JWT)
* strongly-typed request/response helpers

---

## 2  Typed wrappers (**exchange/rest.rs**)

```rust
pub struct <Ex>Rest<'a, R: RestClient>(&'a R);

impl<'a, R: RestClient> <Ex>Rest<'a, R> {
    pub async fn klines(
        &self,
        sym: &str,
        ivl: KlineInterval,
        lim: Option<u32>
    ) -> Result<Vec<RawKline>, ExchangeError> {
        self.0
            .get_json("/api/v3/klines", &[("symbol", sym), ("interval", ivl.as_str())], lim)
            .await
    }

    // …other endpoints…
}
```

*All REST specifics are here; the connector never touches URLs.*

---

## 3  WebSocket dialect (**exchange/codec.rs**)

```rust
pub enum <Ex>WsEvent {
    Trade(Trade),
    OrderBook(BookDepth),
    // …
}

pub struct <Ex>Codec;
impl WsCodec for <Ex>Codec {
    type Message = <Ex>WsEvent;

    fn encode_subscription(&self, streams: &[impl AsRef<str>]) -> Result<Message> { /* … */ }
    fn encode_unsubscription(&self, streams: &[impl AsRef<str>]) -> Result<Message> { /* … */ }
    fn decode_message(&self, msg: Message) -> Result<Option<Self::Message>> { /* … */ }
}
```

*Only encode/decode logic lives here; no ping/pong, no reconnect.*

---

## 4  Sub-trait implementations (**exchange/connector/**)

### market\_data.rs – `MarketDataSource`

```rust
pub struct MarketData<R, W> {
    rest: <Ex>Rest<'static, R>,
    ws:   ReconnectWs<W>,
}

#[async_trait]
impl<R, W> MarketDataSource for MarketData<R, W>
where
    R: RestClient + Send + Sync,
    W: WsSession<<Ex>Codec> + Send + Sync,
{
    async fn get_klines(&self, req: GetKlines) -> Result<Vec<Kline>, ExchangeError> {
        let raw = self.rest.klines(&req.symbol, req.interval, req.limit).await?;
        Ok(raw.into_iter().map(convert_raw_kline).collect())
    }

    async fn subscribe_ticks(&self, symbols: Vec<String>)
        -> Result<mpsc::Receiver<<Ex>WsEvent>, ExchangeError>
    {
        self.ws.subscribe(symbols.iter().map(|s| format!("{s}@trade")).collect())
    }

    // …
}
```

### trading.rs – `TradingEngine`

```rust
pub struct Trading<R> {
    rest: <Ex>Rest<'static, R>,
}

#[async_trait]
impl<R: RestClient + Send + Sync> TradingEngine for Trading<R> {
    async fn place_order(&self, req: OrderReq) -> Result<Order, ExchangeError> {
        self.rest.place_order(req).await.map(convert_raw_order)
    }
}
```

### account.rs – `AccountInfoSource`

```rust
pub struct Account<R> {
    rest: <Ex>Rest<'static, R>,
}

#[async_trait]
impl<R: RestClient + Send + Sync> AccountInfoSource for Account<R> {
    async fn balances(&self) -> Result<Vec<Balance>, ExchangeError> {
        self.rest.balances().await.map(convert_raw_balance)
    }
}
```

### connector/mod.rs – compose traits

```rust
pub struct <Ex>Connector<R, W> {
    pub market:  MarketData<R, W>,
    pub trading: Trading<R>,
    pub account: Account<R>,
}

impl<R, W> <Ex>Connector<R, W> {
    pub fn new(rest: R, ws: W) -> Self {
        let rest_ref: &'static R = Box::leak(Box::new(rest));   // lifetime hack
        Self {
            market:  MarketData { rest: <Ex>Rest(rest_ref), ws },
            trading: Trading   { rest: <Ex>Rest(rest_ref) },
            account: Account   { rest: <Ex>Rest(rest_ref) },
        }
    }
}
```

---

## 5  Builder and public facade (**exchange/builder.rs & mod.rs**)

```rust
pub fn build_connector(
    cfg: ExchangeConfig,
) -> Result<<Ex>Connector<ReqwestRest, TungsteniteWs<<Ex>Codec>>, ExchangeError> {
    // --- REST ---
    let rest = ReqwestRest::builder()
        .base_url(cfg.rest_url)
        .exchange_name("<ex>".into())
        .signer(<Ex>Signer::new(cfg.key, cfg.secret))
        .build()?;

    // --- WS ---
    let ws = TungsteniteWs::new(cfg.ws_url, "<ex>".into(), <Ex>Codec)
        .into_reconnect(cfg.reconnect);

    Ok(<Ex>Connector::new(rest, ws))
}
```

`mod.rs` simply re-exports:

```rust
pub use builder::build_connector;
pub use connector::{ <Ex>Connector, MarketData, Trading, Account };
```

Down-stream users can therefore:

```rust
let ex = build_connector(cfg)?;
let prices = ex.market.get_klines(req).await?;
ex.trading.place_order(order).await?;
```

---

## 6  SRP & separation-of-concerns recap

| Concern                          | File / Layer                                           |
| -------------------------------- | ------------------------------------------------------ |
| Transport, reconnect, rate-limit | **kernel**                                             |
| WS JSON dialect                  | `codec.rs`                                             |
| REST endpoint paths + auth       | `rest.rs`, `signer.rs`                                 |
| Data-model conversion            | `conversions.rs`                                       |
| Business interfaces (traits)     | `connector/market_data.rs`, `trading.rs`, `account.rs` |
| Composition / DI                 | `connector/mod.rs`, `builder.rs`                       |
| Testing & demos                  | `tests/`, `examples/`                                  |

Each module has **one reason to change**; adding a new venue touches only its own subtree, never the kernel or other exchanges.

---

### Copy-and-go checklist for a **new exchange**

1. `cargo new exchanges/kraken` (for example).
2. Paste raw REST/WS JSON → generate `types.rs` with quicktype.
3. Write `codec.rs` encode/ decode logic, unit-test with captured frames.
4. Wrap REST endpoints in `rest.rs`; add signer if needed.
5. Implement sub-trait files under `connector/`.
6. Provide `builder.rs` that wires rest+ws with defaults.
7. Add golden-file tests & a runnable example.

You now have a fully-typed, kernel-compatible connector satisfying **all sub-traits** in `traits.rs`, with minimal boilerplate and maximum maintainability.
