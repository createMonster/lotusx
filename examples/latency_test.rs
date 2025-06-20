#![allow(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::suboptimal_flops
)]

use lotusx::utils::exchange_factory::ExchangeFactory;
use lotusx::utils::latency_testing::{
    calculate_liquidity_score, calculate_market_impact, ExchangePerformance, LatencyTestConfig,
    LatencyTester,
};
use std::time::Duration;

// HFT-specific constants
const ARBITRAGE_PROFIT_THRESHOLD_BPS: f64 = 0.5; // 0.5 bps minimum profit

// Helper function to format duration as microseconds with 2 decimal places
#[allow(dead_code)]
fn format_us(duration: Duration) -> String {
    format!("{:.2}", duration.as_secs_f64() * 1_000_000.0)
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 HFT Exchange Latency Analysis");
    println!("================================");

    // Get test configuration - choose between quick, default, or comprehensive
    let test_config = if std::env::args().any(|arg| arg == "--quick") {
        LatencyTestConfig::quick()
    } else if std::env::args().any(|arg| arg == "--comprehensive") {
        LatencyTestConfig::comprehensive()
    } else {
        LatencyTestConfig::default()
    };

    println!("📋 Test Configuration:");
    println!("  Markets tests: {}", test_config.markets_test_count);
    println!("  K-lines tests: {}", test_config.klines_test_count);
    println!("  WebSocket tests: {}", test_config.websocket_test_count);

    let tester = LatencyTester::new(test_config);

    // Get exchange configurations - automatically detects available credentials
    let exchange_configs = if std::env::args().any(|arg| arg == "--all") {
        ExchangeFactory::get_test_configs_from_env()
    } else {
        // Use default exchanges that don't require credentials
        ExchangeFactory::get_default_test_configs()
    };

    println!("\n🎯 Testing {} exchanges:", exchange_configs.len());
    for config in &exchange_configs {
        println!("  - {} (testnet: {})", config.name, config.testnet);
    }

    let mut all_performance = Vec::new();

    for exchange_config in exchange_configs {
        println!("\n📊 Testing {} Performance", exchange_config.name);
        println!("{}", "-".repeat(30 + exchange_config.name.len()));

        // Create the exchange connector
        let client = match ExchangeFactory::create_connector(
            &exchange_config.exchange_type,
            None, // Use default read-only config
            exchange_config.testnet,
        ) {
            Ok(client) => client,
            Err(e) => {
                println!(
                    "❌ Failed to create connector for {}: {}",
                    exchange_config.name, e
                );
                continue;
            }
        };

        // Test 1: Market data latency
        let markets_metrics = tester
            .test_markets_latency(client.as_ref(), &exchange_config.name)
            .await;

        // Test 2: K-lines latency
        let klines_metrics = tester
            .test_klines_latency(
                client.as_ref(),
                &exchange_config.name,
                &exchange_config.symbols,
            )
            .await;

        // Test 3: WebSocket performance
        let default_symbol = "BTCUSDT".to_string();
        let test_symbol = exchange_config.symbols.first().unwrap_or(&default_symbol);
        let (ws_conn_time, ws_first_msg, ws_success_rate) = tester
            .test_websocket_performance(client.as_ref(), &exchange_config.name, test_symbol)
            .await;

        // Test 4: Tick-to-trade simulation
        let tick_to_trade = tester
            .simulate_tick_to_trade(client.as_ref(), &exchange_config.name)
            .await;

        // Calculate HFT-specific metrics
        let market_impact = calculate_market_impact(&markets_metrics);
        let liquidity_score = calculate_liquidity_score(&markets_metrics, &klines_metrics);

        let performance = ExchangePerformance {
            name: exchange_config.name.clone(),
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

    println!("\n💡 Usage Tips:");
    println!("  --quick          : Run with reduced test counts for faster results");
    println!("  --comprehensive  : Run with increased test counts for better statistics");
    println!("  --all           : Include exchanges that require credentials (set env vars)");
    println!("\n🔧 Environment Variables for Authenticated Testing:");
    println!("  BACKPACK_API_KEY=your_key BACKPACK_SECRET_KEY=your_secret");

    println!("\n🏁 HFT Latency Analysis Completed!");
    Ok(())
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
