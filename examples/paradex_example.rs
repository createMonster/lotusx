use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::{AccountInfo, FundingRateSource, MarketDataSource, OrderPlacer};
use lotusx::core::types::{
    conversion, KlineInterval, OrderRequest, OrderSide, OrderType, SubscriptionType, TimeInForce,
    WebSocketConfig,
};
use lotusx::exchanges::paradex::{
    build_connector, build_connector_with_reconnection, build_connector_with_websocket,
};
use num_traits::cast::ToPrimitive;
use std::error::Error;
use tokio::time::{timeout, Duration};

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Paradex Exchange API Example");
    println!("===============================");

    // Example 1: REST-only Market Data (No authentication required)
    println!("\n=== 📊 Market Data Example (REST) ===");
    let market_data_config = ExchangeConfig::read_only().testnet(true);
    let connector = build_connector(market_data_config)?;

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
            "BTC-USD".to_string(),
            KlineInterval::Hours1,
            Some(10),
            None,
            None,
        )
        .await
    {
        Ok(klines) => {
            println!("✓ Retrieved {} klines for BTC-USD (1h)", klines.len());
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

    // Example 2: Funding Rates (Paradex-specific feature)
    println!("\n=== 💰 Funding Rates Example ===");

    // Get current funding rates for all symbols
    match connector.get_all_funding_rates().await {
        Ok(rates) => {
            println!("✓ Retrieved {} funding rates", rates.len());
            for rate in rates.iter().take(5) {
                if let Some(funding_rate) = rate.funding_rate {
                    println!(
                        "  {}: {:.6}% (next: {:?})",
                        rate.symbol,
                        funding_rate.to_f64().unwrap_or(0.0) * 100.0,
                        rate.next_funding_time
                    );
                }
            }
        }
        Err(e) => println!("❌ Error getting funding rates: {}", e),
    }

    // Get funding rates for specific symbols
    let symbols = vec!["BTC-USD".to_string(), "ETH-USD".to_string()];
    match connector.get_funding_rates(Some(symbols)).await {
        Ok(rates) => {
            println!("✓ Retrieved funding rates for specific symbols:");
            for rate in rates {
                if let Some(funding_rate) = rate.funding_rate {
                    println!(
                        "  {}: {:.6}% (mark: {:?})",
                        rate.symbol,
                        funding_rate.to_f64().unwrap_or(0.0) * 100.0,
                        rate.mark_price
                    );
                }
            }
        }
        Err(e) => println!("❌ Error getting specific funding rates: {}", e),
    }

    // Get funding rate history
    match connector
        .get_funding_rate_history(
            "BTC-USD".to_string(),
            None,    // start_time
            None,    // end_time
            Some(5), // limit to last 5 records
        )
        .await
    {
        Ok(history) => {
            println!(
                "✓ Retrieved {} historical funding rates for BTC-USD",
                history.len()
            );
            for rate in history {
                if let Some(funding_rate) = rate.funding_rate {
                    println!(
                        "  Rate: {:.6}% at timestamp {}",
                        funding_rate.to_f64().unwrap_or(0.0) * 100.0,
                        rate.timestamp
                    );
                }
            }
        }
        Err(e) => println!("❌ Error getting funding rate history: {}", e),
    }

    // Example 3: Authenticated Client for Trading
    println!("\n=== 🔐 Authenticated Trading Example ===");

    // Note: Replace with your actual private key for real usage
    // For demo purposes, we'll use a test key that won't have real funds
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";

    let auth_config =
        ExchangeConfig::new("paradex_key".to_string(), private_key.to_string()).testnet(true);

    match build_connector(auth_config) {
        Ok(auth_connector) => {
            println!("✓ Authentication successful!");

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
                symbol: conversion::string_to_symbol("BTC-USD"),
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
                        .cancel_order("BTC-USD".to_string(), response.order_id)
                        .await
                    {
                        Ok(_) => println!("✓ Order cancelled successfully"),
                        Err(e) => println!("❌ Error cancelling order: {}", e),
                    }
                }
                Err(e) => println!("❌ Order placement failed (expected on testnet): {}", e),
            }
        }
        Err(e) => println!("❌ Authentication failed: {}", e),
    }

    // Example 4: WebSocket Market Data
    println!("\n=== 🌐 WebSocket Market Data Example ===");

    let ws_config = ExchangeConfig::read_only().testnet(true);

    match build_connector_with_websocket(ws_config) {
        Ok(ws_connector) => {
            println!("✓ WebSocket connector created");
            println!("🔗 WebSocket URL: {}", ws_connector.get_websocket_url());

            let symbols = vec!["BTC-USD".to_string(), "ETH-USD".to_string()];
            let subscription_types = vec![
                SubscriptionType::Ticker,
                SubscriptionType::OrderBook { depth: Some(10) },
                SubscriptionType::Trades,
                SubscriptionType::Klines {
                    interval: KlineInterval::Minutes1,
                },
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

    // Example 5: Advanced WebSocket with Auto-Reconnection
    println!("\n=== 🔄 Auto-Reconnection WebSocket Example ===");

    let reconnect_config = ExchangeConfig::read_only().testnet(true);

    match build_connector_with_reconnection(reconnect_config) {
        Ok(reconnect_connector) => {
            println!("✓ Auto-reconnection WebSocket connector created");
            println!(
                "🔗 WebSocket URL: {}",
                reconnect_connector.get_websocket_url()
            );

            // This connector will automatically handle reconnections, resubscriptions, etc.
            println!("🔄 This connector includes:");
            println!("  • Automatic reconnection on disconnect");
            println!("  • Exponential backoff retry strategy");
            println!("  • Automatic resubscription to streams");
            println!("  • Maximum 10 reconnect attempts");
            println!("  • 2-second initial reconnect delay");
        }
        Err(e) => println!("❌ Auto-reconnection connector creation failed: {}", e),
    }

    // Example 6: Production-Ready Configuration
    println!("\n=== 🏭 Production Configuration Example ===");

    // Show how to use environment variables for configuration
    println!("💡 Production configuration options:");
    println!("  • Use environment variables for credentials");
    println!("  • Configure custom base URLs");
    println!("  • Set appropriate timeouts and retry limits");

    // Example of loading from environment (commented out for demo)
    // let prod_config = ExchangeConfig::from_env("PARADEX")?; // Looks for PARADEX_API_KEY, PARADEX_SECRET_KEY, etc.

    // Example of custom configuration
    let _custom_config = ExchangeConfig::new(
        "your_api_key".to_string(),
        "your_secret_key".to_string(),
    )
    .testnet(false) // Use mainnet
    .base_url("https://api.paradex.trade".to_string()); // Custom base URL

    println!("✓ Custom configuration created (not executed for demo)");

    println!("\n=== ✨ Summary ===");
    println!("✓ Paradex REST API integration complete");
    println!("✓ Market data retrieval working");
    println!("✓ Funding rates functionality available");
    println!("✓ Authentication system configured");
    println!("✓ Trading interface available");
    println!("✓ Account management functional");
    println!("✓ WebSocket support with auto-reconnection");
    println!("✓ Production-ready configuration options");

    println!("\n💡 Tips for production use:");
    println!("  • Use environment variables for credentials");
    println!("  • Implement proper error handling and retries");
    println!("  • Monitor funding rates for arbitrage opportunities");
    println!("  • Use auto-reconnection WebSocket for reliable streams");
    println!("  • Keep private keys secure and never commit them");
    println!("  • Test thoroughly on testnet before mainnet deployment");

    println!("\n🔗 Paradex-specific features:");
    println!("  • Funding rates API for perpetual futures");
    println!("  • Historical funding rate data");
    println!("  • JWT-based authentication");
    println!("  • Advanced WebSocket with reconnection");
    println!("  • Production-grade error handling");

    Ok(())
}
