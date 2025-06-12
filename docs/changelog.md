# Changelog

All notable changes to the LotusX project will be documented in this file.

## 2025-06-12

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
