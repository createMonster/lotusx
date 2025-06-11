use lotusx::{
    core::{
        config::{ConfigError, ExchangeConfig},
        traits::ExchangeConnector,
    },
    exchanges::binance::BinanceConnector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 LotuSX Secure Configuration Examples");
    println!("=====================================\n");

    // Example 1: Environment Variables (Recommended)
    println!("1. 📝 Loading from Environment Variables:");
    match ExchangeConfig::from_env("BINANCE") {
        Ok(config) => {
            println!("   ✅ Configuration loaded from environment");
            println!("   🔍 Has credentials: {}", config.has_credentials());
            println!("   🧪 Testnet mode: {}", config.testnet);

            if config.has_credentials() {
                println!("   🚀 Ready for authenticated operations");

                // Test with actual connector
                let connector = BinanceConnector::new(config);
                demo_authenticated_operations(&connector).await?;
            } else {
                println!("   📊 Running in read-only mode");
            }
        }
        Err(ConfigError::MissingEnvironmentVariable(var)) => {
            println!("   ⚠️  Missing environment variable: {}", var);
            println!("   💡 Set it with: export {}=your_value", var);
        }
        Err(e) => {
            println!("   ❌ Configuration error: {}", e);
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 2: Read-Only Configuration
    println!("2. 👁️  Read-Only Configuration:");
    let readonly_config = ExchangeConfig::read_only().testnet(true);
    println!("   ✅ Created read-only configuration");
    println!(
        "   🔍 Has credentials: {}",
        readonly_config.has_credentials()
    );

    let readonly_connector = BinanceConnector::new(readonly_config);
    demo_public_operations(&readonly_connector).await?;

    println!("\n{}\n", "=".repeat(50));

    // Example 3: Manual Configuration (Development Only)
    println!("3. 🛠️  Manual Configuration (Development Only):");

    // WARNING: Never hardcode real credentials!
    let dev_config = ExchangeConfig::new(
        "test_api_key".to_string(),
        "test_secret_key".to_string(),
    )
    .testnet(true)  // Always use testnet for development
    .base_url("https://testnet.binance.vision".to_string());

    println!("   ✅ Created development configuration");
    println!("   🔍 Has credentials: {}", dev_config.has_credentials());
    println!("   🧪 Testnet mode: {}", dev_config.testnet);

    let _dev_connector = BinanceConnector::new(dev_config);
    println!("   📊 Development connector ready");

    println!("\n{}\n", "=".repeat(50));

    // Example 4: Configuration Validation
    println!("4. ✅ Configuration Validation:");
    demonstrate_config_validation().await?;

    println!("\n{}\n", "=".repeat(50));

    // Example 5: Error Handling
    println!("5. 🚨 Error Handling:");
    demonstrate_error_handling().await?;

    println!("\n🎉 All examples completed successfully!");
    Ok(())
}

async fn demo_authenticated_operations(
    connector: &BinanceConnector,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔑 Testing authenticated operations...");

    // Get markets (this works with or without credentials)
    match connector.get_markets().await {
        Ok(markets) => {
            println!("   📈 Retrieved {} markets", markets.len());

            // Show a few examples
            for market in markets.iter().take(3) {
                println!("      - {} ({})", market.symbol.symbol, market.status);
            }
        }
        Err(e) => {
            println!("   ❌ Failed to get markets: {}", e);
        }
    }

    // Note: We don't actually place orders in examples for safety
    println!("   💡 Order placement would work with valid credentials");

    Ok(())
}

async fn demo_public_operations(
    connector: &BinanceConnector,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   📊 Testing public operations...");

    match connector.get_markets().await {
        Ok(markets) => {
            println!(
                "   ✅ Successfully retrieved {} markets without credentials",
                markets.len()
            );

            // Find some popular markets
            let popular_symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"];
            for symbol in &popular_symbols {
                if let Some(market) = markets.iter().find(|m| m.symbol.symbol == *symbol) {
                    println!(
                        "      📈 {}: {} (Precision: {}/{})",
                        market.symbol.symbol,
                        market.status,
                        market.base_precision,
                        market.quote_precision
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed to get markets: {}", e);
        }
    }

    Ok(())
}

async fn demonstrate_config_validation() -> Result<(), Box<dyn std::error::Error>> {
    // Test different configuration scenarios
    let configs = vec![
        ("Empty credentials", ExchangeConfig::read_only()),
        (
            "Test credentials",
            ExchangeConfig::new("test".to_string(), "test".to_string()),
        ),
    ];

    for (name, config) in configs {
        println!(
            "   Testing {}: has_credentials = {}",
            name,
            config.has_credentials()
        );

        // Demonstrate safe credential checking
        if config.has_credentials() {
            println!("      ✅ Ready for authenticated operations");

            // You could create connector here
            let _connector = BinanceConnector::new(config);
            println!("      🔗 Connector created successfully");
        } else {
            println!("      📊 Limited to public operations only");
        }
    }

    Ok(())
}

async fn demonstrate_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    // Try to load from non-existent environment variables
    match ExchangeConfig::from_env("NONEXISTENT_EXCHANGE") {
        Ok(_) => {
            println!("   🤔 Unexpectedly found configuration");
        }
        Err(ConfigError::MissingEnvironmentVariable(var)) => {
            println!("   ✅ Properly caught missing variable: {}", var);
            println!("      💡 This is expected when the variable doesn't exist");
        }
        Err(e) => {
            println!("   ❓ Other error: {}", e);
        }
    }

    // Demonstrate safe operation checking
    let config = ExchangeConfig::read_only();
    if !config.has_credentials() {
        println!("   ✅ Properly detected missing credentials");
        println!("      🛡️  Application can safely handle this case");
    }

    Ok(())
}

// Utility function to show environment setup
#[allow(dead_code)]
fn show_environment_setup() {
    println!("📋 Environment Variable Setup:");
    println!("   export BINANCE_API_KEY='your_binance_api_key'");
    println!("   export BINANCE_SECRET_KEY='your_binance_secret_key'");
    println!("   export BINANCE_TESTNET='true'  # Optional, for safety");
    println!("   export BINANCE_BASE_URL='https://testnet.binance.vision'  # Optional");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_only_config() {
        let config = ExchangeConfig::read_only();
        assert!(!config.has_credentials());
    }

    #[test]
    fn test_config_with_credentials() {
        let config = ExchangeConfig::new("test_key".to_string(), "test_secret".to_string());
        assert!(config.has_credentials());
    }

    #[test]
    fn test_testnet_setting() {
        let config = ExchangeConfig::read_only().testnet(true);
        assert!(config.testnet);
    }

    #[tokio::test]
    async fn test_connector_creation() {
        let config = ExchangeConfig::read_only().testnet(true);
        let _connector = BinanceConnector::new(config);
        // Just test that creation doesn't panic
    }
}
