# Changelog

All notable changes to the LotusX project will be documented in this file.

## PR-16

### Refactored
- **Core Module Simplification**: Major refactoring of bloated core modules for better maintainability and HFT performance
  - **errors.rs**: Reduced from 516 lines to 127 lines (-75%) - removed over-engineered optimizations and excessive boilerplate
  - **types.rs**: Reduced from 684 lines to 440 lines (-35%) - simplified Symbol struct and removed complex optimizations
  - **Focused Architecture**: Eliminated unnecessary complexity while maintaining essential HFT features

### Enhanced
- **HFT WebSocket Optimization**: Improved WebSocket performance with kernel-based architecture
  - **WsConfig Struct**: Added HFT-optimized configuration with bulk message sending capabilities
  - **Kernel Integration**: Updated all exchanges to use unified WebSocket kernel implementation
  - **Connection Management**: Enhanced reconnection logic and statistics tracking
  - **Latency Optimization**: Streamlined message processing for sub-millisecond performance

- **Exchange Configuration**: Added caching mechanisms for improved HFT performance
  - **Credential Validation Caching**: Cached validation results to reduce authentication overhead
  - **Environment File Handling**: Replaced unmaintained dotenv with secure in-house implementation
  - **Inline Optimizations**: Added performance-critical method inlining for API key access

### Removed
- **Unsupported Features**: Eliminated Seconds1 interval support across all exchanges
  - **Binance**: Removed Seconds1 from kline interval conversions
  - **Bybit**: Removed Seconds1 from both spot and perpetual implementations
  - **Backpack**: Removed Seconds1 from kline interval enum
  - **Hyperliquid**: Removed Seconds1 from conversion functions
  - **Paradex**: Removed Seconds1 from kline interval implementation

- **Legacy Components**: Removed deprecated WebSocket management structures
  - **WebSocketManager**: Eliminated redundant WebSocket manager in favor of kernel implementation
  - **BybitWebSocketManager**: Removed exchange-specific WebSocket manager

### Fixed
- **Compilation Errors**: Resolved all compilation issues from refactoring
  - **Symbol Construction**: Fixed Symbol::new() usage across all exchange implementations
  - **Error Handling**: Unified error handling patterns with simplified error types
  - **Type Safety**: Improved type conversion safety and validation
  - **Import Cleanup**: Removed unused imports and dependencies

### Technical Improvements
- **Code Quality**: Achieved 100% compilation success with improved maintainability
- **Performance**: HFT-optimized patterns for sub-millisecond latency requirements
- **Architecture**: Consistent kernel-based WebSocket implementation across all exchanges
- **Error Handling**: Simplified and focused error types for better debugging and performance

## PR-15

### Added
- **Hyperliquid WebSocket Implementation Analysis**: Comprehensive documentation and analysis of WebSocket implementation challenges
  - **Implementation Status Documentation**: Complete analysis of current Hyperliquid exchange implementation strengths and weaknesses
  - **WebSocket Architecture Analysis**: Detailed examination of WebSocket implementation issues and architectural constraints
  - **Improvement Planning**: Strategic planning for leveraging existing kernel infrastructure for WebSocket management
  - **Error Handling Documentation**: Analysis of enhanced error handling improvements already implemented
  - **Order Management Documentation**: Coverage of completed order modification support and builder configuration enhancements

### Technical Analysis
- **WebSocket Implementation Issues** (`docs/hyperliquid_analysis.md`)
  - **Incomplete WebSocket Support**: Documented current WebSocket subscription limitations and missing functionality
  - **Subscription Handling**: Analysis of limited multi-stream subscription capabilities
  - **Reconnection Logic**: Identified lack of robust reconnection mechanisms
  - **Error Handling**: Evaluation of generic error messages and inconsistent error handling patterns
  - **Code Duplication**: Analysis of redundant code and refactoring opportunities

- **Strategic Improvement Plan** (`docs/hyperliquid_improvement_plan.md`)
  - **Completed Improvements**: Documentation of enhanced error handling, order modification, and builder configuration
  - **Critical Issue Identification**: Detailed analysis of WebSocket trait design limitations and architectural mismatches
  - **Kernel Infrastructure Solution**: Strategic recommendation to leverage existing `ReconnectWs` and `WsSession` components
  - **Implementation Strategy**: Phased approach for proper WebSocket integration using kernel components

### Performance Considerations
- **HFT Optimization**: Analysis includes HFT-focused improvements like `#[cold]` annotations for error paths
- **Latency Requirements**: WebSocket solution designed to meet sub-millisecond latency requirements
- **Connection Management**: Proposed architecture leverages kernel's efficient connection pooling and management
- **Memory Efficiency**: Strategy to avoid duplication of WebSocket management functionality

### Architecture Recommendations
- **Kernel Component Utilization**: Recommendation to use existing `TungsteniteWs` and `ReconnectWs` infrastructure
- **Channel-Based Distribution**: Proposed message distribution system for thread-safe WebSocket data streaming
- **Codec Integration**: Strategy to leverage existing `HyperliquidCodec` for message handling
- **Subscription Management**: Framework for batch subscription handling and state tracking

### Development Priorities
- **WebSocket Implementation**: Priority 1 focus on fixing WebSocket subscription handling using kernel components
- **Feature Parity**: Priority 2 completion of K-line pagination and vault address support
- **Testing Framework**: Priority 3 comprehensive WebSocket integration testing and validation

### Code Quality Improvements
- **Documentation Standards**: Comprehensive analysis documentation following established patterns
- **Error Handling Enhancement**: Detailed coverage of improved error handling with 8 additional error variants
- **Architectural Consistency**: Alignment with existing kernel patterns and HFT optimization principles
- **Implementation Roadmap**: Clear next steps for completing WebSocket functionality

## PR-13

### Added
- **Kernel Architecture**: Complete architectural refactoring with unified transport layer
  - **Core Kernel Module**: New `src/core/kernel/` with codec, REST client, WebSocket session, and signer abstractions
  - **Unified Transport**: Standardized HTTP and WebSocket communication patterns across all exchanges
  - **Builder Pattern**: New builder-based exchange instantiation replacing direct constructors
  - **Modular Exchange Structure**: Consistent file organization with separate concerns per module
  - **Transport Abstraction**: Generic `RestClient` and `WebSocketSession` traits for protocol-agnostic communication

### Technical Implementation
- **Kernel Components** (`src/core/kernel/`)
  - **Codec**: Unified message encoding/decoding with `Codec` trait
  - **REST Client**: Generic HTTP client abstraction with `RestClient` trait
  - **WebSocket Session**: Unified WebSocket handling with `WebSocketSession` trait
  - **Signer**: Authentication abstraction with `Signer` trait
  - **Transport Layer**: Protocol-agnostic communication infrastructure

- **Exchange Refactoring**: All exchanges updated to use kernel architecture
  - **Binance Spot & Perp**: Complete migration to builder pattern with modular structure
  - **Bybit Spot & Perp**: Full kernel integration with unified transport
  - **Hyperliquid**: Kernel-based architecture with EIP-712 authentication
  - **Backpack**: Unified transport with builder pattern
  - **Paradex**: Complete kernel integration with modular design

### Enhanced Architecture
- **Connector Pattern**: New `ExchangeConnector` structs composing sub-trait implementations
- **Modular Design**: Separate files for account, market data, trading, codec, conversions, REST, and signer
- **Builder Interface**: Consistent `ExchangeBuilder` pattern across all exchanges
- **Trait Composition**: Clean separation of concerns with focused trait implementations

### Performance Improvements
- **Unified Transport**: Optimized HTTP and WebSocket connection management
- **Memory Efficiency**: Reduced overhead through shared transport abstractions
- **Connection Pooling**: Efficient resource management in kernel layer
- **HFT Optimizations**: Low-latency patterns maintained with new architecture

### Breaking Changes
- **Exchange Instantiation**: All exchanges now use builder pattern instead of direct constructors
- **Client Removal**: Legacy client structs replaced with connector pattern
- **Import Changes**: Updated import paths for new modular structure
- **Configuration**: Builder-based configuration replacing direct config passing

### Code Quality
- **Consistency**: Unified patterns across all exchange implementations
- **Maintainability**: Clear separation of concerns with focused modules
- **Extensibility**: Easy addition of new exchanges with established patterns
- **Type Safety**: Enhanced compile-time validation through trait system

### Dependencies
- **num-traits**: Added for numeric trait abstractions in kernel layer

## PR-12

### Added
- **Comprehensive Type System Migration**: Complete migration to type-safe decimal arithmetic and symbol handling
  - **Core Type System**: New `rust_decimal::Decimal` integration with `serde-with-str` feature for high-precision arithmetic
  - **Type-Safe Symbols**: New `Symbol` type with validation and parsing capabilities
  - **Unified Data Types**: Consistent `Price`, `Quantity`, and `Volume` types across all exchanges
  - **Enhanced Market Data**: Type-safe market information with proper decimal precision
  - **Order Management**: Type-safe order placement with validated price and quantity fields
  - **Account Integration**: Type-safe balance and position tracking with decimal precision

### Technical Implementation
- **Core Library** (`src/core/types.rs`)
  - **Symbol Type**: New `Symbol` struct with validation and parsing from string formats
  - **Decimal Integration**: `rust_decimal::Decimal` for all monetary and quantity values
  - **Type Conversions**: Comprehensive conversion functions for string-to-type transformations
  - **Error Handling**: Type-safe error handling with proper validation messages

- **Exchange Implementations**: All exchanges updated to use new type system
  - **Binance Spot & Perp**: Complete migration with type-safe market data and trading
  - **Bybit Spot & Perp**: Full type system integration with decimal precision
  - **Hyperliquid**: Type-safe perpetual trading with decimal arithmetic
  - **Backpack**: Enhanced type safety for all market operations
  - **Paradex**: Complete type system migration with validation

### Performance & Quality Improvements
- **Memory Efficiency**: Optimized data structures with `arrayvec` and `bitvec` for HFT performance
- **Type Safety**: 100% compile-time validation of all monetary and quantity operations
- **Precision**: High-precision decimal arithmetic eliminating floating-point errors
- **Consistency**: Unified type handling across all exchange implementations
- **Error Reduction**: Eliminated runtime type conversion errors through compile-time validation

### Breaking Changes
- **Core Types**: `Market`, `OrderRequest`, and related structs now use type-safe fields
- **API Methods**: All exchange methods now return type-safe data structures
- **Symbol Handling**: Symbol fields now use `Symbol` type instead of strings
- **Decimal Precision**: All monetary values use `rust_decimal::Decimal` for precision

### Code Quality
- **Zero Runtime Errors**: Complete elimination of type conversion runtime errors
- **Consistent Patterns**: Unified type handling across all exchange modules
- **Enhanced Validation**: Compile-time validation of all data structures
- **Professional Standards**: Production-ready type safety for HFT applications

## PR-11

### Added
- **Comprehensive Funding Rates Support**: Complete funding rate functionality for perpetual exchanges
  - **Bybit Perpetual**: Full funding rate implementation using V5 API endpoints
  - **Hyperliquid**: Complete funding rate support with info endpoint integration
  - **Enhanced Existing**: Extended Binance Perp and Backpack with new `get_all_funding_rates()` method
  - **Core Infrastructure**: New `FundingRateSource` trait with three key methods:
    - `get_funding_rates()` - Current rates for specific symbols
    - `get_all_funding_rates()` - All available funding rates from exchange
    - `get_funding_rate_history()` - Historical funding rate data
  - **Data Structures**: Added `FundingRate` struct with ccxt-compatible fields
  - **Backward Compatibility**: Non-breaking trait composition maintaining existing interfaces

### Technical Implementation
- **Bybit Perpetual** (`src/exchanges/bybit_perp/`)
  - **Current Rates**: `/v5/market/tickers` endpoint for real-time funding rates and mark prices
  - **Historical Data**: `/v5/market/funding/history` endpoint with configurable time ranges
  - **Response Handling**: Custom string-to-integer deserializer for timestamp fields
  - **Error Handling**: Comprehensive V5 API error handling with proper context

- **Hyperliquid** (`src/exchanges/hyperliquid/`)
  - **Current Rates**: `metaAndAssetCtxs` info request for funding rates and mark prices
  - **Historical Data**: `fundingHistory` info request with time range support
  - **Type Safety**: Safe casting between u64/i64 types with overflow protection
  - **Error Handling**: Graceful handling of API response format variations

- **Core Enhancements** (`src/core/`)
  - **FundingRate Struct**: Complete data structure with funding rate, mark price, and timing fields
  - **FundingRateSource Trait**: Async trait with full method signatures for all funding rate operations
  - **Trait Composition**: `FundingRateConnector` and `PerpetualExchangeConnector` for enhanced functionality

### Performance Achievements
- **HFT Optimized**: All implementations meet sub-250ms response time requirements
  - **Binance Perp**: 537 symbols in 164ms
  - **Bybit Perp**: 566 symbols in 215ms
  - **Backpack**: Efficient per-symbol filtering approach
  - **Hyperliquid**: Single API call for all asset contexts

### Comprehensive Testing
- **18 Funding Rate Tests**: Complete test coverage across all exchange implementations
  - **Binance Perp**: 6 tests (single symbol, all rates, history, direct methods)
  - **Bybit Perp**: 3 tests (single symbol, all rates, history)
  - **Hyperliquid**: 3 tests (single symbol, all rates, history)
  - **Backpack**: 3 tests (single symbol, all rates, direct methods)
  - **Cross-Exchange**: 3 tests (error handling, concurrency, performance benchmarks)
- **Performance Testing**: Multi-exchange performance validation with HFT timing requirements
- **Error Handling**: Comprehensive error scenario testing with graceful degradation

### Code Quality Improvements
- **Clippy Compliance**: Resolved all clippy warnings including:
  - Option pattern optimizations (`map_or_else` usage)
  - Type casting safety improvements
  - Function complexity management
- **Type Safety**: Enhanced type safety with proper deserialization and casting
- **Memory Efficiency**: Pre-allocated vectors and optimal data structure usage

### API Compatibility
- **ccxt-Compatible**: Funding rate structure follows established ccxt patterns
- **Exchange-Specific**: Leverages each exchange's optimal API endpoints
- **Unified Interface**: Consistent trait-based interface across all exchanges
- **Flexible Parameters**: Support for symbol filtering, time ranges, and result limits

### Documentation
- **Implementation Guide**: Comprehensive 466-line guide covering all implementation aspects
- **Usage Examples**: Complete examples demonstrating all funding rate functionality
- **Performance Metrics**: Documented response times and HFT compliance

## PR-10

### Added
- **KlineInterval Enum**: Unified interval handling across all exchanges
  - **Type Safety**: Replaced string-based intervals with strongly-typed `KlineInterval` enum
  - **Exchange Compatibility**: Built-in support for different exchange interval formats
  - **Validation**: Compile-time validation of supported intervals per exchange
  - **Consistency**: Standardized interval handling across Binance, Bybit, and Hyperliquid

### Enhanced
- **Comprehensive Bybit Example**: Massively expanded `bybit_example.rs` with full functionality demonstration
  - **Complete API Coverage**: Demonstrates all Bybit Spot and Perpetual functionality
  - **WebSocket Testing**: Real-time data streaming with connection management
  - **Error Handling**: Comprehensive error handling patterns for production use
  - **V5 API Integration**: Shows proper usage of Bybit V5 API with fixed K-lines and WebSocket protocols
  - **Multi-Exchange Demo**: Side-by-side comparison of spot and perpetual features

- **Example Standardization**: Updated all examples to use new `KlineInterval` enum
  - **backpack_example.rs**: Added KlineInterval import and usage
  - **hyperliquid_example.rs**: Updated WebSocket subscriptions with typed intervals
  - **klines_example.rs**: Comprehensive interval demonstration with format conversion
  - **websocket_example.rs**: Updated WebSocket subscriptions with typed intervals

### Refactored
- **Core Traits**: Updated `MarketDataSource` trait to use `KlineInterval` instead of `String`
  - **Type Safety**: Eliminated string-based interval passing
  - **API Consistency**: All exchanges now use the same interval type
  - **Better Developer Experience**: IDE completion and compile-time validation

### Fixed
- **Code Quality Improvements**: Resolved clippy warnings across the codebase
  - **Function Length**: Added `#[allow(clippy::too_many_lines)]` for comprehensive examples
  - **String Creation**: Replaced manual `"".to_string()` with efficient `String::new()` calls
  - **Inefficient Conversions**: Fixed `symbol.to_string()` on `&&str` to use `(*symbol).to_string()`
  - **Lint Compliance**: All code now passes `cargo clippy --all-targets --all-features -- -D warnings`

### Technical Implementation
- **Interval Management**: `KlineInterval` enum with exchange-specific format conversion methods
- **WebSocket Integration**: Enhanced WebSocket examples with proper timeout handling
- **Performance Optimization**: String conversion improvements reduce overhead in HFT-critical paths
- **Code Standards**: Maintained strict clippy compliance for production-ready code quality
- **API Modernization**: Consistent use of typed parameters across all exchange interfaces

### Breaking Changes
- **MarketDataSource Trait**: `get_klines` method now accepts `KlineInterval` instead of `String`
- **Example Updates**: All examples updated to use new `KlineInterval` enum (existing code using string intervals will need updating)

## PR-9

### Enhanced
- **Unified Error Handling**: Standardized error handling patterns across all exchange implementations
  - **Consistent Error Types**: Harmonized error handling to use unified error types across all exchanges
  - **Improved Error Propagation**: Enhanced error propagation patterns for better debugging and monitoring
  - **Exchange Consistency**: All exchanges now follow the same error handling approach for maintainability
  - **Better Error Messages**: More descriptive and actionable error messages across all exchange operations
  - **Type Safety**: Improved type safety in error handling with consistent error enum usage

### Technical Implementation
- **Error Standardization**: Unified approach to handling API errors, network errors, and parsing errors
- **Code Quality**: Consistent error handling patterns make the codebase more maintainable
- **Developer Experience**: Better error messages and consistent error types improve debugging experience
- **Reliability**: More robust error handling improves overall system reliability

## PR-8

### Fixed
- **Bybit Trading Functionality**: Completed implementation of missing `cancel_order` functionality for both Bybit exchanges
  - **Bybit Spot**: Added complete `cancel_order` method implementation with proper V5 API integration
  - **Bybit Perpetual**: Added complete `cancel_order` method implementation for linear contracts
  - **API Consistency**: Both implementations now use proper Bybit V5 API endpoints and authentication
  - **Error Handling**: Comprehensive error handling for cancellation failures and API errors
  - **Trading Completeness**: Both Bybit exchanges now have full OrderPlacer trait implementation

### Technical Implementation
- **Bybit Spot Cancel Order** (`src/exchanges/bybit/trading.rs`)
  - Uses `/v5/order/cancel` endpoint with proper V5 API signature
  - Category set to "spot" for spot trading cancellations
  - Proper request body formatting with `orderId` and `symbol` parameters
  - Complete error handling for both HTTP and API-level errors

- **Bybit Perpetual Cancel Order** (`src/exchanges/bybit_perp/trading.rs`)
  - Uses `/v5/order/cancel` endpoint with proper V5 API signature  
  - Category set to "linear" for perpetual futures cancellations
  - Reuses authentication module from spot Bybit for consistency
  - Identical error handling patterns for maintainability

### Code Quality
- **Trait Completeness**: Both Bybit exchanges now fully implement the `OrderPlacer` trait
- **API Compliance**: Uses official Bybit V5 API endpoints and proper request formatting
- **Authentication**: Secure HMAC-SHA256 signature authentication for all cancel requests
- **Error Messages**: Clear, actionable error messages for debugging and monitoring

## PR-6

### Added
- **Complete Bybit Exchange Integration**: Full implementation of both Bybit Spot and Bybit Perpetual exchange support
  - **Bybit Spot Trading**: Complete spot trading functionality with market data, trading, and account management
  - **Bybit Perpetual Futures**: Full perpetual futures support including positions, margin, and leverage management
  - **V5 API Implementation**: Modern Bybit V5 API with unified account architecture
  - **Modular Architecture**: Following established patterns with dedicated modules for each feature area
  - **WebSocket Streaming**: Real-time market data for both spot and perpetual markets
  - **Complete Trading Suite**: Order placement, cancellation, and management for both spot and futures
  - **Account Management**: Balance checking, position monitoring, and portfolio management
  - **Testnet Support**: Full testnet integration for safe development and testing

### Technical Implementation
- **Bybit Spot** (`src/exchanges/bybit/`)
  - **Market Data**: Comprehensive instrument info, klines, and real-time WebSocket streaming
  - **Trading**: Full order lifecycle management with limit, market, and stop orders
  - **Account**: UNIFIED account balance retrieval and portfolio management
  - **Authentication**: Secure HMAC-SHA256 signature authentication with proper timestamping
  - **WebSocket**: Real-time ticker, orderbook, trades, and klines streaming

- **Bybit Perpetual** (`src/exchanges/bybit_perp/`)
  - **Futures Markets**: Linear perpetual contracts with comprehensive market information
  - **Position Management**: Long/short position tracking with PnL and liquidation prices
  - **Advanced Trading**: Perpetual-specific features including leverage and margin management
  - **Risk Management**: Position sizing, leverage control, and liquidation monitoring
  - **WebSocket Streams**: Real-time perpetual market data with orderbook depth and trade feeds

### Module Structure
```
exchanges/bybit/           # Spot Trading
├── client.rs             # Core client implementation
├── account.rs            # Account balance and portfolio
├── trading.rs            # Order placement and management
├── market_data.rs        # Market info and WebSocket streams
├── auth.rs               # HMAC authentication
├── converters.rs         # Data type conversions
├── types.rs              # Exchange-specific types
└── mod.rs                # Module exports

exchanges/bybit_perp/      # Perpetual Futures
├── client.rs             # Perpetual client implementation
├── account.rs            # Balance and position management
├── trading.rs            # Futures order management
├── market_data.rs        # Perpetual market data
├── converters.rs         # Futures-specific conversions
├── types.rs              # Perpetual futures types
└── mod.rs                # Module exports
```

### Features Implemented
- **Market Data Source Trait**: Complete implementation for both exchanges
- **Order Placer Trait**: Full trading functionality with proper order type support
- **Account Info Trait**: Balance and position retrieval for both spot and futures
- **WebSocket Integration**: Real-time data streaming with proper message parsing
- **Configuration Support**: Environment variable and .env file configuration
- **Error Handling**: Comprehensive error handling with proper API error mapping
- **Type Safety**: Strong typing for all API responses and requests

### Performance Characteristics
- **Bybit Spot**: ~2.1s for 641 markets, ~189ms average klines retrieval
- **Bybit Perpetual**: ~1.8s for 500 markets, ~201ms average klines retrieval
- **WebSocket**: <100ms connection times for both spot and perpetual streams
- **API Efficiency**: Optimized request patterns following Bybit best practices

### Example Integration
- **Complete Example**: `examples/bybit_example.rs` demonstrating both spot and perpetual usage
- **Configuration**: Proper .env setup with BYBIT_API_KEY and BYBIT_SECRET_KEY
- **Error Handling**: Comprehensive error handling patterns for production use
- **Multi-Exchange**: Seamless integration alongside existing Binance and Hyperliquid support

### Code Quality
- **Consistent Architecture**: Follows the established modular pattern used by other exchanges
- **Authentication Security**: Proper credential handling with memory-safe patterns
- **WebSocket Reliability**: Robust connection management with auto-reconnection support
- **Comprehensive Types**: Full type coverage for all Bybit API responses and requests

## PR-5

### Enhanced
- **Code Quality Improvements**: Comprehensive clippy fixes across the entire codebase
  - **Documentation**: Added backticks around type names in doc comments (`OrderBook` → `` `OrderBook` ``)
  - **String Creation**: Replaced manual `"".to_string()` with efficient `String::new()`
  - **Method Chaining**: Converted `map().unwrap_or()` patterns to more idiomatic `map_or()` 
  - **Branch Logic**: Simplified redundant conditional branches that shared identical code
  - **Type Casting**: Added safe casting with bounds checking for `u64` to `i64` conversions
  - **Function Length**: Added appropriate `#[allow(clippy::too_many_lines)]` for complex functions
  - **Performance Patterns**: Fixed `map().unwrap_or_else()` to use `map_or_else()` for better performance

### Code Quality Metrics
- **16 Clippy Warnings Resolved**: All quality issues eliminated across library and examples
- **Files Improved**: 
  - `src/exchanges/backpack/converters.rs` - Documentation and string creation fixes
  - `src/exchanges/backpack/trading.rs` - Map/unwrap patterns and error handling
  - `src/exchanges/backpack/market_data.rs` - Casting safety and function complexity
  - `examples/backpack_example.rs` - Import cleanup and function length
  - `examples/backpack_streams_example.rs` - Map/unwrap patterns and message handling
  - `examples/latency_test.rs` - Performance test casting warnings
- **Standards Compliance**: All code now passes `cargo clippy --all-targets --all-features -- -D warnings`
- **Maintainability**: Improved code readability and Rust idiom compliance

### Technical Improvements
- **TLS Stack**: More reliable cross-platform WebSocket connections using rustls
- **Error Handling**: Enhanced error messages and diagnostics for connection failures
- **Performance**: Optimized method chaining patterns for better runtime efficiency
- **Safety**: Improved type casting with overflow protection

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
