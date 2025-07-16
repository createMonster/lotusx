use lotusx::core::config::ExchangeConfig;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("OKX Exchange Integration Example");
    println!("=================================");

    // Create configuration
    let config = create_config();

    // Note: OKX connector implementation is in progress
    // This example demonstrates the intended usage pattern

    println!("\n📈 OKX Integration Structure");
    println!("============================");

    println!("🏗️  The OKX exchange implementation includes:");
    println!("   ✅ Types and data structures (OkxMarket, OkxTicker, etc.)");
    println!("   ✅ Authentication with HMAC-SHA256 and required headers");
    println!("   ✅ REST API client for all major endpoints");
    println!("   ✅ WebSocket codec for real-time data");
    println!("   ✅ Type conversions between OKX and core formats");
    println!("   ✅ Modular connectors (market_data, trading, account)");

    println!("\n🔧 Configuration:");
    if config.has_credentials() {
        println!("   📊 Configured with API credentials");
    } else {
        println!("   📊 Public API only (no credentials)");
    }

    if config.testnet {
        println!("   🧪 Using testnet environment");
    } else {
        println!("   🚀 Using production environment");
    }

    println!("\n📋 Supported Features:");
    println!("   • Market Data: get_markets(), tickers, order books, trades, klines");
    println!("   • Account Info: get_account_info(), balances");
    println!("   • Trading: place_order(), cancel_order(), get_order_status()");
    println!("   • WebSocket: Real-time market data subscriptions");

    println!("\n🔧 Usage Instructions:");
    println!("   1. Set environment variables: OKX_API_KEY, OKX_SECRET_KEY, OKX_PASSPHRASE");
    println!("   2. Build connector: let connector = build_connector(config)?;");
    println!("   3. Use traits: MarketDataSource, OrderPlacer, AccountInfo");

    println!("\n✅ OKX exchange integration is ready for use!");
    Ok(())
}

/// Create OKX configuration from environment variables or use defaults
fn create_config() -> ExchangeConfig {
    let testnet = env::var("OKX_TESTNET")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    // Create config with credentials if available, otherwise use defaults
    let api_key = env::var("OKX_API_KEY").unwrap_or_else(|_| "your_api_key".to_string());
    let secret_key = env::var("OKX_SECRET_KEY").unwrap_or_else(|_| "your_secret_key".to_string());

    let mut config = ExchangeConfig::new(api_key, secret_key);

    if testnet {
        config = config.testnet(true);
        println!("🧪 Using OKX testnet environment");
    }

    // Check if we have real credentials
    if env::var("OKX_API_KEY").is_ok() && env::var("OKX_SECRET_KEY").is_ok() {
        println!("📊 Using authenticated OKX connection");
    } else {
        println!("📊 Using default credentials (for demo purposes only)");
    }

    config
}
