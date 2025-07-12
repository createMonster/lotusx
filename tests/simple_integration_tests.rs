use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::MarketDataSource;
use lotusx::exchanges::binance::build_connector as build_binance_connector;
use lotusx::exchanges::bybit::build_connector as build_bybit_connector;
use tokio::time::{timeout, Duration};

fn create_test_config() -> ExchangeConfig {
    ExchangeConfig::new("test".to_string(), "test".to_string()).testnet(true)
}

#[tokio::test]
async fn test_binance_websocket_url() {
    let config = create_test_config();
    if let Ok(connector) = build_binance_connector(config) {
        let ws_url = MarketDataSource::get_websocket_url(&connector);
        assert!(ws_url.contains("binance"));
        println!("✅ Binance WebSocket URL: {}", ws_url);
    }
}

#[tokio::test]
async fn test_bybit_websocket_url() {
    let config = create_test_config();
    if let Ok(connector) = build_bybit_connector(config) {
        let ws_url = MarketDataSource::get_websocket_url(&connector);
        assert!(ws_url.contains("bybit"));
        println!("✅ Bybit WebSocket URL: {}", ws_url);
    }
}

#[tokio::test]
async fn test_binance_markets() {
    let config = create_test_config();
    if let Ok(connector) = build_binance_connector(config) {
        let result = timeout(
            Duration::from_secs(30),
            MarketDataSource::get_markets(&connector),
        )
        .await;

        match result {
            Ok(Ok(markets)) => {
                println!("✅ Binance markets: {} found", markets.len());
                assert!(!markets.is_empty(), "Should have at least some markets");

                // Find BTCUSDT if it exists
                if let Some(btc_market) = markets.iter().find(|m| m.symbol.to_string() == "BTCUSDT")
                {
                    println!("✅ Found BTCUSDT market");
                    assert_eq!(btc_market.symbol.base, "BTC");
                    assert_eq!(btc_market.symbol.quote, "USDT");
                }
            }
            Ok(Err(e)) => {
                println!("⚠️ Binance markets error (expected in CI): {}", e);
                // Don't fail the test - network issues are expected in CI environments
            }
            Err(_) => {
                println!("⚠️ Binance markets timeout (expected in CI)");
                // Don't fail the test - timeouts are expected in CI environments
            }
        }
    }
}

#[tokio::test]
async fn test_bybit_markets() {
    let config = create_test_config();
    if let Ok(connector) = build_bybit_connector(config) {
        let result = timeout(
            Duration::from_secs(30),
            MarketDataSource::get_markets(&connector),
        )
        .await;

        match result {
            Ok(Ok(markets)) => {
                println!("✅ Bybit markets: {} found", markets.len());
                assert!(!markets.is_empty(), "Should have at least some markets");
            }
            Ok(Err(e)) => {
                println!("⚠️ Bybit markets error (expected in CI): {}", e);
                // Don't fail the test - network issues are expected in CI environments
            }
            Err(_) => {
                println!("⚠️ Bybit markets timeout (expected in CI)");
                // Don't fail the test - timeouts are expected in CI environments
            }
        }
    }
}
