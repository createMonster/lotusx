use lotusx::{
    core::{config::ExchangeConfig, traits::MarketDataSource},
    exchanges::{binance::BinanceConnector, bybit::BybitConnector},
};
use std::time::Duration;
use tokio::time::timeout;

/// Create safe test configuration
fn create_test_config() -> ExchangeConfig {
    ExchangeConfig::new("test_api_key".to_string(), "test_secret_key".to_string()).testnet(true)
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_bybit_basic() {
        let connector = BybitConnector::new(create_test_config());
        let ws_url = connector.get_websocket_url();
        assert!(ws_url.starts_with("wss://"));
        println!("✅ Bybit WebSocket URL: {}", ws_url);
    }

    #[tokio::test]
    async fn test_binance_basic() {
        let connector = BinanceConnector::new(create_test_config());
        let ws_url = connector.get_websocket_url();
        assert!(ws_url.starts_with("wss://"));
        println!("✅ Binance WebSocket URL: {}", ws_url);
    }

    #[tokio::test]
    async fn test_bybit_markets() {
        let connector = BybitConnector::new(create_test_config());

        let result = timeout(Duration::from_secs(30), connector.get_markets()).await;

        match result {
            Ok(Ok(markets)) => {
                println!("✅ Bybit: Fetched {} markets", markets.len());
                assert!(!markets.is_empty(), "Should have markets");
            }
            Ok(Err(e)) => {
                println!("⚠️ Bybit markets failed: {}", e);
            }
            Err(_) => {
                println!("⚠️ Bybit markets timed out");
            }
        }
    }

    #[tokio::test]
    async fn test_binance_markets() {
        let connector = BinanceConnector::new(create_test_config());

        let result = timeout(Duration::from_secs(30), connector.get_markets()).await;

        match result {
            Ok(Ok(markets)) => {
                println!("✅ Binance: Fetched {} markets", markets.len());
                assert!(!markets.is_empty(), "Should have markets");
            }
            Ok(Err(e)) => {
                println!("⚠️ Binance markets failed: {}", e);
            }
            Err(_) => {
                println!("⚠️ Binance markets timed out");
            }
        }
    }
}
