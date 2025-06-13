use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::MarketDataSource;
use lotusx::exchanges::binance::BinanceConnector;
use lotusx::exchanges::binance_perp::BinancePerpConnector;
use lotusx::exchanges::hyperliquid::HyperliquidClient;
use std::time::{Duration, Instant};

// Configuration constants
const MARKETS_TEST_COUNT: usize = 10;
const KLINES_TEST_COUNT: usize = 10;
const TEST_SYMBOLS: [&str; 3] = ["BTCUSDT", "ETHUSDT", "ADAUSDT"];
const MARKETS_DELAY_MS: u64 = 100;
const KLINES_DELAY_MS: u64 = 200;
const WEBSOCKET_TIMEOUT_SECS: u64 = 5;

// Helper function to format duration as milliseconds with 2 decimal places
fn format_ms(duration: Duration) -> String {
    format!("{:.2}", duration.as_secs_f64() * 1000.0)
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Exchange Latency Test");
    println!("========================");

    // Test configurations - using trait objects
    let test_configs: Vec<(&str, Box<dyn MarketDataSource>)> = vec![
        (
            "Binance Spot",
            Box::new(BinanceConnector::new(
                ExchangeConfig::new("your_api_key".to_string(), "your_secret_key".to_string())
                    .testnet(false),
            )),
        ),
        (
            "Binance Perp",
            Box::new(BinancePerpConnector::new(
                ExchangeConfig::new("your_api_key".to_string(), "your_secret_key".to_string())
                    .testnet(false),
            )),
        ),
        ("Hyperliquid", Box::new(HyperliquidClient::read_only(false))),
    ];

    for (exchange_name, client) in test_configs {
        println!("\n📊 Testing {} Latency", exchange_name);
        println!("{}", "-".repeat(30 + exchange_name.len()));

        // Test 1: get_markets latency
        test_get_markets_latency(&*client, exchange_name).await;

        // Test 2: get_klines latency (if supported)
        test_get_klines_latency(&*client, exchange_name).await;

        // Test 3: Multiple sequential requests
        test_sequential_requests(&*client, exchange_name).await;

        // Test 4: WebSocket connection latency
        test_websocket_latency(&*client, exchange_name).await;
    }

    println!("\n🏁 Latency testing completed!");
    Ok(())
}

#[allow(clippy::future_not_send)]
async fn test_get_markets_latency(client: &dyn MarketDataSource, exchange_name: &str) {
    println!("\n🔍 Testing get_markets latency for {}:", exchange_name);

    let mut latencies = Vec::with_capacity(MARKETS_TEST_COUNT);

    for i in 0..MARKETS_TEST_COUNT {
        let start = Instant::now();
        let result = client.get_markets().await;
        let duration = start.elapsed();

        match result {
            Ok(markets) => {
                latencies.push(duration);
                println!(
                    "  Test {}: ✅ {}ms ({} markets)",
                    i + 1,
                    format_ms(duration),
                    markets.len()
                );
            }
            Err(e) => {
                println!(
                    "  Test {}: ❌ {}ms - Error: {}",
                    i + 1,
                    format_ms(duration),
                    e
                );
            }
        }

        // Small delay between requests to avoid rate limiting
        tokio::time::sleep(Duration::from_millis(MARKETS_DELAY_MS)).await;
    }

    if !latencies.is_empty() {
        print_latency_stats(&latencies, "get_markets");
    }
}

#[allow(clippy::future_not_send)]
async fn test_get_klines_latency(client: &dyn MarketDataSource, exchange_name: &str) {
    println!("\n📈 Testing get_klines latency for {}:", exchange_name);

    let mut latencies = Vec::with_capacity(KLINES_TEST_COUNT * TEST_SYMBOLS.len());

    for symbol in TEST_SYMBOLS {
        println!("  Testing symbol: {}", symbol);

        for i in 0..KLINES_TEST_COUNT {
            let start = Instant::now();
            let result = client
                .get_klines(symbol.to_string(), "1m".to_string(), Some(10), None, None)
                .await;
            let duration = start.elapsed();

            match result {
                Ok(klines) => {
                    latencies.push(duration);
                    println!(
                        "    Test {}: ✅ {}ms ({} k-lines)",
                        i + 1,
                        format_ms(duration),
                        klines.len()
                    );
                }
                Err(e) => {
                    println!(
                        "    Test {}: ❌ {}ms - Error: {}",
                        i + 1,
                        format_ms(duration),
                        e
                    );
                }
            }

            // Small delay between requests
            tokio::time::sleep(Duration::from_millis(KLINES_DELAY_MS)).await;
        }
    }

    if !latencies.is_empty() {
        print_latency_stats(&latencies, "get_klines");
    }
}

#[allow(clippy::future_not_send)]
async fn test_sequential_requests(client: &dyn MarketDataSource, exchange_name: &str) {
    println!(
        "\n⚡ Testing sequential requests latency for {}:",
        exchange_name
    );

    let start = Instant::now();
    let mut latencies = Vec::new();
    let mut success_count = 0;

    // Request 1: get_markets
    let req1_start = Instant::now();
    match client.get_markets().await {
        Ok(_) => {
            let duration = req1_start.elapsed();
            latencies.push(duration);
            success_count += 1;
            println!("  ✅ get_markets: {}ms", format_ms(duration));
        }
        Err(e) => {
            println!(
                "  ❌ get_markets: {}ms - Error: {}",
                format_ms(req1_start.elapsed()),
                e
            );
        }
    }

    // Request 2: get_klines for BTCUSDT
    let req2_start = Instant::now();
    match client
        .get_klines("BTCUSDT".to_string(), "1m".to_string(), Some(5), None, None)
        .await
    {
        Ok(_) => {
            let duration = req2_start.elapsed();
            latencies.push(duration);
            success_count += 1;
            println!("  ✅ get_klines BTCUSDT: {}ms", format_ms(duration));
        }
        Err(e) => {
            println!(
                "  ❌ get_klines BTCUSDT: {}ms - Error: {}",
                format_ms(req2_start.elapsed()),
                e
            );
        }
    }

    // Request 3: get_klines for ETHUSDT
    let req3_start = Instant::now();
    match client
        .get_klines("ETHUSDT".to_string(), "1m".to_string(), Some(5), None, None)
        .await
    {
        Ok(_) => {
            let duration = req3_start.elapsed();
            latencies.push(duration);
            success_count += 1;
            println!("  ✅ get_klines ETHUSDT: {}ms", format_ms(duration));
        }
        Err(e) => {
            println!(
                "  ❌ get_klines ETHUSDT: {}ms - Error: {}",
                format_ms(req3_start.elapsed()),
                e
            );
        }
    }

    let total_duration = start.elapsed();

    println!(
        "  Sequential requests: {}ms total, {} successful",
        format_ms(total_duration),
        success_count
    );

    if !latencies.is_empty() {
        print_latency_stats(&latencies, "sequential");
    }
}

#[allow(clippy::future_not_send)]
async fn test_websocket_latency(client: &dyn MarketDataSource, exchange_name: &str) {
    println!(
        "\n🔌 Testing WebSocket connection latency for {}:",
        exchange_name
    );

    let start = Instant::now();

    match client
        .subscribe_market_data(
            vec!["BTCUSDT".to_string()],
            vec![lotusx::core::types::SubscriptionType::Ticker],
            None,
        )
        .await
    {
        Ok(mut receiver) => {
            let connection_time = start.elapsed();
            println!(
                "  ✅ WebSocket connected in {}ms",
                format_ms(connection_time)
            );

            // Wait for first message
            let message_start = Instant::now();
            match tokio::time::timeout(
                Duration::from_secs(WEBSOCKET_TIMEOUT_SECS),
                receiver.recv(),
            )
            .await
            {
                Ok(Some(_)) => {
                    let message_time = message_start.elapsed();
                    println!(
                        "  ✅ First message received in {}ms",
                        format_ms(message_time)
                    );
                }
                Ok(None) => {
                    println!("  ⚠️  WebSocket closed without messages");
                }
                Err(_) => {
                    println!("  ⚠️  Timeout waiting for first message");
                }
            }
        }
        Err(e) => {
            let connection_time = start.elapsed();
            println!(
                "  ❌ WebSocket connection failed in {}ms: {}",
                format_ms(connection_time),
                e
            );
        }
    }
}

#[allow(clippy::cast_possible_wrap, clippy::cast_precision_loss)]
fn print_latency_stats(latencies: &[Duration], operation: &str) {
    if latencies.is_empty() {
        return;
    }

    let min = latencies.iter().min().unwrap();
    let max = latencies.iter().max().unwrap();
    let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;

    // Calculate median
    let mut sorted = latencies.to_vec();
    sorted.sort();
    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2
    } else {
        sorted[sorted.len() / 2]
    };

    println!("  📊 {} Stats ({} samples):", operation, latencies.len());
    println!("    Min: {}ms", format_ms(*min));
    println!("    Max: {}ms", format_ms(*max));
    println!("    Avg: {}ms", format_ms(avg));
    println!("    Median: {}ms", format_ms(median));

    // Calculate standard deviation
    let variance = latencies
        .iter()
        .map(|&d| {
            let diff = d.as_secs_f64() * 1000.0 - avg.as_secs_f64() * 1000.0;
            diff * diff
        })
        .sum::<f64>()
        / latencies.len() as f64;
    let std_dev = variance.sqrt();
    println!("    Std Dev: {:.2}ms", std_dev);
}
