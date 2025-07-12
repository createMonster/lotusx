/// `LotusX` Kernel - Unified transport layer for all exchanges
///
/// This module provides a unified, exchange-agnostic transport layer for both
/// REST and WebSocket communication. The kernel follows strict separation of
/// concerns, containing only transport logic and generic interfaces.
///
/// # Architecture
///
/// The kernel is organized around three main components:
///
/// ## Transport Layer
/// - `RestClient`: Unified HTTP client interface
/// - `WsSession`: WebSocket connection management
/// - `ReconnectWs`: Automatic reconnection wrapper
///
/// ## Authentication
/// - `Signer`: Pluggable authentication interface
/// - `HmacSigner`: HMAC-SHA256 for Binance/Bybit
/// - `Ed25519Signer`: Ed25519 for Backpack
/// - `JwtSigner`: JWT for Paradex
///
/// ## Message Handling
/// - `WsCodec`: Exchange-specific message encoding/decoding
///
/// # Key Principles
///
/// 1. **Transport Only**: The kernel contains NO exchange-specific logic
/// 2. **Pluggable**: All components are trait-based and configurable
/// 3. **Type Safe**: Strong typing throughout with proper error handling
/// 4. **Observable**: Comprehensive tracing and metrics support
/// 5. **Testable**: Dependency injection for easy testing
///
/// # Real-World Usage Examples
///
/// ## Basic REST-Only Connector
/// ```rust,no_run
/// use lotusx::core::kernel::*;
/// use lotusx::core::config::ExchangeConfig;
/// use lotusx::core::types::Market;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ExchangeConfig::new("api_key".to_string(), "secret_key".to_string());
/// let rest_config = RestClientConfig::new("https://api.binance.com".to_string(), "binance".to_string());
/// let signer = Arc::new(HmacSigner::new(
///     config.api_key().to_string(),
///     config.secret_key().to_string(),
///     HmacExchangeType::Binance,
/// ));
/// let rest = RestClientBuilder::new(rest_config)
///     .with_signer(signer)
///     .build()?;
///
/// // Use typed responses for zero-copy deserialization
/// let markets: Vec<Market> = rest.get_json("/api/v3/exchangeInfo", &[], false).await?;
/// # Ok(())
/// # }
/// ```
///
/// ## WebSocket Integration with Codec
/// ```rust,no_run
/// use lotusx::core::kernel::*;
/// use lotusx::exchanges::binance::codec::{BinanceCodec, BinanceMessage};
///
/// # async fn websocket_example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create exchange-specific codec
/// let codec = BinanceCodec;
/// let ws = TungsteniteWs::new(
///     "wss://stream.binance.com:443/ws".to_string(),
///     "binance".to_string(),
///     codec,
/// );
///
/// // Subscribe to streams
/// let streams = ["btcusdt@ticker", "ethusdt@ticker"];
/// // Note: In a real implementation, you'd call ws.subscribe(&streams).await?;
///
/// // Receive typed messages would be handled by the codec
/// // This is just an example of the pattern
/// # Ok(())
/// # }
/// ```
///
/// ## Factory Pattern Implementation
/// ```rust,no_run
/// use lotusx::core::kernel::*;
/// use lotusx::core::config::ExchangeConfig;
/// use lotusx::core::errors::ExchangeError;
/// use lotusx::exchanges::binance::codec::BinanceCodec;
/// use lotusx::exchanges::binance::signer::BinanceSigner;
/// use lotusx::exchanges::binance::connector::BinanceConnector;
/// use std::sync::Arc;
///
/// pub fn create_exchange_connector(
///     config: ExchangeConfig,
///     enable_websocket: bool,
/// ) -> Result<(), ExchangeError> {
///     let base_url = "https://api.binance.com".to_string();
///     let exchange_name = "binance".to_string();
///     
///     // REST client setup
///     let rest_config = RestClientConfig::new(base_url, exchange_name.clone());
///     let mut rest_builder = RestClientBuilder::new(rest_config);
///     
///     if config.has_credentials() {
///         let signer = Arc::new(BinanceSigner::new(
///             config.api_key().to_string(),
///             config.secret_key().to_string(),
///         ));
///         rest_builder = rest_builder.with_signer(signer);
///     }
///     
///     let rest = rest_builder.build()?;
///     
///     // Create connector based on WebSocket requirement
///     if enable_websocket {
///         let ws_url = "wss://stream.binance.com:443/ws".to_string();
///         let codec = BinanceCodec;
///         let ws = TungsteniteWs::new(ws_url, exchange_name, codec);
///         let _connector = BinanceConnector::new(rest, ws, config);
///     } else {
///         let _connector = BinanceConnector::new_without_ws(rest, config);
///     }
///     
///     Ok(())
/// }
/// ```
///
/// # Performance Benefits
///
/// - **Zero-copy deserialization**: `get_json<T>()` eliminates intermediate `serde_json::Value` allocations
/// - **Typed responses**: Compile-time guarantees eliminate runtime serialization errors
/// - **Efficient WebSocket handling**: Codec pattern minimizes message processing overhead
/// - **Connection pooling**: Automatic HTTP connection reuse via reqwest
/// - **Tracing integration**: Minimal overhead observability with structured logging
///
/// # Common Patterns
///
/// ## Error Handling
/// ```rust,no_run
/// use lotusx::core::errors::ExchangeError;
/// use lotusx::core::types::Ticker;
/// use lotusx::core::kernel::RestClient;
/// use tracing::instrument;
///
/// struct ExchangeClient<R: RestClient> {
///     rest: R,
/// }
///
/// impl<R: RestClient> ExchangeClient<R> {
///     #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
///     async fn get_ticker(&self, symbol: &str) -> Result<Ticker, ExchangeError> {
///         let params = [("symbol", symbol)];
///         self.rest.get_json("/api/v3/ticker/24hr", &params, false).await
///     }
/// }
/// ```
///
/// ## Authentication Checks
/// ```rust,no_run
/// use lotusx::core::errors::ExchangeError;
/// use lotusx::core::config::ExchangeConfig;
///
/// struct ExchangeClient {
///     config: ExchangeConfig,
/// }
///
/// impl ExchangeClient {
///     fn ensure_authenticated(&self) -> Result<(), ExchangeError> {
///         if !self.config.has_credentials() {
///             return Err(ExchangeError::AuthenticationRequired);
///         }
///         Ok(())
///     }
/// }
/// ```
///
/// ## WebSocket Message Handling
/// ```rust,no_run
/// use lotusx::core::errors::ExchangeError;
/// use lotusx::core::types::{Ticker, OrderBook, Trade};
/// use lotusx::core::kernel::WsSession;
/// use lotusx::exchanges::binance::codec::{BinanceCodec, BinanceMessage};
///
/// struct ExchangeClient<W: WsSession<BinanceCodec>> {
///     ws: W,
/// }
///
/// impl<W: WsSession<BinanceCodec>> ExchangeClient<W> {
///     async fn handle_websocket_stream(&mut self) -> Result<(), ExchangeError> {
///         // Note: This is a simplified example of the pattern
///         // In practice, you'd use the codec to decode messages
///         Ok(())
///     }
/// }
/// ```
pub mod codec;
pub mod rest;
pub mod signer;
pub mod ws;

// Re-export key types for convenience
pub use codec::WsCodec;
pub use rest::{ReqwestRest, RestClient, RestClientBuilder, RestClientConfig};
pub use signer::{Ed25519Signer, HmacExchangeType, HmacSigner, JwtSigner, SignatureResult, Signer};
pub use ws::{ReconnectWs, TungsteniteWs, WsSession};
