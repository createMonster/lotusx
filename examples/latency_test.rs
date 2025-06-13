#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::suboptimal_flops
)]

use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::MarketDataSource;
use lotusx::exchanges::binance::BinanceConnector;
use lotusx::exchanges::binance_perp::BinancePerpConnector;
use lotusx::exchanges::hyperliquid::HyperliquidClient;
use std::time::{Duration, Instant};

// Configuration constants
const MARKETS_TEST_COUNT: usize = 100; // Increased for better statistics
const KLINES_TEST_COUNT: usize = 100;
const WEBSOCKET_TEST_COUNT: usize = 10;
const TEST_SYMBOLS: [&str; 3] = ["BTCUSDT", "ETHUSDT", "ADAUSDT"];
const MARKETS_DELAY_MS: u64 = 50; // Reduced for faster testing
const KLINES_DELAY_MS: u64 = 50;
const WEBSOCKET_TIMEOUT_SECS: u64 = 5;

// HFT-specific constants
const OUTLIER_THRESHOLD_MULTIPLIER: f64 = 3.0; // 3-sigma for outliers
const ARBITRAGE_PROFIT_THRESHOLD_BPS: f64 = 0.5; // 0.5 bps minimum profit

// Helper function to format duration as microseconds with 2 decimal places
fn format_us(duration: Duration) -> String {
    format!("{:.2}", duration.as_secs_f64() * 1_000_000.0)
}

// Calculate percentiles from a sorted vector
#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn calculate_percentile(sorted_data: &[Duration], percentile: f64) -> Duration {
    if sorted_data.is_empty() {
        return Duration::ZERO;
    }
    let index = (percentile / 100.0 * (sorted_data.len() - 1) as f64).round() as usize;
    sorted_data[index.min(sorted_data.len() - 1)]
}

// Calculate jitter (standard deviation)
fn calculate_jitter(latencies: &[Duration]) -> f64 {
    if latencies.len() < 2 {
        return 0.0;
    }
    let mean = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let variance = latencies
        .iter()
        .map(|&d| {
            let diff = d.as_secs_f64() * 1_000_000.0 - mean.as_secs_f64() * 1_000_000.0;
            diff * diff
        })
        .sum::<f64>()
        / (latencies.len() - 1) as f64;
    variance.sqrt()
}

// Calculate reliability score based on success rate and latency consistency
fn calculate_reliability_score(success_rate: f64, jitter: f64, avg_latency: Duration) -> f64 {
    let jitter_penalty = (jitter / avg_latency.as_secs_f64() / 1_000_000.0).min(1.0);
    (success_rate * (1.0 - jitter_penalty) * 100.0).max(0.0)
}

// Detect outliers using 3-sigma rule
fn detect_outliers(latencies: &[Duration]) -> (f64, f64) {
    if latencies.len() < 2 {
        return (0.0, 0.0);
    }
    let mean = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let std_dev = calculate_jitter(latencies);
    let threshold = mean.as_secs_f64() * 1_000_000.0 + OUTLIER_THRESHOLD_MULTIPLIER * std_dev;
    let outlier_count = latencies
        .iter()
        .filter(|&&d| d.as_secs_f64() * 1_000_000.0 > threshold)
        .count();
    let frequency = (outlier_count as f64 / latencies.len() as f64) * 100.0;
    (threshold, frequency)
}

#[derive(Debug, Clone)]
struct LatencyMetrics {
    min: Duration,
    p50: Duration,
    p95: Duration,
    p99: Duration,
    max: Duration,
    mean: Duration,
    jitter: f64,
    success_rate: f64,
    reliability_score: f64,
    outlier_threshold: f64,
    outlier_frequency: f64,
}

impl LatencyMetrics {
    fn new(latencies: &[Duration], total_attempts: usize) -> Self {
        if latencies.is_empty() {
            return Self {
                min: Duration::ZERO,
                p50: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                max: Duration::ZERO,
                mean: Duration::ZERO,
                jitter: 0.0,
                success_rate: 0.0,
                reliability_score: 0.0,
                outlier_threshold: 0.0,
                outlier_frequency: 0.0,
            };
        }

        let mut sorted = latencies.to_vec();
        sorted.sort();

        let min = *sorted.first().unwrap();
        let max = *sorted.last().unwrap();
        let p50 = calculate_percentile(&sorted, 50.0);
        let p95 = calculate_percentile(&sorted, 95.0);
        let p99 = calculate_percentile(&sorted, 99.0);
        let mean = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let jitter = calculate_jitter(latencies);
        let success_rate = latencies.len() as f64 / total_attempts as f64;
        let reliability_score = calculate_reliability_score(success_rate, jitter, mean);
        let (outlier_threshold, outlier_frequency) = detect_outliers(latencies);

        Self {
            min,
            p50,
            p95,
            p99,
            max,
            mean,
            jitter,
            success_rate,
            reliability_score,
            outlier_threshold,
            outlier_frequency,
        }
    }

    fn print_summary(&self, operation: &str) {
        println!(
            "  📊 {} Metrics ({} samples, {:.1}% success):",
            operation,
            (self.success_rate * 100.0) as usize,
            self.success_rate * 100.0
        );
        println!("    Min: {}μs", format_us(self.min));
        println!("    P50: {}μs", format_us(self.p50));
        println!("    P95: {}μs", format_us(self.p95));
        println!("    P99: {}μs", format_us(self.p99));
        println!("    Max: {}μs", format_us(self.max));
        println!("    Mean: {}μs", format_us(self.mean));
        println!("    Jitter: {:.2}μs", self.jitter);
        println!("    Reliability: {:.1}%", self.reliability_score);
        println!(
            "    Outliers: {:.1}% (threshold: {:.0}μs)",
            self.outlier_frequency, self.outlier_threshold
        );
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct ExchangePerformance {
    name: String,
    markets_metrics: LatencyMetrics,
    klines_metrics: LatencyMetrics,
    websocket_connection_time: Duration,
    websocket_first_message: Duration,
    websocket_success_rate: f64,
    tick_to_trade_latency: Duration,
    market_impact_bps: f64,
    liquidity_score: f64,
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 HFT Exchange Latency Analysis");
    println!("================================");

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

    let mut all_performance = Vec::new();

    for (exchange_name, client) in test_configs {
        println!("\n📊 Testing {} Performance", exchange_name);
        println!("{}", "-".repeat(30 + exchange_name.len()));

        // Test 1: Market data latency
        let markets_metrics = test_markets_latency(&*client, exchange_name).await;

        // Test 2: K-lines latency
        let klines_metrics = test_klines_latency(&*client, exchange_name).await;

        // Test 3: WebSocket performance
        let (ws_conn_time, ws_first_msg, ws_success_rate) =
            test_websocket_performance(&*client, exchange_name).await;

        // Test 4: Tick-to-trade simulation
        let tick_to_trade = simulate_tick_to_trade(&*client, exchange_name).await;

        // Calculate HFT-specific metrics
        let market_impact = calculate_market_impact(&markets_metrics);
        let liquidity_score = calculate_liquidity_score(&markets_metrics, &klines_metrics);

        let performance = ExchangePerformance {
            name: exchange_name.to_string(),
            markets_metrics,
            klines_metrics,
            websocket_connection_time: ws_conn_time,
            websocket_first_message: ws_first_msg,
            websocket_success_rate: ws_success_rate,
            tick_to_trade_latency: tick_to_trade,
            market_impact_bps: market_impact,
            liquidity_score,
        };

        all_performance.push(performance);
    }

    // Generate HFT report
    generate_hft_report(&all_performance);

    println!("\n🏁 HFT Latency Analysis Completed!");
    Ok(())
}

#[allow(clippy::future_not_send)]
async fn test_markets_latency(
    client: &dyn MarketDataSource,
    exchange_name: &str,
) -> LatencyMetrics {
    println!("\n🔍 Testing Market Data Latency for {}:", exchange_name);

    let mut latencies = Vec::with_capacity(MARKETS_TEST_COUNT);
    let mut total_attempts = 0;

    for i in 0..MARKETS_TEST_COUNT {
        total_attempts += 1;
        let start = Instant::now();
        let result = client.get_markets().await;
        let duration = start.elapsed();

        match result {
            Ok(markets) => {
                latencies.push(duration);
                if i < 5 || i % 20 == 0 {
                    // Show first 5 and every 20th result
                    println!(
                        "  Test {}: ✅ {}μs ({} markets)",
                        i + 1,
                        format_us(duration),
                        markets.len()
                    );
                }
            }
            Err(e) => {
                if i < 5 || i % 20 == 0 {
                    println!(
                        "  Test {}: ❌ {}μs - Error: {}",
                        i + 1,
                        format_us(duration),
                        e
                    );
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(MARKETS_DELAY_MS)).await;
    }

    let metrics = LatencyMetrics::new(&latencies, total_attempts);
    metrics.print_summary("Market Data");
    metrics
}

#[allow(clippy::future_not_send)]
async fn test_klines_latency(client: &dyn MarketDataSource, exchange_name: &str) -> LatencyMetrics {
    println!("\n📈 Testing K-Lines Latency for {}:", exchange_name);

    let mut latencies = Vec::with_capacity(KLINES_TEST_COUNT * TEST_SYMBOLS.len());
    let mut total_attempts = 0;

    for symbol in TEST_SYMBOLS {
        println!("  Testing symbol: {}", symbol);

        for i in 0..KLINES_TEST_COUNT {
            total_attempts += 1;
            let start = Instant::now();
            let result = client
                .get_klines(symbol.to_string(), "1m".to_string(), Some(10), None, None)
                .await;
            let duration = start.elapsed();

            match result {
                Ok(klines) => {
                    latencies.push(duration);
                    if i < 3 || i % 20 == 0 {
                        println!(
                            "    Test {}: ✅ {}μs ({} k-lines)",
                            i + 1,
                            format_us(duration),
                            klines.len()
                        );
                    }
                }
                Err(e) => {
                    if i < 3 || i % 20 == 0 {
                        println!(
                            "    Test {}: ❌ {}μs - Error: {}",
                            i + 1,
                            format_us(duration),
                            e
                        );
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(KLINES_DELAY_MS)).await;
        }
    }

    let metrics = LatencyMetrics::new(&latencies, total_attempts);
    metrics.print_summary("K-Lines");
    metrics
}

#[allow(clippy::future_not_send)]
async fn test_websocket_performance(
    client: &dyn MarketDataSource,
    exchange_name: &str,
) -> (Duration, Duration, f64) {
    println!("\n🔌 Testing WebSocket Performance for {}:", exchange_name);

    let mut connection_times = Vec::new();
    let mut first_message_times = Vec::new();
    let mut success_count = 0;

    for i in 0..WEBSOCKET_TEST_COUNT {
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
                connection_times.push(connection_time);

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
                        first_message_times.push(message_time);
                        success_count += 1;

                        println!(
                            "  Test {}: ✅ Conn: {}μs, First Msg: {}μs",
                            i + 1,
                            format_us(connection_time),
                            format_us(message_time)
                        );
                    }
                    Ok(None) => {
                        println!("  Test {}: ⚠️  Connected but no messages", i + 1);
                    }
                    Err(_) => {
                        println!("  Test {}: ⚠️  Connection timeout", i + 1);
                    }
                }
            }
            Err(e) => {
                let connection_time = start.elapsed();
                println!(
                    "  Test {}: ❌ Connection failed: {}μs - {}",
                    i + 1,
                    format_us(connection_time),
                    e
                );
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let avg_connection =
        connection_times.iter().sum::<Duration>() / connection_times.len().max(1) as u32;
    let avg_first_message =
        first_message_times.iter().sum::<Duration>() / first_message_times.len().max(1) as u32;
    let success_rate = success_count as f64 / WEBSOCKET_TEST_COUNT as f64;

    println!("  📊 WebSocket Summary:");
    println!("    Avg Connection: {}μs", format_us(avg_connection));
    println!("    Avg First Message: {}μs", format_us(avg_first_message));
    println!("    Success Rate: {:.1}%", success_rate * 100.0);

    (avg_connection, avg_first_message, success_rate)
}

#[allow(clippy::future_not_send)]
async fn simulate_tick_to_trade(client: &dyn MarketDataSource, exchange_name: &str) -> Duration {
    println!("\n⚡ Simulating Tick-to-Trade for {}:", exchange_name);

    let mut round_trip_times = Vec::new();

    for i in 0..10 {
        let start = Instant::now();

        // Simulate market data reception
        let market_data_start = Instant::now();
        let _market_result = client.get_markets().await;
        let market_data_time = market_data_start.elapsed();

        // Simulate order processing (simplified)
        let order_start = Instant::now();
        tokio::time::sleep(Duration::from_micros(100)).await; // Simulate order processing
        let order_time = order_start.elapsed();

        let total_time = start.elapsed();
        round_trip_times.push(total_time);

        println!(
            "  Test {}: Market Data: {}μs, Order: {}μs, Total: {}μs",
            i + 1,
            format_us(market_data_time),
            format_us(order_time),
            format_us(total_time)
        );
    }

    let avg_round_trip = round_trip_times.iter().sum::<Duration>() / round_trip_times.len() as u32;
    println!("  📊 Avg Tick-to-Trade: {}μs", format_us(avg_round_trip));

    avg_round_trip
}

fn calculate_market_impact(metrics: &LatencyMetrics) -> f64 {
    // Simplified market impact calculation based on latency consistency
    let jitter_ratio = metrics.jitter / metrics.mean.as_secs_f64() / 1_000_000.0;
    (jitter_ratio * 10.0).min(5.0) // Max 5 bps impact
}

fn calculate_liquidity_score(metrics: &LatencyMetrics, _klines_metrics: &LatencyMetrics) -> f64 {
    // Simplified liquidity score based on success rate and latency
    let success_factor = metrics.success_rate;
    let latency_factor = 1.0 / (1.0 + metrics.mean.as_secs_f64() * 1_000_000.0 / 1_000_000.0); // Normalize to 1ms
    let consistency_factor = 1.0 / (1.0 + metrics.jitter / 1000.0); // Normalize jitter

    (success_factor * latency_factor * consistency_factor * 100.0).min(100.0)
}

fn generate_hft_report(performance: &[ExchangePerformance]) {
    println!("\n{}", "=".repeat(80));
    println!("🚀 HFT EXCHANGE LATENCY REPORT");
    println!("{}", "=".repeat(80));

    // Critical Performance Metrics
    println!("\n📊 CRITICAL PERFORMANCE METRICS");
    println!("{:-<80}", "");
    println!(
        "{:<15} {:<10} {:<10} {:<10} {:<10} {:<15}",
        "Exchange", "P99 (μs)", "P95 (μs)", "Mean (μs)", "Jitter (μs)", "Reliability (%)"
    );
    println!("{:-<80}", "");

    for perf in performance {
        println!(
            "{:<15} {:<10.0} {:<10.0} {:<10.0} {:<10.0} {:<15.1}",
            perf.name,
            perf.markets_metrics.p99.as_secs_f64() * 1_000_000.0,
            perf.markets_metrics.p95.as_secs_f64() * 1_000_000.0,
            perf.markets_metrics.mean.as_secs_f64() * 1_000_000.0,
            perf.markets_metrics.jitter,
            perf.markets_metrics.reliability_score
        );
    }

    // HFT-Specific Metrics
    println!("\n⚡ HFT-SPECIFIC METRICS");
    println!("{:-<80}", "");
    println!(
        "{:<15} {:<15} {:<15} {:<15} {:<15}",
        "Exchange",
        "Tick-to-Trade (μs)",
        "Market Impact (bps)",
        "Liquidity Score",
        "WS Success (%)"
    );
    println!("{:-<80}", "");

    for perf in performance {
        println!(
            "{:<15} {:<15.0} {:<15.2} {:<15.1} {:<15.1}",
            perf.name,
            perf.tick_to_trade_latency.as_secs_f64() * 1_000_000.0,
            perf.market_impact_bps,
            perf.liquidity_score,
            perf.websocket_success_rate * 100.0
        );
    }

    // Risk Assessment
    println!("\n🚨 RISK ASSESSMENT");
    println!("{:-<80}", "");
    for perf in performance {
        let risk_level = if perf.markets_metrics.reliability_score > 90.0
            && perf.markets_metrics.outlier_frequency < 1.0
        {
            "🟢 LOW"
        } else if perf.markets_metrics.reliability_score > 70.0
            && perf.markets_metrics.outlier_frequency < 5.0
        {
            "🟡 MEDIUM"
        } else {
            "🔴 HIGH"
        };

        println!(
            "{} | {} | Outliers: {:.1}% | Reliability: {:.1}%",
            risk_level,
            perf.name,
            perf.markets_metrics.outlier_frequency,
            perf.markets_metrics.reliability_score
        );
    }

    // Cross-Exchange Arbitrage Analysis
    println!("\n🔄 CROSS-EXCHANGE ARBITRAGE ANALYSIS");
    println!("{:-<80}", "");
    if performance.len() >= 2 {
        for i in 0..performance.len() {
            for j in (i + 1)..performance.len() {
                let latency_diff = (performance[i].markets_metrics.mean.as_secs_f64()
                    - performance[j].markets_metrics.mean.as_secs_f64())
                .abs()
                    * 1_000_000.0;
                let min_profit = latency_diff / 1000.0; // Simplified calculation
                let feasible = min_profit > ARBITRAGE_PROFIT_THRESHOLD_BPS;

                println!(
                    "{} ↔ {} | Latency Diff: {:.0}μs | Min Profit: {:.2}bps | Feasible: {}",
                    performance[i].name,
                    performance[j].name,
                    latency_diff,
                    min_profit,
                    if feasible { "✅" } else { "❌" }
                );
            }
        }
    }

    println!("\n{}", "=".repeat(80));
}
