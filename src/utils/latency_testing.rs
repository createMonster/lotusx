use crate::core::traits::MarketDataSource;
use crate::core::types::KlineInterval;
use std::time::{Duration, Instant};

/// Configuration for latency tests
#[derive(Debug, Clone)]
pub struct LatencyTestConfig {
    pub markets_test_count: usize,
    pub klines_test_count: usize,
    pub websocket_test_count: usize,
    pub markets_delay_ms: u64,
    pub klines_delay_ms: u64,
    pub websocket_timeout_secs: u64,
    pub outlier_threshold_multiplier: f64,
    pub arbitrage_profit_threshold_bps: f64,
}

impl Default for LatencyTestConfig {
    fn default() -> Self {
        Self {
            markets_test_count: 100,
            klines_test_count: 100,
            websocket_test_count: 10,
            markets_delay_ms: 50,
            klines_delay_ms: 50,
            websocket_timeout_secs: 5,
            outlier_threshold_multiplier: 3.0,
            arbitrage_profit_threshold_bps: 0.5,
        }
    }
}

impl LatencyTestConfig {
    pub fn quick() -> Self {
        Self {
            markets_test_count: 20,
            klines_test_count: 20,
            websocket_test_count: 3,
            markets_delay_ms: 100,
            klines_delay_ms: 100,
            websocket_timeout_secs: 5,
            outlier_threshold_multiplier: 3.0,
            arbitrage_profit_threshold_bps: 0.5,
        }
    }

    pub fn comprehensive() -> Self {
        Self {
            markets_test_count: 200,
            klines_test_count: 200,
            websocket_test_count: 20,
            markets_delay_ms: 25,
            klines_delay_ms: 25,
            websocket_timeout_secs: 10,
            outlier_threshold_multiplier: 3.0,
            arbitrage_profit_threshold_bps: 0.5,
        }
    }
}

/// Latency metrics with statistical analysis
#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    pub min: Duration,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
    pub mean: Duration,
    pub jitter: f64,
    pub success_rate: f64,
    pub reliability_score: f64,
    pub outlier_threshold: f64,
    pub outlier_frequency: f64,
}

impl LatencyMetrics {
    pub fn new(latencies: &[Duration], total_attempts: usize) -> Self {
        if latencies.is_empty() {
            return Self::zero();
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
        #[allow(clippy::cast_precision_loss)]
        let success_rate = latencies.len() as f64 / total_attempts as f64;
        let reliability_score = calculate_reliability_score(success_rate, jitter, mean);
        let (outlier_threshold, outlier_frequency) = detect_outliers(latencies, 3.0);

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

    fn zero() -> Self {
        Self {
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
        }
    }

    pub fn print_summary(&self, operation: &str) {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let sample_count = (self.success_rate * 100.0) as usize;
        println!(
            "  📊 {} Metrics ({} samples, {:.1}% success):",
            operation,
            sample_count,
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

/// Exchange performance metrics
#[derive(Debug)]
pub struct ExchangePerformance {
    pub name: String,
    pub markets_metrics: LatencyMetrics,
    pub klines_metrics: LatencyMetrics,
    pub websocket_connection_time: Duration,
    pub websocket_first_message: Duration,
    pub websocket_success_rate: f64,
    pub tick_to_trade_latency: Duration,
    pub market_impact_bps: f64,
    pub liquidity_score: f64,
}

/// Main latency tester
pub struct LatencyTester {
    config: LatencyTestConfig,
}

impl LatencyTester {
    pub fn new(config: LatencyTestConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(LatencyTestConfig::default())
    }

    pub fn with_quick_config() -> Self {
        Self::new(LatencyTestConfig::quick())
    }

    pub fn with_comprehensive_config() -> Self {
        Self::new(LatencyTestConfig::comprehensive())
    }

    /// Test market data latency
    #[allow(clippy::future_not_send)]
    pub async fn test_markets_latency(
        &self,
        client: &dyn MarketDataSource,
        exchange_name: &str,
    ) -> LatencyMetrics {
        println!("\n🔍 Testing Market Data Latency for {}:", exchange_name);

        let mut latencies = Vec::with_capacity(self.config.markets_test_count);
        let mut total_attempts = 0;

        for i in 0..self.config.markets_test_count {
            total_attempts += 1;
            let start = Instant::now();
            let result = client.get_markets().await;
            let duration = start.elapsed();

            match result {
                Ok(markets) => {
                    latencies.push(duration);
                    if i < 5 || i % 20 == 0 {
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

            tokio::time::sleep(Duration::from_millis(self.config.markets_delay_ms)).await;
        }

        let metrics = LatencyMetrics::new(&latencies, total_attempts);
        metrics.print_summary("Market Data");
        metrics
    }

    /// Test K-lines latency
    #[allow(clippy::future_not_send)]
    pub async fn test_klines_latency(
        &self,
        client: &dyn MarketDataSource,
        exchange_name: &str,
        symbols: &[String],
    ) -> LatencyMetrics {
        println!("\n📈 Testing K-Lines Latency for {}:", exchange_name);

        let mut latencies = Vec::with_capacity(self.config.klines_test_count * symbols.len());
        let mut total_attempts = 0;

        for symbol in symbols {
            println!("  Testing symbol: {}", symbol);

            for i in 0..self.config.klines_test_count {
                total_attempts += 1;
                let start = Instant::now();
                let result = client
                    .get_klines(
                        symbol.clone(),
                        KlineInterval::Minutes1,
                        Some(10),
                        None,
                        None,
                    )
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

                tokio::time::sleep(Duration::from_millis(self.config.klines_delay_ms)).await;
            }
        }

        let metrics = LatencyMetrics::new(&latencies, total_attempts);
        metrics.print_summary("K-Lines");
        metrics
    }

    /// Test WebSocket performance
    #[allow(clippy::future_not_send)]
    pub async fn test_websocket_performance(
        &self,
        client: &dyn MarketDataSource,
        exchange_name: &str,
        test_symbol: &str,
    ) -> (Duration, Duration, f64) {
        println!("\n🔌 Testing WebSocket Performance for {}:", exchange_name);

        let mut connection_times = Vec::new();
        let mut first_message_times = Vec::new();
        let mut success_count = 0;

        for i in 0..self.config.websocket_test_count {
            let start = Instant::now();

            match client
                .subscribe_market_data(
                    vec![test_symbol.to_string()],
                    vec![crate::core::types::SubscriptionType::Ticker],
                    None,
                )
                .await
            {
                Ok(mut receiver) => {
                    let connection_time = start.elapsed();
                    connection_times.push(connection_time);

                    let message_start = Instant::now();
                    match tokio::time::timeout(
                        Duration::from_secs(self.config.websocket_timeout_secs),
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
        #[allow(clippy::cast_precision_loss)]
        let success_rate = success_count as f64 / self.config.websocket_test_count as f64;

        println!("  📊 WebSocket Summary:");
        println!("    Avg Connection: {}μs", format_us(avg_connection));
        println!("    Avg First Message: {}μs", format_us(avg_first_message));
        println!("    Success Rate: {:.1}%", success_rate * 100.0);

        (avg_connection, avg_first_message, success_rate)
    }

    /// Simulate tick-to-trade latency
    #[allow(clippy::future_not_send)]
    pub async fn simulate_tick_to_trade(
        &self,
        client: &dyn MarketDataSource,
        exchange_name: &str,
    ) -> Duration {
        println!("\n⚡ Simulating Tick-to-Trade for {}:", exchange_name);

        let mut round_trip_times = Vec::new();

        for i in 0..10 {
            let start = Instant::now();

            let market_data_start = Instant::now();
            let _market_result = client.get_markets().await;
            let market_data_time = market_data_start.elapsed();

            let order_start = Instant::now();
            tokio::time::sleep(Duration::from_micros(100)).await;
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

        let avg_round_trip =
            round_trip_times.iter().sum::<Duration>() / round_trip_times.len() as u32;
        println!("  📊 Avg Tick-to-Trade: {}μs", format_us(avg_round_trip));

        avg_round_trip
    }
}

// Helper functions

/// Format duration as microseconds with 2 decimal places
pub fn format_us(duration: Duration) -> String {
    format!("{:.2}", duration.as_secs_f64() * 1_000_000.0)
}

/// Calculate percentiles from a sorted vector
#[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
fn calculate_percentile(sorted_data: &[Duration], percentile: f64) -> Duration {
    if sorted_data.is_empty() {
        return Duration::ZERO;
    }
    let index = (percentile / 100.0 * (sorted_data.len() - 1) as f64).round() as usize;
    sorted_data[index.min(sorted_data.len() - 1)]
}

/// Calculate jitter (standard deviation)
fn calculate_jitter(latencies: &[Duration]) -> f64 {
    if latencies.len() < 2 {
        return 0.0;
    }
    let mean = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    #[allow(clippy::cast_precision_loss)]
    let variance = latencies
        .iter()
        .map(|&d| {
            let diff = d
                .as_secs_f64()
                .mul_add(1_000_000.0, -(mean.as_secs_f64() * 1_000_000.0));
            diff * diff
        })
        .sum::<f64>()
        / (latencies.len() - 1) as f64;
    variance.sqrt()
}

/// Calculate reliability score based on success rate and latency consistency
fn calculate_reliability_score(success_rate: f64, jitter: f64, avg_latency: Duration) -> f64 {
    let jitter_penalty = (jitter / avg_latency.as_secs_f64() / 1_000_000.0).min(1.0);
    (success_rate * (1.0 - jitter_penalty) * 100.0).max(0.0)
}

/// Detect outliers using configurable sigma rule
fn detect_outliers(latencies: &[Duration], threshold_multiplier: f64) -> (f64, f64) {
    if latencies.len() < 2 {
        return (0.0, 0.0);
    }
    let mean = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let std_dev = calculate_jitter(latencies);
    let threshold = mean
        .as_secs_f64()
        .mul_add(1_000_000.0, threshold_multiplier * std_dev);
    let outlier_count = latencies
        .iter()
        .filter(|&&d| d.as_secs_f64() * 1_000_000.0 > threshold)
        .count();
    #[allow(clippy::cast_precision_loss)]
    let frequency = (outlier_count as f64 / latencies.len() as f64) * 100.0;
    (threshold, frequency)
}

/// Calculate market impact based on latency consistency
pub fn calculate_market_impact(metrics: &LatencyMetrics) -> f64 {
    let jitter_ratio = metrics.jitter / metrics.mean.as_secs_f64() / 1_000_000.0;
    (jitter_ratio * 10.0).min(5.0)
}

/// Calculate liquidity score based on performance metrics
pub fn calculate_liquidity_score(
    metrics: &LatencyMetrics,
    _klines_metrics: &LatencyMetrics,
) -> f64 {
    let success_factor = metrics.success_rate;
    let latency_factor = 1.0 / (1.0 + metrics.mean.as_secs_f64() * 1_000_000.0 / 1_000_000.0);
    let consistency_factor = 1.0 / (1.0 + metrics.jitter / 1000.0);

    (success_factor * latency_factor * consistency_factor * 100.0).min(100.0)
}
