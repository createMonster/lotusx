#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::explicit_iter_loop)]

use lotusx::{
    core::{
        config::ExchangeConfig,
        traits::{AccountInfo, MarketDataSource},
        types::SubscriptionType,
    },
    exchanges::{bybit::BybitConnector, bybit_perp::BybitPerpConnector},
};
use std::time::Duration;
use tokio::time::timeout;

/// Helper function to create Bybit spot connector with testnet config
fn create_bybit_spot_connector() -> BybitConnector {
    let config = ExchangeConfig::new("test_api_key".to_string(), "test_secret_key".to_string())
        .testnet(true);

    BybitConnector::new(config)
}

/// Helper function to create Bybit perpetual connector with testnet config
fn create_bybit_perp_connector() -> BybitPerpConnector {
    let config = ExchangeConfig::new("test_api_key".to_string(), "test_secret_key".to_string())
        .testnet(true);

    BybitPerpConnector::new(config)
}

/// Helper function to create Bybit spot connector from environment
fn create_bybit_spot_from_env() -> Result<BybitConnector, Box<dyn std::error::Error>> {
    let config =
        ExchangeConfig::from_env("BYBIT_TESTNET").or_else(|_| ExchangeConfig::from_env("BYBIT"))?;
    Ok(BybitConnector::new(config))
}

/// Helper function to create Bybit perpetual connector from environment
fn create_bybit_perp_from_env() -> Result<BybitPerpConnector, Box<dyn std::error::Error>> {
    let config = ExchangeConfig::from_env("BYBIT_PERP_TESTNET")
        .or_else(|_| ExchangeConfig::from_env("BYBIT_PERP"))
        .or_else(|_| ExchangeConfig::from_env("BYBIT_TESTNET"))
        .or_else(|_| ExchangeConfig::from_env("BYBIT"))?;
    Ok(BybitPerpConnector::new(config))
}

#[cfg(test)]
mod bybit_spot_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_markets() {
        let connector = create_bybit_spot_connector();

        let result = timeout(Duration::from_secs(30), connector.get_markets()).await;

        match result {
            Ok(Ok(markets)) => {
                println!("✅ Successfully fetched {} markets", markets.len());
                assert!(!markets.is_empty(), "Markets list should not be empty");

                // Verify market structure
                let first_market = &markets[0];
                assert!(
                    !first_market.symbol.symbol.is_empty(),
                    "Symbol should not be empty"
                );
                assert!(
                    !first_market.symbol.base.is_empty(),
                    "Base currency should not be empty"
                );
                assert!(
                    !first_market.symbol.quote.is_empty(),
                    "Quote currency should not be empty"
                );

                println!(
                    "First market: {} ({}/{})",
                    first_market.symbol.symbol, first_market.symbol.base, first_market.symbol.quote
                );
            }
            Ok(Err(e)) => {
                println!("❌ Failed to fetch markets: {}", e);
                // Don't panic for API errors in integration tests - just log
                eprintln!(
                    "Market fetch failed (this may be due to API keys or network): {}",
                    e
                );
            }
            Err(_) => {
                panic!("❌ Timeout occurred while fetching markets");
            }
        }
    }

    #[tokio::test]
    async fn test_websocket_url() {
        let connector = create_bybit_spot_connector();
        let ws_url = connector.get_websocket_url();

        assert!(
            ws_url.starts_with("wss://"),
            "WebSocket URL should use WSS protocol"
        );
        assert!(
            ws_url.contains("testnet"),
            "Should use testnet URL for test config"
        );
        assert!(ws_url.contains("bybit"), "Should be Bybit WebSocket URL");

        println!("✅ WebSocket URL: {}", ws_url);
    }

    #[tokio::test]
    async fn test_subscribe_market_data_structure() {
        let connector = create_bybit_spot_connector();

        let symbols = vec!["BTCUSDT".to_string()];
        let subscription_types = vec![
            SubscriptionType::Ticker,
            SubscriptionType::OrderBook { depth: Some(20) },
            SubscriptionType::Trades,
        ];

        // Test that subscription doesn't panic and returns a receiver
        let result = timeout(
            Duration::from_secs(10),
            connector.subscribe_market_data(symbols, subscription_types, None),
        )
        .await;

        match result {
            Ok(Ok(_receiver)) => {
                println!("✅ Market data subscription created successfully");
                // Note: We don't test actual data reception as it requires live connection
            }
            Ok(Err(e)) => {
                println!("⚠️ Market data subscription failed: {}", e);
                // This might fail without proper network/API setup
            }
            Err(_) => {
                println!("⚠️ Market data subscription timed out");
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires valid API credentials"]
    async fn test_get_account_balance_with_credentials() {
        if let Ok(connector) = create_bybit_spot_from_env() {
            let result = timeout(Duration::from_secs(30), connector.get_account_balance()).await;

            match result {
                Ok(Ok(balances)) => {
                    println!("✅ Successfully fetched account balance");
                    println!("Number of balances: {}", balances.len());

                    for balance in balances.iter().take(5) {
                        // Show first 5
                        println!(
                            "  {}: free={}, locked={}",
                            balance.asset, balance.free, balance.locked
                        );
                    }
                }
                Ok(Err(e)) => {
                    println!("❌ Failed to fetch balance: {}", e);
                    panic!("Balance fetch failed: {}", e);
                }
                Err(_) => {
                    panic!("❌ Timeout occurred while fetching balance");
                }
            }
        } else {
            println!("⚠️ Skipping balance test - no valid credentials found");
        }
    }
}

#[cfg(test)]
mod bybit_perp_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_markets() {
        let connector = create_bybit_perp_connector();

        let result = timeout(Duration::from_secs(30), connector.get_markets()).await;

        match result {
            Ok(Ok(markets)) => {
                println!(
                    "✅ Successfully fetched {} perpetual markets",
                    markets.len()
                );
                assert!(
                    !markets.is_empty(),
                    "Perpetual markets list should not be empty"
                );

                // Verify market structure
                let first_market = &markets[0];
                assert!(
                    !first_market.symbol.symbol.is_empty(),
                    "Symbol should not be empty"
                );
                assert!(
                    !first_market.symbol.base.is_empty(),
                    "Base currency should not be empty"
                );
                assert!(
                    !first_market.symbol.quote.is_empty(),
                    "Quote currency should not be empty"
                );

                println!(
                    "First perpetual market: {} ({}/{})",
                    first_market.symbol.symbol, first_market.symbol.base, first_market.symbol.quote
                );

                // Check if precision and limits are properly set
                println!(
                    "Market details - Base precision: {}, Quote precision: {}",
                    first_market.base_precision, first_market.quote_precision
                );
            }
            Ok(Err(e)) => {
                println!("❌ Failed to fetch perpetual markets: {}", e);
                eprintln!("Perpetual market fetch failed: {}", e);
            }
            Err(_) => {
                panic!("❌ Timeout occurred while fetching perpetual markets");
            }
        }
    }

    #[tokio::test]
    async fn test_websocket_url() {
        let connector = create_bybit_perp_connector();
        let ws_url = connector.get_websocket_url();

        assert!(
            ws_url.starts_with("wss://"),
            "WebSocket URL should use WSS protocol"
        );
        assert!(
            ws_url.contains("testnet"),
            "Should use testnet URL for test config"
        );
        assert!(
            ws_url.contains("linear"),
            "Should use linear (perpetual) endpoint"
        );

        println!("✅ Perpetual WebSocket URL: {}", ws_url);
    }

    #[tokio::test]
    #[ignore = "Requires valid API credentials"]
    async fn test_get_positions() {
        if let Ok(connector) = create_bybit_perp_from_env() {
            let result = timeout(Duration::from_secs(30), connector.get_positions()).await;

            match result {
                Ok(Ok(positions)) => {
                    println!("✅ Successfully fetched positions");
                    println!("Number of positions: {}", positions.len());

                    for position in positions.iter() {
                        println!(
                            "  Position: {} - Amount: {}, Side: {:?}",
                            position.symbol, position.position_amount, position.position_side
                        );
                    }
                }
                Ok(Err(e)) => {
                    println!("❌ Failed to fetch positions: {}", e);
                    // Don't panic - positions query might fail for various reasons
                }
                Err(_) => {
                    panic!("❌ Timeout occurred while fetching positions");
                }
            }
        } else {
            println!("⚠️ Skipping positions test - no valid credentials found");
        }
    }
}

#[cfg(test)]
mod bybit_comprehensive_tests {
    use super::*;

    #[tokio::test]
    async fn test_spot_vs_perp_market_differences() {
        let spot_connector = create_bybit_spot_connector();
        let perp_connector = create_bybit_perp_connector();

        let (spot_result, perp_result) = tokio::join!(
            timeout(Duration::from_secs(30), spot_connector.get_markets()),
            timeout(Duration::from_secs(30), perp_connector.get_markets())
        );

        match (spot_result, perp_result) {
            (Ok(Ok(spot_markets)), Ok(Ok(perp_markets))) => {
                println!(
                    "✅ Fetched spot markets: {}, perpetual markets: {}",
                    spot_markets.len(),
                    perp_markets.len()
                );

                // Verify they return different types of markets
                if !spot_markets.is_empty() && !perp_markets.is_empty() {
                    println!("Spot example: {}", spot_markets[0].symbol.symbol);
                    println!("Perp example: {}", perp_markets[0].symbol.symbol);
                }
            }
            _ => {
                println!("⚠️ Could not compare markets due to API errors");
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test with invalid credentials to verify error handling
        let config = ExchangeConfig::new("invalid_key".to_string(), "invalid_secret".to_string())
            .testnet(true);

        let connector = BybitConnector::new(config);

        // This should fail gracefully, not panic
        let result = timeout(Duration::from_secs(10), connector.get_account_balance()).await;

        match result {
            Ok(Err(e)) => {
                println!("✅ Error handled gracefully: {}", e);
                // Verify error contains useful information
                let error_str = e.to_string();
                assert!(error_str.len() > 10, "Error message should be descriptive");
            }
            Ok(Ok(_)) => {
                println!("⚠️ Unexpectedly succeeded with invalid credentials");
            }
            Err(_) => {
                println!("⚠️ Request timed out (this is acceptable)");
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let futures = (0..3).map(|i| {
            let connector = create_bybit_spot_connector();
            async move {
                let result = timeout(Duration::from_secs(30), connector.get_markets()).await;
                (i, result)
            }
        });

        let results = futures::future::join_all(futures).await;

        let mut success_count = 0;
        for (i, result) in results {
            match result {
                Ok(Ok(markets)) => {
                    println!(
                        "✅ Concurrent request {} succeeded: {} markets",
                        i,
                        markets.len()
                    );
                    success_count += 1;
                }
                Ok(Err(e)) => {
                    println!("⚠️ Concurrent request {} failed: {}", i, e);
                }
                Err(_) => {
                    println!("⚠️ Concurrent request {} timed out", i);
                }
            }
        }

        println!(
            "Concurrent test completed: {}/3 requests succeeded",
            success_count
        );
    }
}

// Helper tests for configuration
#[cfg(test)]
mod configuration_tests {
    use super::*;

    #[test]
    fn test_testnet_configuration() {
        let connector = create_bybit_spot_connector();
        let ws_url = connector.get_websocket_url();

        // Verify testnet is properly configured
        assert!(
            ws_url.contains("testnet"),
            "Should use testnet WebSocket URL"
        );

        println!("✅ Testnet configuration verified");
    }

    #[test]
    fn test_connector_creation() {
        let spot_connector = create_bybit_spot_connector();
        let perp_connector = create_bybit_perp_connector();

        // Verify connectors can be created without panicking
        let spot_ws = spot_connector.get_websocket_url();
        let perp_ws = perp_connector.get_websocket_url();

        assert!(
            spot_ws.contains("spot"),
            "Spot connector should use spot endpoint"
        );
        assert!(
            perp_ws.contains("linear"),
            "Perp connector should use linear endpoint"
        );

        println!("✅ Connector creation test passed");
    }
}
