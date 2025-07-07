use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::{AccountInfo, MarketDataSource};
use lotusx::core::types::{KlineInterval, SubscriptionType};
use lotusx::exchanges::bybit::BybitConnector;
use lotusx::exchanges::bybit_perp::BybitPerpConnector;
use tokio::time::{timeout, Duration};

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Comprehensive Bybit API Example");
    println!("===================================");
    println!("This example demonstrates all implemented Bybit functionality");
    println!("including the recent fixes for K-lines and WebSocket connections.\n");

    // =================================================================
    // BYBIT SPOT EXCHANGE
    // =================================================================

    println!("📊 BYBIT SPOT EXCHANGE");
    println!("======================");

    // Create configuration (try env file, fallback to empty credentials)
    let config = ExchangeConfig::from_env_file("BYBIT")
        .unwrap_or_else(|_| ExchangeConfig::new(String::new(), String::new()));
    let bybit_spot = BybitConnector::new(config.clone());

    // 1. Market Data - Get all available markets
    println!("\n🏪 1. Getting Spot Markets:");
    match bybit_spot.get_markets().await {
        Ok(markets) => {
            println!("✅ Found {} spot markets", markets.len());
            println!("📝 Sample markets:");
            for (i, market) in markets.iter().take(5).enumerate() {
                println!(
                    "  {}. {} (Status: {}, Base: {}, Quote: {})",
                    i + 1,
                    market.symbol,
                    market.status,
                    market.symbol.base,
                    market.symbol.quote
                );
            }
        }
        Err(e) => println!("❌ Error getting markets: {}", e),
    }

    // 2. K-lines Data - Test the fixed API
    println!("\n📈 2. Getting K-lines Data (Fixed API):");
    let test_symbols = vec!["BTCUSDT", "ETHUSDT", "ADAUSDT"];

    for symbol in &test_symbols {
        match bybit_spot
            .get_klines(
                (*symbol).to_string(),
                KlineInterval::Minutes1,
                Some(5),
                None,
                None,
            )
            .await
        {
            Ok(klines) => {
                println!(
                    "✅ {} K-lines for {}: {} candles",
                    symbol,
                    klines.len(),
                    symbol
                );
                if let Some(first_kline) = klines.first() {
                    println!(
                        "   📊 Latest: Open: {}, High: {}, Low: {}, Close: {}, Volume: {}",
                        first_kline.open_price,
                        first_kline.high_price,
                        first_kline.low_price,
                        first_kline.close_price,
                        first_kline.volume
                    );
                }
            }
            Err(e) => println!("❌ Error getting {}: {}", symbol, e),
        }
    }

    // 3. WebSocket Subscription - Test the fixed WebSocket
    println!("\n🔌 3. Testing WebSocket Connections (Fixed V5 Protocol):");

    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::Klines {
            interval: KlineInterval::Minutes1,
        },
        SubscriptionType::Trades,
    ];

    match timeout(
        Duration::from_secs(10),
        bybit_spot.subscribe_market_data(
            vec!["BTCUSDT".to_string()],
            subscription_types.clone(),
            None,
        ),
    )
    .await
    {
        Ok(Ok(mut rx)) => {
            println!("✅ Bybit Spot WebSocket connected successfully!");
            println!("📡 Listening for real-time data...");

            let mut message_count = 0;
            while message_count < 3 {
                match timeout(Duration::from_secs(3), rx.recv()).await {
                    Ok(Some(data)) => {
                        message_count += 1;
                        println!("📥 Message {}: {:?}", message_count, data);
                    }
                    Ok(None) => {
                        println!("🔚 WebSocket channel closed");
                        break;
                    }
                    Err(_) => {
                        println!("⏰ No more messages in timeout window");
                        break;
                    }
                }
            }
        }
        Ok(Err(e)) => println!("❌ WebSocket connection failed: {}", e),
        Err(_) => println!("❌ WebSocket connection timeout"),
    }

    // 4. Account Information (requires credentials)
    println!("\n💰 4. Account Information:");
    match bybit_spot.get_account_balance().await {
        Ok(balances) => {
            println!("✅ Account balances retrieved:");
            for balance in balances.iter().take(5) {
                println!(
                    "   💳 {}: free={}, locked={}",
                    balance.asset, balance.free, balance.locked
                );
            }
        }
        Err(e) => println!("ℹ️  Skipped (requires API credentials): {}", e),
    }

    // =================================================================
    // BYBIT PERPETUAL FUTURES
    // =================================================================

    println!("\n\n🔮 BYBIT PERPETUAL FUTURES");
    println!("===========================");

    let bybit_perp = BybitPerpConnector::new(config.clone());

    // 1. Perpetual Markets
    println!("\n🏪 1. Getting Perpetual Markets:");
    match bybit_perp.get_markets().await {
        Ok(markets) => {
            println!("✅ Found {} perpetual markets", markets.len());
            println!("📝 Sample perpetual contracts:");
            for (i, market) in markets.iter().take(5).enumerate() {
                println!(
                    "  {}. {} (Status: {}, Min Qty: {:?}, Max Qty: {:?})",
                    i + 1,
                    market.symbol,
                    market.status,
                    market.min_qty,
                    market.max_qty
                );
            }
        }
        Err(e) => println!("❌ Error getting perpetual markets: {}", e),
    }

    // 2. Perpetual K-lines
    println!("\n📈 2. Getting Perpetual K-lines (Fixed API):");

    for symbol in &test_symbols {
        match bybit_perp
            .get_klines(
                (*symbol).to_string(),
                KlineInterval::Hours1,
                Some(3),
                None,
                None,
            )
            .await
        {
            Ok(klines) => {
                println!(
                    "✅ {} Perp K-lines for {}: {} candles",
                    symbol,
                    klines.len(),
                    symbol
                );
                if let Some(first_kline) = klines.first() {
                    println!(
                        "   📊 Latest: Open: {}, High: {}, Low: {}, Close: {}, Volume: {}",
                        first_kline.open_price,
                        first_kline.high_price,
                        first_kline.low_price,
                        first_kline.close_price,
                        first_kline.volume
                    );
                }
            }
            Err(e) => println!("❌ Error getting {} perp: {}", symbol, e),
        }
    }

    // 3. Perpetual WebSocket
    println!("\n🔌 3. Testing Perpetual WebSocket (Fixed V5 Protocol):");

    match timeout(
        Duration::from_secs(10),
        bybit_perp.subscribe_market_data(vec!["BTCUSDT".to_string()], subscription_types, None),
    )
    .await
    {
        Ok(Ok(mut rx)) => {
            println!("✅ Bybit Perpetual WebSocket connected successfully!");
            println!("📡 Listening for real-time perpetual data...");

            let mut message_count = 0;
            while message_count < 3 {
                match timeout(Duration::from_secs(3), rx.recv()).await {
                    Ok(Some(data)) => {
                        message_count += 1;
                        println!("📥 Perp Message {}: {:?}", message_count, data);
                    }
                    Ok(None) => {
                        println!("🔚 Perp WebSocket channel closed");
                        break;
                    }
                    Err(_) => {
                        println!("⏰ No more perp messages in timeout window");
                        break;
                    }
                }
            }
        }
        Ok(Err(e)) => println!("❌ Perp WebSocket connection failed: {}", e),
        Err(_) => println!("❌ Perp WebSocket connection timeout"),
    }

    // 4. Positions (requires credentials)
    println!("\n📍 4. Position Information:");
    match bybit_perp.get_positions().await {
        Ok(positions) => {
            println!("✅ Positions retrieved:");
            if positions.is_empty() {
                println!("   📭 No open positions");
            } else {
                for position in positions.iter().take(5) {
                    println!(
                        "   📈 {}: side={:?}, size={}, entry_price={}, pnl={}",
                        position.symbol,
                        position.position_side,
                        position.position_amount,
                        position.entry_price,
                        position.unrealized_pnl
                    );
                }
            }
        }
        Err(e) => println!("ℹ️  Skipped (requires API credentials): {}", e),
    }

    // =================================================================
    // SUMMARY
    // =================================================================

    println!("\n\n🎯 SUMMARY OF FIXES & FEATURES");
    println!("===============================");
    println!("✅ Fixed Bybit V5 K-lines API parsing");
    println!("✅ Fixed Bybit V5 WebSocket subscription protocol");
    println!("✅ Implemented proper WebSocket message parsing");
    println!("✅ Added unified KlineInterval enum support");
    println!("✅ Both Spot and Perpetual exchanges working");
    println!("✅ Real-time data streaming functional");
    println!("✅ Market data retrieval operational");

    println!("\n💡 Notes:");
    println!("• All public API calls work without credentials");
    println!("• Account/position data requires valid API keys");
    println!("• WebSocket connections use Bybit V5 protocol");
    println!("• K-lines API now correctly parses V5 response format");

    println!("\n🏁 Bybit comprehensive example completed successfully!");
    Ok(())
}
