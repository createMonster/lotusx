# Adding New Exchange Guide

## Overview

This guide provides a step-by-step walkthrough for adding a new cryptocurrency exchange to the LotusX trading system. The guide focuses on understanding the project structure and following established patterns used by existing exchanges.

## 🎯 Key Principles

1. **Consistency**: Follow the established patterns used by existing exchanges
2. **Modularity**: Each exchange is self-contained with clear, focused modules
3. **Flexibility**: Adapt the structure based on exchange-specific requirements
4. **Code Reuse**: Reuse authentication and utility modules where possible

## 📁 Current Project Structure

The LotusX project follows this overall structure:

```
src/
├── core/                    # Core system components
│   ├── config.rs           # Configuration management
│   ├── errors.rs           # Error types and handling
│   ├── traits.rs           # Core traits (interfaces)
│   ├── types.rs            # Common data types
│   ├── websocket.rs        # WebSocket infrastructure
│   └── mod.rs              # Module exports
├── exchanges/               # Exchange implementations
│   ├── exchange_name/      # Each exchange has its own directory
│   └── mod.rs              # Exchange registry
├── utils/                   # Utility modules
│   ├── exchange_factory.rs # Factory for creating exchange instances
│   ├── latency_testing.rs  # Performance testing utilities
│   └── mod.rs              # Utility exports
├── lib.rs                  # Library entry point
└── main.rs                 # Binary entry point
```

## 🏗️ Exchange Module Structure

Each exchange follows the new **Kernel Architecture** with unified transport layer and modular design. The structure is consistent across all exchanges:

### Standard Kernel Structure (All Exchanges)
```
src/exchanges/exchange_name/
├── mod.rs           # Module exports and builder
├── builder.rs       # Exchange builder pattern implementation
├── codec.rs         # Message encoding/decoding
├── conversions.rs   # Type conversions between exchange and core types
├── connector/       # Modular connector implementations
│   ├── mod.rs       # Connector composition
│   ├── account.rs   # Account information queries
│   ├── market_data.rs # Market data implementation
│   └── trading.rs   # Order placement and management
├── rest.rs          # REST API client implementation
├── signer.rs        # Authentication and request signing
└── types.rs         # Exchange-specific data structures
```

### Kernel Integration Benefits
- **Unified Transport**: Leverages `src/core/kernel/` for HTTP and WebSocket communication
- **Consistent Authentication**: Uses `Signer` trait for secure credential handling
- **Modular Design**: Clean separation of concerns with focused modules
- **Builder Pattern**: Consistent instantiation across all exchanges

## 🔄 Current Exchange Examples

### Binance Pattern (Standard Kernel)
- `builder.rs` - Exchange builder implementing `BinanceBuilder`
- `signer.rs` - HMAC-SHA256 authentication via `Signer` trait
- `connector/` - Modular trait implementations
- All standard kernel modules present

### Binance Perpetual Pattern (Auth Reuse)
- `builder.rs` - Exchange builder implementing `BinancePerpBuilder`
- `signer.rs` - Reuses binance authentication module
- `connector/` - Modular trait implementations
- All other standard kernel modules present

### Hyperliquid Pattern (Custom Codec)
- `builder.rs` - Exchange builder implementing `HyperliquidBuilder`
- `signer.rs` - EIP-712 cryptographic signing
- `codec.rs` - Custom WebSocket message handling
- `connector/` - Modular trait implementations
- All standard kernel modules present

### Bybit Perpetual Pattern (Minimal Auth)
- `builder.rs` - Exchange builder implementing `BybitPerpBuilder`
- `signer.rs` - Reuses bybit spot authentication
- `connector/` - Modular trait implementations
- All other standard kernel modules present

## 🚀 Step-by-Step Implementation Approach

### Step 1: Plan Your Exchange Structure
Before writing code, determine:
- Does the exchange need custom WebSocket message handling? (enhance `codec.rs`)
- Can you reuse authentication from another exchange? (reuse `signer.rs`)
- What are the exchange's specific API endpoints and authentication requirements?

### Step 2: Create the Exchange Directory
```bash
mkdir src/exchanges/exchange_name
mkdir src/exchanges/exchange_name/connector
```

### Step 3: Implement Core Modules (In Order)

#### Start with Foundation
1. **`types.rs`** - Define all exchange-specific data structures
2. **`builder.rs`** - Create the exchange builder implementing build pattern
3. **`mod.rs`** - Set up module exports and builder

#### Add Transport Layer
4. **`rest.rs`** - Implement `RestClient` trait for HTTP communication
5. **`signer.rs`** - Implement `Signer` trait for authentication
6. **`codec.rs`** - Implement `Codec` trait for message encoding/decoding

#### Implement Core Functionality
7. **`conversions.rs`** - Convert between exchange types and core types
8. **`connector/mod.rs`** - Set up connector composition
9. **`connector/market_data.rs`** - Implement `MarketDataSource` trait
10. **`connector/trading.rs`** - Implement `OrderPlacer` trait
11. **`connector/account.rs`** - Implement `AccountInfo` trait

### Step 4: Register Your Exchange
Add your exchange to `src/exchanges/mod.rs`:
```rust
pub mod exchange_name;
```

### Step 5: Update Utilities (Optional)
Consider adding your exchange to:
- `src/utils/exchange_factory.rs` - For factory pattern creation
- `src/utils/latency_testing.rs` - For performance testing

## 📋 Core Traits to Implement

Every exchange must implement these core traits using the kernel architecture:

### Kernel Layer Traits (in `src/core/kernel/`)
1. **`RestClient`** - HTTP client abstraction for API communication
2. **`Signer`** - Authentication and request signing
3. **`Codec`** - Message encoding/decoding for WebSocket communication

### Exchange Layer Traits (in `src/core/traits.rs`)
1. **`ExchangeConnector`** - Base connector trait
2. **`MarketDataSource`** - Market data retrieval and WebSocket subscriptions
3. **`OrderPlacer`** - Order placement and cancellation
4. **`AccountInfo`** - Account balance and position information

### Optional traits (for specific exchange types)
- **`FundingRateSource`** - For perpetual exchanges with funding rates

### Builder Pattern
- **`ExchangeBuilder`** - Consistent builder pattern for exchange instantiation

## 🎨 Design Patterns

### Builder Pattern
The `builder.rs` file implements the builder pattern:
- Exchange builder struct (e.g., `BinanceBuilder`)
- `build()` method returning configured connector
- Configuration validation and setup

### Kernel Integration Pattern
Each exchange leverages the kernel layer:
- `rest.rs` implements `RestClient` for HTTP communication
- `signer.rs` implements `Signer` for authentication
- `codec.rs` implements `Codec` for WebSocket message handling

### Connector Composition Pattern
The `connector/` directory contains focused implementations:
- `market_data.rs` implements `MarketDataSource`
- `trading.rs` implements `OrderPlacer`  
- `account.rs` implements `AccountInfo`
- `mod.rs` composes all connectors into final exchange connector

### Converter Pattern
The `conversions.rs` module handles all data transformations:
- Exchange format → Core format
- Core format → Exchange format
- Type safety and validation

### Authentication Reuse
Exchanges from the same provider can share authentication:
- `binance_perp` reuses `binance` signer
- `bybit_perp` reuses `bybit` signer

## 🔧 Development Tips

### Start Simple
1. Begin with basic market data (`get_markets`)
2. Add authentication and account info
3. Implement trading functionality
4. Add WebSocket support last

### Follow Existing Patterns
- Look at similar exchanges for guidance
- Copy and modify rather than building from scratch
- Maintain consistency with existing code style

### Test Incrementally
- Test each module as you build it
- Use testnet/sandbox environments first
- Create simple examples to verify functionality

## 📝 Example Structure Implementation

Create a basic example file in `examples/exchange_name_example.rs`:

```rust
// Basic example showing your exchange in action
use lotusx::{
    core::{config::ExchangeConfig, traits::MarketDataSource},
    exchanges::exchange_name::ExchangeNameBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExchangeConfig::from_env("EXCHANGE_NAME")?;
    let connector = ExchangeNameBuilder::new().build(config).await?;

    // Test basic functionality
    let markets = connector.get_markets().await?;
    println!("Found {} markets", markets.len());

    Ok(())
}
```

## ✅ Implementation Checklist

Before considering your exchange complete:

### Structure
- [ ] Exchange directory created under `src/exchanges/`
- [ ] All required modules implemented
- [ ] Exchange registered in `src/exchanges/mod.rs`

### Core Functionality
- [ ] All required traits implemented
- [ ] Basic market data working
- [ ] Authentication working (if required)
- [ ] Trading functionality working
- [ ] Account queries working

### Integration
- [ ] Example file created
- [ ] Configuration working
- [ ] Error handling implemented
- [ ] Code compiles and passes basic tests

### Quality
- [ ] Code follows project patterns
- [ ] Modules are focused and cohesive
- [ ] No unnecessary duplication
- [ ] Documentation is clear

## 🎯 Focus on Structure, Not Implementation Details

This guide emphasizes the structural patterns rather than specific implementation details. Each exchange will have unique API requirements, but following the established structural patterns ensures:

- **Consistency** across all exchange implementations
- **Maintainability** through familiar code organization
- **Extensibility** for future enhancements
- **Testability** through modular design

Remember: The goal is to fit your exchange into the existing patterns, not to reinvent the architecture. Start with the simplest possible implementation and gradually add complexity as needed. 