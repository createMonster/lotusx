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

Each exchange follows a modular structure, but with flexibility based on requirements. Here are the patterns used by existing exchanges:

### Standard Structure (Most Exchanges)
```
src/exchanges/exchange_name/
├── mod.rs           # Module exports and re-exports
├── client.rs        # Main connector struct (lightweight)
├── types.rs         # Exchange-specific data structures
├── converters.rs    # Type conversions between exchange and core types
├── market_data.rs   # Market data implementation
├── trading.rs       # Order placement and management
└── account.rs       # Account information queries
```

### With Authentication Module
Some exchanges require their own authentication logic:
```
src/exchanges/exchange_name/
├── ... (standard files)
└── auth.rs          # Authentication and request signing
```

### With Custom WebSocket Implementation
Exchanges with complex WebSocket requirements may have:
```
src/exchanges/exchange_name/
├── ... (standard files)
└── websocket.rs     # Exchange-specific WebSocket handling
```

## 🔄 Current Exchange Examples

### Binance Pattern (Standard with Auth)
- `client.rs` - Lightweight connector
- `auth.rs` - HMAC-SHA256 authentication
- All standard modules present

### Binance Perpetual Pattern (Auth Reuse)
- `client.rs` - Lightweight connector
- No `auth.rs` - reuses authentication from binance
- All other standard modules present

### Hyperliquid Pattern (Custom WebSocket)
- `client.rs` - More complex due to EIP-712 authentication
- `auth.rs` - EIP-712 cryptographic signing
- `websocket.rs` - Custom WebSocket message handling
- All standard modules present

### Bybit Perpetual Pattern (Minimal)
- `client.rs` - Lightweight connector
- No `auth.rs` - reuses authentication from bybit spot
- All other standard modules present

## 🚀 Step-by-Step Implementation Approach

### Step 1: Plan Your Exchange Structure
Before writing code, determine:
- Does the exchange need custom authentication? (create `auth.rs`)
- Does it have complex WebSocket requirements? (create `websocket.rs`)
- Can you reuse authentication from another exchange?

### Step 2: Create the Exchange Directory
```bash
mkdir src/exchanges/exchange_name
```

### Step 3: Implement Core Modules (In Order)

#### Start with Foundation
1. **`types.rs`** - Define all exchange-specific data structures
2. **`client.rs`** - Create the main connector struct (keep it lightweight)
3. **`mod.rs`** - Set up module exports

#### Add Authentication (If Needed)
4. **`auth.rs`** - Implement authentication logic if exchange requires unique auth

#### Implement Core Functionality
5. **`converters.rs`** - Convert between exchange types and core types
6. **`market_data.rs`** - Implement market data retrieval and WebSocket subscriptions
7. **`trading.rs`** - Implement order placement and cancellation
8. **`account.rs`** - Implement account balance and position retrieval

#### Add Advanced Features (If Needed)
9. **`websocket.rs`** - Custom WebSocket handling for complex exchanges

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

Every exchange must implement these core traits (defined in `src/core/traits.rs`):

1. **`ExchangeConnector`** - Base connector trait
2. **`MarketDataSource`** - Market data retrieval and WebSocket subscriptions
3. **`OrderPlacer`** - Order placement and cancellation
4. **`AccountInfo`** - Account balance and position information

Optional traits (for specific exchange types):
- **`FundingRateSource`** - For perpetual exchanges with funding rates

## 🎨 Design Patterns

### Lightweight Client Pattern
The `client.rs` file should be minimal, containing only:
- The main connector struct
- Basic configuration and setup
- Constructor methods

All functionality is implemented in separate modules.

### Trait-Based Implementation
Each module implements specific traits:
- `market_data.rs` implements `MarketDataSource`
- `trading.rs` implements `OrderPlacer`  
- `account.rs` implements `AccountInfo`

### Converter Pattern
The `converters.rs` module handles all data transformations:
- Exchange format → Core format
- Core format → Exchange format
- Type safety and validation

### Authentication Reuse
Exchanges from the same provider can share authentication:
- `binance_perp` reuses `binance` auth
- `bybit_perp` reuses `bybit` auth

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
    exchanges::exchange_name::ExchangeNameConnector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ExchangeConfig::from_env("EXCHANGE_NAME")?;
    let connector = ExchangeNameConnector::new(config);

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