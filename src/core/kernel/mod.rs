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
/// # Example Usage
///
/// ```rust,no_run
/// use lotusx::core::kernel::*;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a REST client
/// let config = RestClientConfig::new("https://api.exchange.com".to_string(), "exchange".to_string());
/// let api_key = "your_api_key".to_string();
/// let secret_key = "your_secret_key".to_string();
/// let signer = Arc::new(HmacSigner::new(api_key, secret_key, HmacExchangeType::Binance));
/// let client = RestClientBuilder::new(config)
///     .with_signer(signer)
///     .build()?;
///
/// // Note: WebSocket usage would require an exchange-specific codec
/// // which is implemented in the exchange modules, not the kernel
/// # Ok(())
/// # }
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
