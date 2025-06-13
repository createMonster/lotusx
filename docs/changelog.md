# Changelog

All notable changes to the LotusX project will be documented in this file.

## PR-4

### Added
- **Comprehensive Latency Testing Framework**: New performance benchmarking tool for exchange API methods
  - **Multi-Exchange Testing**: Tests Binance Spot, Binance Perpetual, and Hyperliquid simultaneously
  - **Method-Specific Benchmarks**: Individual latency measurements for `get_markets`, `get_klines`, and WebSocket connections
  - **Statistical Analysis**: Comprehensive statistics including min/max/average/median and standard deviation
  - **Sequential Request Testing**: Measures performance of multiple API calls in sequence
  - **WebSocket Latency**: Tests connection establishment and first message reception times
  - **Error Handling**: Graceful handling of unsupported features (e.g., Hyperliquid k-lines)
  - **Rate Limiting Protection**: Built-in delays between requests to avoid API rate limits

### Enhanced
- **K-lines Implementation**: Complete historical candlestick data support across exchanges
  - **Binance Spot**: Full k-lines API with configurable intervals, limits, and time ranges
  - **Binance Perpetual**: Complete k-lines support matching spot implementation
  - **Hyperliquid**: Proper placeholder implementation with clear error messaging for unsupported feature
  - **Consistent Interface**: Unified `get_klines` method across all exchanges via `MarketDataSource` trait
  - **Flexible Parameters**: Support for symbol, interval, limit, start_time, and end_time parameters
  - **Error Handling**: Proper error types and messaging for different failure scenarios

### Technical Improvements
- **Trait Object Support**: Latency testing uses trait objects for polymorphic exchange testing
- **Performance Metrics**: Detailed timing analysis with microsecond precision
- **Code Quality**: All examples pass clippy with `-D warnings` enabled
- **Documentation**: Comprehensive inline documentation for all new features

### Performance Insights
- **Binance Spot**: `get_markets` ~4s (1445 markets), `get_klines` ~214ms avg
- **Binance Perpetual**: `get_markets` ~1.4s (509 markets), `get_klines` ~234ms avg  
- **Hyperliquid**: `get_markets` ~399ms (199 markets), k-lines not supported
- **WebSocket**: Connection times <100ms across all exchanges

## PR-3

### Refactored
- **Binance & Binance-Perp Exchange Architecture**: Complete refactoring to follow hyperliquid's modular structure
  - **Massive Simplification**: Reduced client files from 467/515 lines to 28 lines each (-95% reduction)
  - **Modular Design**: Separated concerns into dedicated modules:
    - `client.rs` - Core client struct and basic setup only
    - `account.rs` - Account functions implementing `AccountInfo` trait
    - `trading.rs` - Trading functions implementing `OrderPlacer` trait  
    - `market_data.rs` - Market data functions implementing `MarketDataSource` trait
    - `converters.rs` - Data conversion and parsing functions
    - `auth.rs` - Authentication and request signing
    - `types.rs` - Exchange-specific data structures
    - `mod.rs` - Module exports and re-exports
  - **Code Reuse**: Binance-perp now reuses binance auth module for consistent authentication
  - **Consistency**: All exchanges (binance, binance-perp, hyperliquid) now follow identical organizational patterns
  - **Maintainability**: Much easier to locate and modify specific functionality
  - **100% Functionality Preservation**: All existing features maintained with improved organization

### Technical Improvements
- **API Consistency**: Standardized configuration access patterns across all modules
- **WebSocket Fixes**: Updated WebSocket implementations to use correct API methods
- **Import Cleanup**: Removed unused imports and dependencies
- **Type Safety**: Improved type handling in binance-perp position and order responses
- **Error Handling**: Consistent error handling patterns across all exchange modules

### Code Quality
- **Compilation Success**: All refactoring changes compile successfully with `cargo check`
- **Single Responsibility**: Each module now has a clear, single purpose
- **Reduced Complexity**: Eliminated massive monolithic client files that were hard to navigate
- **Pattern Consistency**: Established mature, reusable patterns for future exchange integrations

## PR-2

### Added
- **WebSocket Market Data Support for Hyperliquid**: Implemented full WebSocket functionality for real-time market data streaming
  - Support for multiple subscription types: Ticker, OrderBook, Trades, and Klines (candlestick data)
  - Real-time data conversion from Hyperliquid WebSocket format to core MarketDataType
  - Automatic WebSocket connection management with heartbeat/ping-pong mechanism
  - Configurable auto-reconnection with exponential backoff
  - Proper error handling and connection lifecycle management
  - Multi-symbol subscription support with efficient message routing
  - **Code Quality Improvements**: Refactored WebSocket implementation to reduce nested complexity
    - Extracted helper functions to eliminate deeply nested loops (max 3 levels)
    - Improved code readability and maintainability with single-responsibility functions
    - Used iterator chains with `flat_map` to avoid nested loops in subscription creation
    - Separated message processing logic into focused helper functions
- **Hyperliquid Exchange Integration** - Complete implementation of Hyperliquid decentralized perpetual exchange support
  - EIP-712 authentication with secp256k1 cryptographic signing and Keccak256 hashing
  - Trait-based interface implementing `MarketDataSource`, `OrderPlacer`, and `AccountInfo` traits
  - Support for both testnet and mainnet environments
  - Secure wallet address derivation from private keys
  - Type conversions between core library types and Hyperliquid-specific types
  - Market data retrieval for available trading pairs
  - Order placement and cancellation functionality
  - Account balance and position management
  - Comprehensive example demonstrating all features

### Dependencies Added
- `secp256k1` (0.28) with `rand-std` features for cryptographic signing
- `sha3` (0.10) for Keccak256 hashing required by EIP-712
- `chrono` (0.4) for timestamp handling in order responses

### Technical Details
- **Authentication Module** (`src/exchanges/hyperliquid/auth.rs`)
  - EIP-712 structured data signing
  - Secure private key handling with memory cleanup
  - Ethereum-style signature formatting (r + s + v)
  - Nonce generation for replay protection

- **Client Implementation** (`src/exchanges/hyperliquid/client.rs`)
  - Multiple constructor patterns (new, with_private_key, read_only)
  - Trait method implementations only (removed extra helper methods)
  - Proper error handling and type safety
  - HTTP client for REST API communication

- **Type System** (`src/exchanges/hyperliquid/types.rs`)
  - Complete type definitions for Info and Exchange endpoints
  - Serde serialization/deserialization for all API structures
  - Request/response types for orders, balances, positions, and market data

- **Example** (`examples/hyperliquid_example.rs`)
  - Demonstrates read-only market data access
  - Shows authenticated operations (balance, positions, orders)
  - Includes proper error handling and trait usage

### Code Quality
- Fixed all clippy warnings and linting issues
- Added `Default` implementation for `HyperliquidAuth`
- Proper async/await patterns throughout
- Comprehensive error handling with `ExchangeError` types
- Memory-safe credential handling with `zeroize`
