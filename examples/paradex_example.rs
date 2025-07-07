use lotusx::{
    core::{
        config::ExchangeConfig,
        traits::{AccountInfo, FundingRateSource, MarketDataSource, OrderPlacer},
        types::{OrderRequest, OrderSide, OrderType, SubscriptionType, WebSocketConfig},
    },
    exchanges::paradex::ParadexConnector,
};
use secrecy::SecretString;
use std::env;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("🚀 Starting Paradex Exchange Example (Perpetual Trading)");

    // Load configuration from environment
    let config = load_config_from_env();
    let connector = ParadexConnector::new(config);

    // Test basic connectivity
    info!("📡 Testing basic connectivity...");
    test_connectivity(&connector).await?;

    // Test market data
    info!("📊 Testing market data...");
    test_market_data(&connector).await?;

    // Test funding rates (perpetual specific)
    info!("💰 Testing funding rates...");
    test_funding_rates(&connector).await?;

    // Test WebSocket connection
    info!("🔗 Testing WebSocket connection...");
    test_websocket(&connector).await?;

    // Test account information (requires credentials)
    if connector.can_trade() {
        info!("👤 Testing account information...");
        test_account_info(&connector).await?;

        // Test order placement (uncomment for live trading)
        // warn!("⚠️  Skipping live order placement in example");
        // test_order_placement(&connector).await?;
    } else {
        warn!("⚠️  Skipping account and trading tests (missing credentials)");
    }

    info!("✅ Paradex example completed successfully!");
    Ok(())
}

fn load_config_from_env() -> ExchangeConfig {
    let api_key = env::var("PARADEX_API_KEY").unwrap_or_else(|_| {
        warn!("PARADEX_API_KEY not set, account features will be disabled");
        String::new()
    });

    let secret_key = env::var("PARADEX_SECRET_KEY").unwrap_or_else(|_| {
        warn!("PARADEX_SECRET_KEY not set, trading features will be disabled");
        String::new()
    });

    let testnet = env::var("PARADEX_TESTNET")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);

    ExchangeConfig {
        api_key: SecretString::new(api_key),
        secret_key: SecretString::new(secret_key),
        base_url: if testnet {
            Some("https://api.testnet.paradex.trade".to_string())
        } else {
            None
        },
        testnet,
    }
}

async fn test_connectivity(connector: &ParadexConnector) -> Result<(), Box<dyn std::error::Error>> {
    info!("  🔍 Fetching available markets...");
    let markets = connector.get_markets().await?;
    info!("  📈 Found {} markets", markets.len());

    if !markets.is_empty() {
        let sample_market = &markets[0];
        info!(
            "  📊 Sample market: {} (status: {})",
            sample_market.symbol, sample_market.status
        );
    }

    Ok(())
}

async fn test_market_data(connector: &ParadexConnector) -> Result<(), Box<dyn std::error::Error>> {
    let markets = connector.get_markets().await?;
    if markets.is_empty() {
        warn!("  ⚠️  No markets available for testing");
        return Ok(());
    }

    let test_symbol = markets[0].symbol.to_string();
    info!("  📊 Testing market data for symbol: {}", test_symbol);

    // Test klines
    match connector
        .get_klines(
            test_symbol.clone(),
            lotusx::core::types::KlineInterval::Hours1,
            Some(5),
            None,
            None,
        )
        .await
    {
        Ok(klines) => {
            info!("  📈 Retrieved {} klines", klines.len());
        }
        Err(e) => {
            warn!("  ⚠️  Klines not available: {}", e);
        }
    }

    Ok(())
}

async fn test_funding_rates(
    connector: &ParadexConnector,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("  💰 Fetching all funding rates...");
    match connector.get_all_funding_rates().await {
        Ok(rates) => {
            info!("  📊 Found funding rates for {} symbols", rates.len());
            if !rates.is_empty() {
                let sample_rate = &rates[0];
                info!(
                    "  💰 Sample: {} - Rate: {:?}, Next time: {:?}",
                    sample_rate.symbol, sample_rate.funding_rate, sample_rate.next_funding_time
                );
            }
        }
        Err(e) => {
            error!("  ❌ Failed to fetch funding rates: {}", e);
        }
    }

    // Test single symbol funding rate
    let markets = connector.get_markets().await?;
    if !markets.is_empty() {
        let test_symbol = markets[0].symbol.to_string();
        info!("  🎯 Fetching funding rate for {}", test_symbol);
        match connector
            .get_funding_rates(Some(vec![test_symbol.clone()]))
            .await
        {
            Ok(rates) => {
                if !rates.is_empty() {
                    info!(
                        "  💰 Funding rate: {:?}, Mark price: {:?}",
                        rates[0].funding_rate, rates[0].mark_price
                    );
                }
            }
            Err(e) => {
                warn!("  ⚠️  Single funding rate failed: {}", e);
            }
        }
    }

    Ok(())
}

async fn test_websocket(connector: &ParadexConnector) -> Result<(), Box<dyn std::error::Error>> {
    let markets = connector.get_markets().await?;
    if markets.is_empty() {
        warn!("  ⚠️  No markets available for WebSocket testing");
        return Ok(());
    }

    let test_symbol = markets[0].symbol.to_string();
    info!("  🔗 Starting WebSocket connection for {}", test_symbol);

    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::OrderBook { depth: Some(5) },
        SubscriptionType::Trades,
    ];

    let config = WebSocketConfig {
        auto_reconnect: true,
        ping_interval: Some(30),
        max_reconnect_attempts: Some(3),
    };

    match connector
        .subscribe_market_data(vec![test_symbol], subscription_types, Some(config))
        .await
    {
        Ok(mut receiver) => {
            info!("  📡 WebSocket connected, listening for 10 seconds...");
            let timeout = tokio::time::timeout(Duration::from_secs(10), async {
                let mut message_count = 0;
                while let Some(data) = receiver.recv().await {
                    message_count += 1;
                    match data {
                        lotusx::core::types::MarketDataType::Ticker(ticker) => {
                            info!("  📊 Ticker: {} @ {}", ticker.symbol, ticker.price);
                        }
                        lotusx::core::types::MarketDataType::OrderBook(book) => {
                            info!(
                                "  📖 Order Book: {} (bids: {}, asks: {})",
                                book.symbol,
                                book.bids.len(),
                                book.asks.len()
                            );
                        }
                        lotusx::core::types::MarketDataType::Trade(trade) => {
                            info!(
                                "  💱 Trade: {} {} @ {}",
                                trade.symbol, trade.quantity, trade.price
                            );
                        }
                        lotusx::core::types::MarketDataType::Kline(kline) => {
                            info!(
                                "  📈 Kline: {} {} -> {}",
                                kline.symbol, kline.open_price, kline.close_price
                            );
                        }
                    }

                    if message_count >= 10 {
                        break;
                    }
                }
                info!("  📡 Received {} messages", message_count);
            });

            if (timeout.await).is_ok() {
                info!("  ✅ WebSocket test completed");
            } else {
                info!("  ⏰ WebSocket test timed out (this is normal)");
            }
        }
        Err(e) => {
            error!("  ❌ WebSocket connection failed: {}", e);
        }
    }

    Ok(())
}

async fn test_account_info(connector: &ParadexConnector) -> Result<(), Box<dyn std::error::Error>> {
    info!("  👤 Fetching account balance...");
    match connector.get_account_balance().await {
        Ok(balances) => {
            info!("  💰 Account has {} assets", balances.len());
            for balance in balances.iter().take(5) {
                info!(
                    "  💰 {}: {} free, {} locked",
                    balance.asset, balance.free, balance.locked
                );
            }
        }
        Err(e) => {
            error!("  ❌ Failed to fetch balance: {}", e);
        }
    }

    info!("  📊 Fetching positions...");
    match connector.get_positions().await {
        Ok(positions) => {
            info!("  🎯 Found {} positions", positions.len());
            for position in &positions {
                info!(
                    "  🎯 {}: {} {:?} (PnL: {})",
                    position.symbol,
                    position.position_amount,
                    position.position_side,
                    position.unrealized_pnl
                );
            }
        }
        Err(e) => {
            error!("  ❌ Failed to fetch positions: {}", e);
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn test_order_placement(
    connector: &ParadexConnector,
) -> Result<(), Box<dyn std::error::Error>> {
    let markets = connector.get_markets().await?;
    if markets.is_empty() {
        warn!("  ⚠️  No markets available for order testing");
        return Ok(());
    }

    let test_symbol = markets[0].symbol.to_string();
    warn!("  ⚠️  This will place a real order on {}", test_symbol);

    // Create a small test order (modify as needed)
    let order = OrderRequest {
        symbol: lotusx::core::types::conversion::string_to_symbol(&test_symbol),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: lotusx::core::types::conversion::string_to_quantity("0.001"), // Very small quantity
        price: Some(lotusx::core::types::conversion::string_to_price("1.0")), // Very low price (unlikely to fill)
        time_in_force: Some(lotusx::core::types::TimeInForce::GTC),
        stop_price: None,
    };

    info!("  📝 Placing test order...");
    match connector.place_order(order).await {
        Ok(response) => {
            info!(
                "  ✅ Order placed: {} (status: {})",
                response.order_id, response.status
            );

            // Wait a moment then cancel the order
            sleep(Duration::from_secs(2)).await;

            info!("  🗑️  Cancelling test order...");
            match connector
                .cancel_order(test_symbol.clone(), response.order_id)
                .await
            {
                Ok(_) => info!("  ✅ Order cancelled successfully"),
                Err(e) => error!("  ❌ Failed to cancel order: {}", e),
            }
        }
        Err(e) => {
            error!("  ❌ Failed to place order: {}", e);
        }
    }

    Ok(())
}
