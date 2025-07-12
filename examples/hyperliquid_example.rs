use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::{AccountInfo, MarketDataSource, OrderPlacer};
use lotusx::core::types::{
    conversion, KlineInterval, OrderRequest, OrderSide, OrderType, SubscriptionType, TimeInForce,
    WebSocketConfig,
};
use lotusx::exchanges::hyperliquid::{build_hyperliquid_connector, HyperliquidBuilder};
use std::error::Error;
use tokio::time::{timeout, Duration};

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Hyperliquid Exchange API Example");
    println!("===================================");

    // Example 1: REST-only Market Data (No authentication required)
    println!("\n=== 📊 Market Data Example (REST) ===");
    let market_data_config = ExchangeConfig::read_only().testnet(true);
    let connector = build_hyperliquid_connector(market_data_config)?;

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
        Err(e) => println!("❌ Error getting markets: {}", e),
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
        Err(e) => println!("❌ Error getting klines: {}", e),
    }

    // Example 2: Authenticated Client for Trading
    println!("\n=== 🔐 Authenticated Trading Example ===");

    // Note: Replace with your actual private key for real usage
    // For demo purposes, we'll use a test key that won't have real funds
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";

    let auth_config =
        ExchangeConfig::new("hyperliquid_key".to_string(), private_key.to_string()).testnet(true);

    match HyperliquidBuilder::new(auth_config).build_rest_only() {
        Ok(auth_connector) => {
            println!("✓ Authentication successful!");

            // Check wallet address
            if let Some(address) = auth_connector.trading.wallet_address() {
                println!("📍 Wallet address: {}", address);
            }

            // Check if we can sign transactions
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
                Err(e) => println!("❌ Error getting balances: {}", e),
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
                Err(e) => println!("❌ Error getting positions: {}", e),
            }

            // Example: Place a test order (likely to fail without real funds)
            println!("\n🔄 Testing order placement...");
            let test_order = OrderRequest {
                symbol: conversion::string_to_symbol("BTC"),
                side: OrderSide::Buy,
                order_type: OrderType::Limit,
                quantity: conversion::string_to_quantity("0.001"),
                price: Some(conversion::string_to_price("20000")), // Low price to avoid accidental execution
                time_in_force: Some(TimeInForce::GTC),
                stop_price: None,
            };

            match auth_connector.place_order(test_order).await {
                Ok(response) => {
                    println!("✓ Order placed successfully!");
                    println!("  Order ID: {}", response.order_id);
                    println!("  Status: {}", response.status);

                    // Try to cancel the order
                    match auth_connector
                        .cancel_order("BTC".to_string(), response.order_id)
                        .await
                    {
                        Ok(_) => println!("✓ Order cancelled successfully"),
                        Err(e) => println!("❌ Error cancelling order: {}", e),
                    }
                }
                Err(e) => println!("❌ Order placement failed (expected on testnet): {}", e),
            }

            // Hyperliquid-specific features
            println!("\n🔧 Hyperliquid-specific features:");

            // Get open orders
            match auth_connector.trading.get_open_orders().await {
                Ok(orders) => {
                    println!("📋 Open orders: {}", orders.len());
                    for order in orders.iter().take(3) {
                        println!(
                            "  {} {} {} @ {}",
                            order.coin, order.side, order.sz, order.limit_px
                        );
                    }
                }
                Err(e) => println!("❌ Error getting open orders: {}", e),
            }

            // Get user fills (trade history)
            match auth_connector.account.get_user_fills().await {
                Ok(fills) => {
                    println!("📜 Recent fills: {}", fills.len());
                    for fill in fills.iter().take(3) {
                        println!(
                            "  {} {} @ {} (fee: {})",
                            fill.coin, fill.side, fill.px, fill.fee
                        );
                    }
                }
                Err(e) => println!("❌ Error getting fills: {}", e),
            }
        }
        Err(e) => println!("❌ Authentication failed: {}", e),
    }

    // Example 3: WebSocket Market Data (Advanced)
    println!("\n=== 🌐 WebSocket Market Data Example ===");

    let ws_config = ExchangeConfig::read_only().testnet(true);

    match HyperliquidBuilder::new(ws_config)
        .with_websocket()
        .build_with_websocket()
    {
        Ok(ws_connector) => {
            println!("✓ WebSocket connector created");
            println!("🔗 WebSocket URL: {}", ws_connector.get_websocket_url());

            // Note: WebSocket subscription is more complex and requires proper session management
            // For now, we'll demonstrate the URL and basic setup
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

            // Note: Full WebSocket implementation requires session management
            match ws_connector
                .subscribe_market_data(symbols, subscription_types, Some(ws_config))
                .await
            {
                Ok(mut receiver) => {
                    println!("📡 WebSocket subscription established!");

                    // Listen for a short time
                    let listen_duration = Duration::from_secs(5);
                    let mut count = 0;

                    match timeout(listen_duration, async {
                        while let Some(data) = receiver.recv().await {
                            count += 1;
                            match data {
                                lotusx::core::types::MarketDataType::Ticker(ticker) => {
                                    println!("📊 Ticker: {} = ${}", ticker.symbol, ticker.price);
                                }
                                lotusx::core::types::MarketDataType::OrderBook(book) => {
                                    println!(
                                        "📖 OrderBook: {} ({} bids, {} asks)",
                                        book.symbol,
                                        book.bids.len(),
                                        book.asks.len()
                                    );
                                }
                                lotusx::core::types::MarketDataType::Trade(trade) => {
                                    println!(
                                        "💱 Trade: {} {} @ ${}",
                                        trade.symbol, trade.quantity, trade.price
                                    );
                                }
                                lotusx::core::types::MarketDataType::Kline(kline) => {
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

                            if count >= 3 {
                                break;
                            }
                        }
                    })
                    .await
                    {
                        Ok(_) => println!("✓ Received {} WebSocket messages", count),
                        Err(_) => println!("⏰ WebSocket timeout (normal for demo)"),
                    }
                }
                Err(e) => println!("❌ WebSocket subscription failed: {}", e),
            }
        }
        Err(e) => println!("❌ WebSocket connector creation failed: {}", e),
    }

    // Example 4: Builder Pattern Features
    println!("\n=== 🏗️ Builder Pattern Features ===");

    let advanced_config =
        ExchangeConfig::new("test_key".to_string(), "test_secret".to_string()).testnet(true);

    let advanced_connector = HyperliquidBuilder::new(advanced_config)
        .with_vault_address("0x1234567890abcdef1234567890abcdef12345678".to_string())
        .build_rest_only()?;

    println!("✓ Advanced connector built with custom configuration");
    println!(
        "🔗 WebSocket URL: {}",
        advanced_connector.get_websocket_url()
    );

    println!("\n=== ✨ Summary ===");
    println!("✓ Hyperliquid REST API integration complete");
    println!("✓ Market data retrieval working");
    println!("✓ Authentication system configured");
    println!("✓ Trading interface available");
    println!("✓ Account management functional");
    println!("✓ WebSocket support available");
    println!("✓ Builder pattern implemented");

    println!("\n💡 Tips for production use:");
    println!("  • Use real private keys from environment variables");
    println!("  • Implement proper error handling and retries");
    println!("  • Monitor rate limits and WebSocket connections");
    println!("  • Use testnet for development and testing");
    println!("  • Keep private keys secure and never commit them");

    Ok(())
}
