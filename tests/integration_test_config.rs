use lotusx::core::config::ExchangeConfig;
use std::env;

/// Test configuration utilities
pub struct TestConfig;

impl TestConfig {
    /// Check if integration tests should run (based on environment variables)
    pub fn should_run_integration_tests() -> bool {
        env::var("RUN_INTEGRATION_TESTS").unwrap_or_default() == "true"
    }

    /// Check if live API tests should run (requires real credentials)
    pub fn should_run_live_tests() -> bool {
        env::var("RUN_LIVE_TESTS").unwrap_or_default() == "true"
    }

    /// Check if order placement tests should run (very careful - requires funds)
    pub fn should_run_order_tests() -> bool {
        env::var("RUN_ORDER_TESTS").unwrap_or_default() == "true"
    }

    /// Get test timeout duration
    pub fn test_timeout_seconds() -> u64 {
        env::var("TEST_TIMEOUT_SECONDS")
            .unwrap_or_default()
            .parse()
            .unwrap_or(30)
    }

    /// Create safe test exchange config
    pub fn create_safe_config() -> ExchangeConfig {
        ExchangeConfig::new("test_api_key".to_string(), "test_secret_key".to_string()).testnet(true)
    }

    /// Try to create config from environment with fallback to safe config
    pub fn create_config_from_env(prefix: &str) -> ExchangeConfig {
        ExchangeConfig::from_env(prefix).unwrap_or_else(|_| Self::create_safe_config())
    }
}

/// Common test utilities
pub mod utils {
    use super::*;
    use std::time::Duration;

    pub fn default_timeout() -> Duration {
        Duration::from_secs(TestConfig::test_timeout_seconds())
    }

    pub fn short_timeout() -> Duration {
        Duration::from_secs(10)
    }

    pub fn long_timeout() -> Duration {
        Duration::from_secs(60)
    }

    /// Print test result with emoji
    pub fn print_test_result(test_name: &str, success: bool, message: &str) {
        let emoji = if success { "✅" } else { "❌" };
        println!("{} {}: {}", emoji, test_name, message);
    }

    /// Print warning with emoji
    pub fn print_warning(test_name: &str, message: &str) {
        println!("⚠️ {}: {}", test_name, message);
    }

    /// Check if a string represents a valid positive number
    pub fn is_valid_positive_number(s: &str) -> bool {
        s.parse::<f64>().is_ok_and(|n| n > 0.0)
    }

    /// Check if a string represents a valid non-negative number
    pub fn is_valid_non_negative_number(s: &str) -> bool {
        s.parse::<f64>().is_ok_and(|n| n >= 0.0)
    }
}

/// Test data validation utilities
pub mod validation {
    use lotusx::core::types::{Balance, Kline, Market, OrderResponse, Position};

    pub fn validate_market(market: &Market) -> Result<(), String> {
        if market.symbol.symbol.is_empty() {
            return Err("Symbol should not be empty".to_string());
        }
        if market.symbol.base.is_empty() {
            return Err("Base currency should not be empty".to_string());
        }
        if market.symbol.quote.is_empty() {
            return Err("Quote currency should not be empty".to_string());
        }
        if market.base_precision < 0 || market.base_precision > 18 {
            return Err("Base precision should be between 0 and 18".to_string());
        }
        if market.quote_precision < 0 || market.quote_precision > 18 {
            return Err("Quote precision should be between 0 and 18".to_string());
        }
        Ok(())
    }

    pub fn validate_balance(balance: &Balance) -> Result<(), String> {
        if balance.asset.is_empty() {
            return Err("Asset should not be empty".to_string());
        }
        if !super::utils::is_valid_non_negative_number(&balance.free) {
            return Err("Free balance should be a valid non-negative number".to_string());
        }
        if !super::utils::is_valid_non_negative_number(&balance.locked) {
            return Err("Locked balance should be a valid non-negative number".to_string());
        }
        Ok(())
    }

    pub fn validate_kline(kline: &Kline) -> Result<(), String> {
        if kline.symbol.is_empty() {
            return Err("Kline symbol should not be empty".to_string());
        }
        if !super::utils::is_valid_positive_number(&kline.open_price) {
            return Err("Open price should be a valid positive number".to_string());
        }
        if !super::utils::is_valid_positive_number(&kline.close_price) {
            return Err("Close price should be a valid positive number".to_string());
        }
        if !super::utils::is_valid_positive_number(&kline.high_price) {
            return Err("High price should be a valid positive number".to_string());
        }
        if !super::utils::is_valid_positive_number(&kline.low_price) {
            return Err("Low price should be a valid positive number".to_string());
        }

        // Validate price relationships
        let open: f64 = kline.open_price.parse().unwrap();
        let close: f64 = kline.close_price.parse().unwrap();
        let high: f64 = kline.high_price.parse().unwrap();
        let low: f64 = kline.low_price.parse().unwrap();

        if high < open || high < close || high < low {
            return Err("High price should be >= open, close, and low prices".to_string());
        }
        if low > open || low > close || low > high {
            return Err("Low price should be <= open, close, and high prices".to_string());
        }

        Ok(())
    }

    pub fn validate_order_response(order: &OrderResponse) -> Result<(), String> {
        if order.order_id.is_empty() {
            return Err("Order ID should not be empty".to_string());
        }
        if order.symbol.is_empty() {
            return Err("Order symbol should not be empty".to_string());
        }
        if order.quantity.is_empty() {
            return Err("Order quantity should not be empty".to_string());
        }
        if !super::utils::is_valid_positive_number(&order.quantity) {
            return Err("Order quantity should be a valid positive number".to_string());
        }
        if order.timestamp == 0 {
            return Err("Order timestamp should not be zero".to_string());
        }
        Ok(())
    }

    pub fn validate_position(position: &Position) -> Result<(), String> {
        if position.symbol.is_empty() {
            return Err("Position symbol should not be empty".to_string());
        }
        if !super::utils::is_valid_non_negative_number(&position.position_amount) {
            return Err("Position amount should be a valid non-negative number".to_string());
        }
        if !position.entry_price.is_empty()
            && !super::utils::is_valid_positive_number(&position.entry_price)
        {
            return Err("Entry price should be a valid positive number".to_string());
        }
        Ok(())
    }
}

/// Environment setup for tests
pub mod env_setup {
    use std::env;

    pub fn setup_test_environment() {
        // Set up test-specific environment variables if needed
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG", "debug");
        }
    }

    pub fn print_test_configuration() {
        println!("🔧 Test Configuration:");
        println!(
            "  RUN_INTEGRATION_TESTS: {}",
            super::TestConfig::should_run_integration_tests()
        );
        println!(
            "  RUN_LIVE_TESTS: {}",
            super::TestConfig::should_run_live_tests()
        );
        println!(
            "  RUN_ORDER_TESTS: {}",
            super::TestConfig::should_run_order_tests()
        );
        println!(
            "  TEST_TIMEOUT_SECONDS: {}",
            super::TestConfig::test_timeout_seconds()
        );
        println!();
    }

    pub fn check_credentials_available() -> Vec<String> {
        let mut available = Vec::new();

        let prefixes = vec![
            "BYBIT",
            "BYBIT_TESTNET",
            "BYBIT_PERP",
            "BYBIT_PERP_TESTNET",
            "BINANCE",
            "BINANCE_TESTNET",
            "BINANCE_PERP",
            "BINANCE_PERP_TESTNET",
        ];

        for prefix in prefixes {
            let api_key_var = format!("{}_API_KEY", prefix);
            let secret_key_var = format!("{}_SECRET_KEY", prefix);

            if env::var(&api_key_var).is_ok() && env::var(&secret_key_var).is_ok() {
                available.push(prefix.to_string());
            }
        }

        available
    }
}
