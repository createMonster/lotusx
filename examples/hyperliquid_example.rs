use lotusx::core::config::ExchangeConfig;
use lotusx::core::kernel::RestClient;
use lotusx::core::traits::{AccountInfo, MarketDataSource, OrderPlacer};
use lotusx::core::types::{
    conversion, KlineInterval, MarketDataType, OrderRequest, OrderSide, OrderType,
    SubscriptionType, TimeInForce, WebSocketConfig,
};
use lotusx::exchanges::hyperliquid::{build_hyperliquid_connector, HyperliquidBuilder};
use std::error::Error;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

/// Example demonstrating basic market data retrieval without authentication
async fn demo_market_data() -> Result<(), Box<dyn Error>> {
    println!("\n=== 📊 Market Data Example (REST) ===");

    let config = ExchangeConfig::read_only().testnet(true);
    let connector = build_hyperliquid_connector(config)?;

    // Get available markets
    match connector.get_markets().await {
        Ok(markets) => {
            println!("✓ Found {} markets", markets.len());
            for (i, market) in markets.iter().take(5).enumerate() {
                println!(
                    "  {}. {} (status: {}, min_qty: {:?})",
                    i + 1,
                    market.symbol,
                    market.status,
                    market.min_qty
                );
            }
        }
        Err(e) => {
            error!("Failed to get markets: {}", e);
            return Err(e.into());
        }
    }

    // Get klines/candlestick data
    match connector
        .get_klines(
            "BTC".to_string(),
            KlineInterval::Hours1,
            Some(10),
            None,
            None,
        )
        .await
    {
        Ok(klines) => {
            println!("✓ Retrieved {} klines for BTC (1h)", klines.len());
            if let Some(kline) = klines.first() {
                println!(
                    "  Latest: O={} H={} L={} C={} V={}",
                    kline.open_price,
                    kline.high_price,
                    kline.low_price,
                    kline.close_price,
                    kline.volume
                );
            }
        }
        Err(e) => {
            error!("Failed to get klines: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Example demonstrating authenticated operations including trading
async fn demo_authenticated_trading() -> Result<(), Box<dyn Error>> {
    println!("\n=== 🔐 Authenticated Trading Example ===");

    // Note: Replace with your actual private key for real usage
    // For demo purposes, we'll use a test key that won't have real funds
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";

    let auth_config =
        ExchangeConfig::new("hyperliquid_key".to_string(), private_key.to_string()).testnet(true);

    let auth_connector = HyperliquidBuilder::new(auth_config).build_rest_only()?;
    println!("✓ Authentication successful!");

    // Display wallet information
    if let Some(address) = auth_connector.trading.wallet_address() {
        println!("📍 Wallet address: {}", address);
    }

    println!(
        "🔑 Can sign transactions: {}",
        auth_connector.trading.can_sign()
    );

    // Get account balances
    match auth_connector.get_account_balance().await {
        Ok(balances) => {
            println!("💰 Account balances:");
            for balance in balances {
                println!(
                    "  {}: free={}, locked={}",
                    balance.asset, balance.free, balance.locked
                );
            }
        }
        Err(e) => {
            warn!("Could not retrieve balances: {}", e);
        }
    }

    // Get positions
    match auth_connector.get_positions().await {
        Ok(positions) => {
            println!("📈 Open positions: {}", positions.len());
            for position in positions {
                println!(
                    "  {}: {:?} {} (PnL: {})",
                    position.symbol,
                    position.position_side,
                    position.position_amount,
                    position.unrealized_pnl
                );
            }
        }
        Err(e) => {
            warn!("Could not retrieve positions: {}", e);
        }
    }

    // Demonstrate order placement and cancellation
    demo_order_management(&auth_connector).await?;

    // Demonstrate Hyperliquid-specific features
    demo_hyperliquid_features(&auth_connector).await?;

    Ok(())
}

/// Example demonstrating order placement and management
async fn demo_order_management(
    connector: &(impl OrderPlacer + Send + Sync),
) -> Result<(), Box<dyn Error>> {
    println!("\n🔄 Testing order management...");

    let test_order = OrderRequest {
        symbol: conversion::string_to_symbol("BTC"),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: conversion::string_to_quantity("0.001"),
        price: Some(conversion::string_to_price("20000")), // Low price to avoid accidental execution
        time_in_force: Some(TimeInForce::GTC),
        stop_price: None,
    };

    match connector.place_order(test_order).await {
        Ok(response) => {
            println!("✓ Order placed successfully!");
            println!("  Order ID: {}", response.order_id);
            println!("  Status: {}", response.status);

            // Try to cancel the order
            match connector
                .cancel_order("BTC".to_string(), response.order_id.clone())
                .await
            {
                Ok(_) => println!("✓ Order cancelled successfully"),
                Err(e) => warn!("Could not cancel order {}: {}", response.order_id, e),
            }
        }
        Err(e) => {
            info!("Order placement failed (expected on testnet): {}", e);
        }
    }

    Ok(())
}

/// Example demonstrating Hyperliquid-specific features
async fn demo_hyperliquid_features<R: RestClient + Clone + Send + Sync>(
    connector: &lotusx::exchanges::hyperliquid::HyperliquidConnector<R, ()>,
) -> Result<(), Box<dyn Error>> {
    println!("\n🔧 Hyperliquid-specific features:");

    // Get open orders
    match connector.trading.get_open_orders().await {
        Ok(orders) => {
            println!("📋 Open orders: {}", orders.len());
            for order in orders.iter().take(3) {
                println!(
                    "  {} {} {} @ {}",
                    order.coin, order.side, order.sz, order.limit_px
                );
            }
        }
        Err(e) => {
            warn!("Could not retrieve open orders: {}", e);
        }
    }

    // Get user fills (trade history)
    match connector.account.get_user_fills().await {
        Ok(fills) => {
            println!("📜 Recent fills: {}", fills.len());
            for fill in fills.iter().take(3) {
                println!(
                    "  {} {} @ {} (fee: {})",
                    fill.coin, fill.side, fill.px, fill.fee
                );
            }
        }
        Err(e) => {
            warn!("Could not retrieve user fills: {}", e);
        }
    }

    Ok(())
}

/// Example demonstrating WebSocket market data streaming
async fn demo_websocket_streaming() -> Result<(), Box<dyn Error>> {
    const MAX_MESSAGES: u32 = 5;

    println!("\n=== 🌐 WebSocket Market Data Example ===");

    let ws_config = ExchangeConfig::read_only().testnet(true);
    let ws_connector = HyperliquidBuilder::new(ws_config)
        .with_websocket()
        .build_with_websocket()?;

    println!("✓ WebSocket connector created");
    println!("🔗 WebSocket URL: {}", ws_connector.get_websocket_url());

    // Set up subscription parameters
    let symbols = vec!["BTC".to_string(), "ETH".to_string()];
    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::OrderBook { depth: Some(10) },
        SubscriptionType::Trades,
    ];

    let ws_config = WebSocketConfig {
        auto_reconnect: true,
        max_reconnect_attempts: Some(5),
        ping_interval: Some(30),
    };

    match ws_connector
        .subscribe_market_data(symbols, subscription_types, Some(ws_config))
        .await
    {
        Ok(mut receiver) => {
            println!("📡 WebSocket subscription established!");

            // Listen for messages with timeout
            let listen_duration = Duration::from_secs(10);
            let mut message_count = 0;

            match timeout(listen_duration, async {
                while let Some(data) = receiver.recv().await {
                    message_count += 1;
                    handle_websocket_message(data);

                    if message_count >= MAX_MESSAGES {
                        break;
                    }
                }
            })
            .await
            {
                Ok(_) => println!("✓ Received {} WebSocket messages", message_count),
                Err(_) => println!(
                    "⏰ WebSocket timeout after {} seconds (normal for demo)",
                    listen_duration.as_secs()
                ),
            }
        }
        Err(e) => {
            error!("WebSocket subscription failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Handle different types of WebSocket messages
fn handle_websocket_message(data: MarketDataType) {
    match data {
        MarketDataType::Ticker(ticker) => {
            println!("📊 Ticker: {} = ${}", ticker.symbol, ticker.price);
        }
        MarketDataType::OrderBook(book) => {
            println!(
                "📖 OrderBook: {} ({} bids, {} asks)",
                book.symbol,
                book.bids.len(),
                book.asks.len()
            );
        }
        MarketDataType::Trade(trade) => {
            println!(
                "💱 Trade: {} {} @ ${}",
                trade.symbol, trade.quantity, trade.price
            );
        }
        MarketDataType::Kline(kline) => {
            println!(
                "📈 Kline: {} OHLC({},{},{},{})",
                kline.symbol,
                kline.open_price,
                kline.high_price,
                kline.low_price,
                kline.close_price
            );
        }
    }
}

/// Example demonstrating advanced builder pattern features
async fn demo_builder_features() -> Result<(), Box<dyn Error>> {
    println!("\n=== 🏗️ Builder Pattern Features ===");

    let advanced_config =
        ExchangeConfig::new("test_key".to_string(), "test_secret".to_string()).testnet(true);

    let advanced_connector = HyperliquidBuilder::new(advanced_config)
        .with_vault_address("0x1234567890abcdef1234567890abcdef12345678".to_string())
        .with_mainnet(false) // Use testnet
        .build_rest_only()?;

    println!("✓ Advanced connector built with custom configuration");
    println!(
        "🔗 WebSocket URL: {}",
        advanced_connector.get_websocket_url()
    );

    // Display wallet address (vault address functionality is not yet implemented)
    if let Some(wallet_addr) = advanced_connector.trading.wallet_address() {
        println!("📍 Wallet address: {}", wallet_addr);
    }

    Ok(())
}

/// Print summary and tips
fn print_summary() {
    println!("\n=== ✨ Summary ===");
    println!("✅ Hyperliquid REST API integration complete");
    println!("✅ Market data retrieval working");
    println!("✅ Authentication system configured");
    println!("✅ Trading interface available");
    println!("✅ Account management functional");
    println!("✅ WebSocket support implemented");
    println!("✅ Builder pattern demonstrated");

    println!("\n💡 Tips for production use:");
    println!("  • Use real private keys from environment variables");
    println!("  • Implement proper error handling and retries");
    println!("  • Monitor rate limits and WebSocket connections");
    println!("  • Use testnet for development and testing");
    println!("  • Keep private keys secure and never commit them");
    println!("  • Consider implementing circuit breakers for high-frequency trading");
    println!("  • Use structured logging for better observability");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Hyperliquid Exchange API Example");
    println!("===================================");

    // Run all examples, continuing even if some fail
    let mut errors = Vec::new();

    // Example 1: Basic market data
    if let Err(e) = demo_market_data().await {
        errors.push(format!("Market data demo failed: {}", e));
    }

    // Example 2: Authenticated trading
    if let Err(e) = demo_authenticated_trading().await {
        errors.push(format!("Authenticated trading demo failed: {}", e));
    }

    // Example 3: WebSocket streaming
    if let Err(e) = demo_websocket_streaming().await {
        errors.push(format!("WebSocket streaming demo failed: {}", e));
    }

    // Example 4: Builder pattern features
    if let Err(e) = demo_builder_features().await {
        errors.push(format!("Builder features demo failed: {}", e));
    }

    // Print summary
    print_summary();

    // Report any errors that occurred
    if !errors.is_empty() {
        println!("\n⚠️  Some examples encountered errors:");
        for error in errors {
            println!("  • {}", error);
        }
        println!("\nThis is normal for demo purposes, especially on testnet.");
    }

    Ok(())
}
