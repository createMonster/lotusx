use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::MarketDataSource;

#[cfg(feature = "env-file")]
use lotusx::{
    core::{config::ConfigError},
    exchanges::binance::BinanceConnector,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 LotuSX .env File Configuration Examples");
    println!("==========================================\n");

    // Example 1: Basic .env file loading
    println!("1. 📄 Basic .env File Loading:");

    #[cfg(feature = "env-file")]
    {
        match ExchangeConfig::from_env_file("BINANCE") {
            Ok(config) => {
                println!("   ✅ Configuration loaded from .env file");
                println!("   🔍 Has credentials: {}", config.has_credentials());
                println!("   🧪 Testnet mode: {}", config.testnet);

                if config.has_credentials() {
                    let connector = BinanceConnector::new(config);
                    demo_with_connector(&connector).await?;
                }
            }
            Err(ConfigError::MissingEnvironmentVariable(var)) => {
                println!("   ⚠️  Missing variable in .env file: {}", var);
                println!("   💡 Add '{}=your_value' to your .env file", var);
            }
            Err(e) => {
                println!("   ❌ Error loading from .env file: {}", e);
            }
        }
    }

    #[cfg(not(feature = "env-file"))]
    {
        println!("   ⚠️  .env file support not enabled");
        println!("   💡 Enable with: cargo run --features env-file");
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 2: Custom .env file path
    println!("2. 🎯 Custom .env File Path:");

    #[cfg(feature = "env-file")]
    {
        // Try loading from different .env files
        let env_files = [".env.development", ".env.local", ".env"];

        for env_file in &env_files {
            println!("   Trying: {}", env_file);
            match ExchangeConfig::from_env_file_with_path("BINANCE", env_file) {
                Ok(config) => {
                    println!("   ✅ Loaded from {}", env_file);
                    println!("   🔍 Has credentials: {}", config.has_credentials());
                    break;
                }
                Err(ConfigError::MissingEnvironmentVariable(var)) => {
                    println!("   ⚠️  Missing variable '{}' in {}", var, env_file);
                }
                Err(e) => {
                    println!("   ❌ Could not load from {}: {}", env_file, e);
                }
            }
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 3: Automatic .env file detection
    println!("3. 🔍 Automatic .env File Detection:");

    #[cfg(feature = "env-file")]
    {
        match ExchangeConfig::from_env_auto("BINANCE") {
            Ok(config) => {
                println!("   ✅ Configuration loaded automatically");
                println!("   🔍 Has credentials: {}", config.has_credentials());
                println!("   🧪 Testnet mode: {}", config.testnet);
            }
            Err(e) => {
                println!("   ❌ Auto-detection failed: {}", e);
            }
        }
    }

    println!("\n{}\n", "=".repeat(50));

    // Example 4: Fallback behavior
    println!("4. 🔄 Fallback Behavior:");
    demonstrate_fallback_behavior().await?;

    println!("\n{}\n", "=".repeat(50));

    // Example 5: Security best practices
    println!("5. 🛡️  Security Best Practices:");
    demonstrate_security_practices();

    println!("\n🎉 All .env file examples completed!");
    Ok(())
}

#[cfg(feature = "env-file")]
async fn demo_with_connector(
    connector: &BinanceConnector,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("   🔗 Testing connector...");

    match connector.get_markets().await {
        Ok(markets) => {
            println!("   📈 Successfully retrieved {} markets", markets.len());

            // Show a few example markets
            for market in markets.iter().take(3) {
                println!("      - {} ({})", market.symbol.symbol, market.status);
            }
        }
        Err(e) => {
            println!("   ❌ Failed to get markets: {}", e);
        }
    }

    Ok(())
}

async fn demonstrate_fallback_behavior() -> Result<(), Box<dyn std::error::Error>> {
    println!("   Testing fallback from .env to system environment variables...");

    // This will try .env first, then fall back to system env vars
    #[cfg(feature = "env-file")]
    {
        match ExchangeConfig::from_env_file("BINANCE") {
            Ok(_config) => {
                println!("   ✅ Loaded from .env file or environment variables");
            }
            Err(ConfigError::MissingEnvironmentVariable(var)) => {
                println!("   ⚠️  Variable '{}' not found in .env or environment", var);
                println!("   💡 Set it in .env file or export {}=your_value", var);
            }
            Err(e) => {
                println!("   ❌ Configuration error: {}", e);
            }
        }
    }

    // Direct environment variable loading (no .env file)
    println!("   Testing direct environment variable loading...");
    match ExchangeConfig::from_env("BINANCE") {
        Ok(_config) => {
            println!("   ✅ Loaded directly from environment variables");
        }
        Err(e) => {
            println!("   ⚠️  Direct environment loading failed: {}", e);
        }
    }

    Ok(())
}

fn demonstrate_security_practices() {
    println!("   📋 Security Checklist for .env Files:");
    println!();

    println!("   ✅ DO:");
    println!("      • Add .env* to your .gitignore file");
    println!("      • Use different .env files for different environments");
    println!("      • Keep .env files in the project root (not in subdirectories)");
    println!("      • Use strong, unique API keys for each environment");
    println!("      • Set restrictive file permissions (chmod 600 .env)");
    println!();

    println!("   ❌ DON'T:");
    println!("      • Commit .env files to version control");
    println!("      • Share .env files via email or chat");
    println!("      • Use production credentials in development .env files");
    println!("      • Store .env files in public directories");
    println!("      • Use the same credentials across multiple projects");
    println!();

    println!("   📝 Example .gitignore entries:");
    println!("      .env");
    println!("      .env.*");
    println!("      !.env.example  # This is safe to commit");
    println!();

    println!("   📄 Example .env.example file (safe to commit):");
    println!("      # Copy this to .env and fill in your actual values");
    println!("      BINANCE_API_KEY=your_binance_api_key_here");
    println!("      BINANCE_SECRET_KEY=your_binance_secret_key_here");
    println!("      BINANCE_TESTNET=true");
    println!("      BINANCE_BASE_URL=https://testnet.binance.vision");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_file_feature_availability() {
        // Test that the feature flag is working correctly
        #[cfg(feature = "env-file")]
        {
            // If env-file feature is enabled, these methods should be available
            let _result = ExchangeConfig::from_env_file("TEST");
            let _result = ExchangeConfig::from_env_file_with_path("TEST", ".env.test");
            let _result = ExchangeConfig::from_env_auto("TEST");
        }

        #[cfg(not(feature = "env-file"))]
        {
            // If env-file feature is not enabled, we can still use from_env
            let _result = ExchangeConfig::from_env("TEST");
        }
    }

    #[tokio::test]
    async fn test_fallback_to_regular_env() {
        // Test that the system still works without .env files
        match ExchangeConfig::from_env("NONEXISTENT") {
            Ok(_) => panic!("Should have failed with missing environment variable"),
            Err(ConfigError::MissingEnvironmentVariable(_)) => {
                // This is expected
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
}
