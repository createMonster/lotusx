# Integration Testing Guide

This guide explains how to run and configure integration tests for the LotusTX trading system.

## Overview

The integration tests verify that all exchange implementations work correctly with their respective APIs. Tests are designed to be safe and use testnet environments whenever possible.

## Test Structure

### Test Categories

1. **Basic Functionality Tests** - No credentials required
   - WebSocket URL validation
   - Market data structure verification
   - Error handling validation

2. **Live API Tests** - Requires valid credentials
   - Account balance retrieval
   - Position queries
   - Market data fetching

3. **Performance Tests** - Tests concurrent requests and response times

4. **Comparison Tests** - Validates consistency across exchanges

## Quick Start

### Running Basic Tests

```bash
# Run all basic tests (safe, no credentials needed)
cargo test --test simple_integration_tests

# Run with output
cargo test --test simple_integration_tests -- --nocapture
```

### Running with Live APIs

```bash
# Set up testnet credentials
export BYBIT_TESTNET_API_KEY="your_api_key"
export BYBIT_TESTNET_SECRET_KEY="your_secret_key"
export BINANCE_TESTNET_API_KEY="your_api_key"
export BINANCE_TESTNET_SECRET_KEY="your_secret_key"

# Enable live tests
export RUN_LIVE_TESTS=true

# Run live API tests
cargo test --test simple_integration_tests -- --ignored --nocapture
```

### Using the Test Runner Script

```bash
# Make script executable
chmod +x scripts/run_integration_tests.sh

# Run comprehensive tests
./scripts/run_integration_tests.sh
```

## Environment Variables

### Test Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `RUN_INTEGRATION_TESTS` | `false` | Enable integration tests |
| `RUN_LIVE_TESTS` | `false` | Enable tests with real APIs |
| `RUN_ORDER_TESTS` | `false` | Enable order placement tests (⚠️ CAREFUL) |
| `TEST_TIMEOUT_SECONDS` | `30` | Timeout for API requests |

### Exchange Credentials

#### Bybit
- `BYBIT_API_KEY` / `BYBIT_SECRET_KEY` - Production
- `BYBIT_TESTNET_API_KEY` / `BYBIT_TESTNET_SECRET_KEY` - Testnet (recommended)

#### Binance
- `BINANCE_API_KEY` / `BINANCE_SECRET_KEY` - Production
- `BINANCE_TESTNET_API_KEY` / `BINANCE_TESTNET_SECRET_KEY` - Testnet (recommended)

#### Perpetual Futures
- `BYBIT_PERP_TESTNET_API_KEY` / `BYBIT_PERP_TESTNET_SECRET_KEY`
- `BINANCE_PERP_TESTNET_API_KEY` / `BINANCE_PERP_TESTNET_SECRET_KEY`

## Setting Up API Credentials

### Bybit Testnet

1. Go to [Bybit Testnet](https://testnet.bybit.com/)
2. Create an account and verify email
3. Navigate to API Management
4. Create new API key with these permissions:
   - Read-only (for account info)
   - Derivatives (for perpetual trading)
   - Spot trading (for spot trading)
5. Save your API key and secret

### Binance Testnet

1. Go to [Binance Testnet](https://testnet.binance.vision/)
2. Create an account
3. Generate API credentials
4. Set appropriate permissions

### Environment Setup

Create a `.env` file (never commit this!):

```bash
# Bybit Testnet
BYBIT_TESTNET_API_KEY=your_bybit_testnet_api_key
BYBIT_TESTNET_SECRET_KEY=your_bybit_testnet_secret_key

# Binance Testnet  
BINANCE_TESTNET_API_KEY=your_binance_testnet_api_key
BINANCE_TESTNET_SECRET_KEY=your_binance_testnet_secret_key

# Test configuration
RUN_LIVE_TESTS=true
TEST_TIMEOUT_SECONDS=30
```

Load environment:
```bash
source .env
# or
export $(cat .env | xargs)
```

## Test Coverage

### Bybit Integration Tests

#### Spot Trading
- ✅ Market data retrieval
- ✅ WebSocket URL validation
- ✅ Account balance queries
- ✅ Error handling

#### Perpetual Futures
- ✅ Market data retrieval  
- ✅ WebSocket URL validation
- ✅ Position queries
- ✅ Error handling

### Binance Integration Tests

#### Spot Trading
- ✅ Market data retrieval
- ✅ WebSocket URL validation
- ✅ Kline/candlestick data
- ✅ Account balance queries
- ✅ Error handling

#### Perpetual Futures
- ✅ Market data retrieval
- ✅ WebSocket URL validation
- ✅ Position queries
- ✅ Error handling

## Understanding Test Output

### Success Indicators
- ✅ Green checkmarks indicate successful tests
- Numbers show market counts, kline data, etc.
- Timing information for performance validation

### Warning Indicators
- ⚠️ Yellow warnings for non-critical issues
- API rate limits or temporary failures
- Missing credentials (expected for basic tests)

### Error Indicators
- ❌ Red errors for test failures
- Authentication issues
- Network connectivity problems
- API endpoint changes

## Continuous Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      
    - name: Run basic integration tests
      run: cargo test --test simple_integration_tests
      
    - name: Run live API tests
      if: github.event_name == 'push'
      env:
        RUN_LIVE_TESTS: true
        BYBIT_TESTNET_API_KEY: ${{ secrets.BYBIT_TESTNET_API_KEY }}
        BYBIT_TESTNET_SECRET_KEY: ${{ secrets.BYBIT_TESTNET_SECRET_KEY }}
      run: cargo test --test simple_integration_tests -- --ignored
```

## Safety Guidelines

### ⚠️ Important Safety Notes

1. **Always use testnet credentials** for automated testing
2. **Never commit API keys** to version control
3. **Monitor rate limits** to avoid API bans
4. **Use minimal quantities** if testing order placement
5. **Set up alerts** for unexpected API usage

### Order Placement Tests (Extra Caution)

Order placement tests are disabled by default. To enable:

```bash
export RUN_ORDER_TESTS=true
```

**⚠️ WARNING**: These tests place real orders (though on testnet). They use:
- Very small quantities (0.001)
- Very low prices (to avoid execution)
- Immediate cancellation

Monitor your testnet accounts and ensure you understand the risks.

## Troubleshooting

### Common Issues

#### Authentication Errors
```
❌ Bybit Spot balance failed: API signature error
```
**Solution**: Verify API key and secret are correct, check permissions

#### Rate Limiting
```
⚠️ Request failed: Too many requests
```
**Solution**: Reduce test frequency, add delays between requests

#### Network Timeouts
```
⚠️ Binance Spot markets timed out
```
**Solution**: Increase `TEST_TIMEOUT_SECONDS`, check network connectivity

#### Missing Dependencies
```
failed to resolve: use of unresolved module `futures`
```
**Solution**: Run `cargo build` to ensure all dependencies are installed

### Debug Mode

Enable verbose logging:
```bash
RUST_LOG=debug cargo test --test simple_integration_tests -- --nocapture
```

### Isolated Testing

Test specific exchanges:
```bash
# Test only Bybit
cargo test --test simple_integration_tests bybit_tests:: -- --nocapture

# Test only Binance  
cargo test --test simple_integration_tests binance_tests:: -- --nocapture
```

## Best Practices

### Development Workflow

1. **Start with basic tests** during development
2. **Use testnet credentials** for integration testing
3. **Run performance tests** before releases
4. **Monitor API changes** and update tests accordingly
5. **Document any test failures** and their resolutions

### Production Readiness

Before deploying to production:

1. ✅ All basic tests pass
2. ✅ Live API tests pass with testnet
3. ✅ Performance tests meet requirements
4. ✅ Error handling works correctly
5. ✅ Rate limiting is respected
6. ✅ WebSocket connections are stable

### Maintenance

Regular maintenance tasks:

- Update API endpoints as exchanges evolve
- Refresh testnet credentials periodically
- Monitor test execution times
- Review and update test coverage
- Check for new exchange features to test

## Contributing

When adding new exchanges or features:

1. **Follow the existing test patterns**
2. **Add both basic and live API tests**
3. **Include error handling tests**
4. **Update this documentation**
5. **Test with multiple scenarios**

### Test Template

Use this template for new exchange tests:

```rust
#[tokio::test]
async fn test_new_exchange_markets() {
    let connector = NewExchangeConnector::new(create_test_config());
    
    let result = timeout(Duration::from_secs(30), connector.get_markets()).await;
    
    match result {
        Ok(Ok(markets)) => {
            println!("✅ New Exchange: Fetched {} markets", markets.len());
            assert!(!markets.is_empty(), "Should have markets");
            // Add specific validations
        }
        Ok(Err(e)) => {
            println!("⚠️ New Exchange markets failed: {}", e);
        }
        Err(_) => {
            println!("⚠️ New Exchange markets timed out");
        }
    }
}
```

## Support

For issues with integration tests:

1. Check this documentation first
2. Review test output for specific error messages
3. Verify API credentials and permissions
4. Check exchange API status pages
5. Open an issue with full error logs

Remember: Integration tests are a safety net, not a replacement for careful development and manual testing. 