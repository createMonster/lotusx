use lotusx::utils::exchange_factory::{ExchangeFactory, ExchangeTestConfigBuilder, ExchangeType};
use lotusx::utils::latency_testing::{
    calculate_liquidity_score, calculate_market_impact, ExchangePerformance, LatencyTester,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Custom Exchange Latency Test");
    println!("===============================");

    // Example 1: Quick test with only specific exchanges
    let custom_configs = ExchangeTestConfigBuilder::new()
        .add_exchange("Binance Spot".to_string(), ExchangeType::Binance, false)
        .add_exchange("Hyperliquid".to_string(), ExchangeType::Hyperliquid, false)
        .with_symbols(vec!["BTC".to_string(), "ETH".to_string()]) // Applies to last added (Hyperliquid)
        .build();

    println!("🎯 Testing {} custom exchanges:", custom_configs.len());
    for config in &custom_configs {
        println!("  - {} with symbols: {:?}", config.name, config.symbols);
    }

    // Use quick testing for this example
    let tester = LatencyTester::with_quick_config();

    #[allow(clippy::collection_is_never_read)]
    let mut all_performance = Vec::new();

    for exchange_config in custom_configs {
        println!("\n📊 Testing {} Performance", exchange_config.name);
        println!("{}", "-".repeat(30 + exchange_config.name.len()));

        // Create the exchange connector
        let client = match ExchangeFactory::create_connector(
            &exchange_config.exchange_type,
            None,
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

        // Run only market data tests for this example
        let markets_metrics = tester
            .test_markets_latency(client.as_ref(), &exchange_config.name)
            .await;

        let performance = ExchangePerformance {
            name: exchange_config.name.clone(),
            markets_metrics: markets_metrics.clone(),
            klines_metrics: markets_metrics.clone(), // Simplified for demo
            websocket_connection_time: std::time::Duration::from_millis(100),
            websocket_first_message: std::time::Duration::from_millis(1000),
            websocket_success_rate: 1.0,
            tick_to_trade_latency: std::time::Duration::from_millis(50),
            market_impact_bps: calculate_market_impact(&markets_metrics),
            liquidity_score: calculate_liquidity_score(&markets_metrics, &markets_metrics),
        };

        all_performance.push(performance);
    }

    // Example 2: Show how to test all available exchanges
    println!("\n\n🔧 All Available Exchange Types:");
    for exchange_type in ExchangeFactory::get_available_exchanges() {
        println!("  - {}", exchange_type);
    }

    // Example 3: Show environment-based configuration
    println!("\n🌍 Environment-based Configurations:");
    let env_configs = ExchangeFactory::get_test_configs_from_env();
    for config in env_configs {
        println!(
            "  - {} (requires auth: {})",
            config.name, config.requires_auth
        );
    }

    println!("\n✅ Custom latency test completed!");
    Ok(())
}
