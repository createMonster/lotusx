#![allow(clippy::match_wild_err_arm)]
#![allow(clippy::explicit_iter_loop)]

use lotusx::{
    core::{
        config::ExchangeConfig,
        traits::{AccountInfo, MarketDataSource},
        types::SubscriptionType,
    },
    exchanges::{binance::BinanceConnector, binance_perp::BinancePerpConnector},
};
use std::time::Duration;
use tokio::time::timeout;

/// Helper function to create Binance spot connector with testnet config
fn create_binance_spot_connector() -> BinanceConnector {
    let config = ExchangeConfig::new("test_api_key".to_string(), "test_secret_key".to_string())
        .testnet(true);

    BinanceConnector::new(config)
}

/// Helper function to create Binance perpetual connector with testnet config
fn create_binance_perp_connector() -> BinancePerpConnector {
    let config = ExchangeConfig::new("test_api_key".to_string(), "test_secret_key".to_string())
        .testnet(true);

    BinancePerpConnector::new(config)
}

/// Helper function to create Binance spot connector from environment
fn create_binance_spot_from_env() -> Result<BinanceConnector, Box<dyn std::error::Error>> {
    let config = ExchangeConfig::from_env("BINANCE_TESTNET")
        .or_else(|_| ExchangeConfig::from_env("BINANCE"))?;
    Ok(BinanceConnector::new(config))
}

/// Helper function to create Binance perpetual connector from environment
fn create_binance_perp_from_env() -> Result<BinancePerpConnector, Box<dyn std::error::Error>> {
    let config = ExchangeConfig::from_env("BINANCE_PERP_TESTNET")
        .or_else(|_| ExchangeConfig::from_env("BINANCE_PERP"))
        .or_else(|_| ExchangeConfig::from_env("BINANCE_TESTNET"))
        .or_else(|_| ExchangeConfig::from_env("BINANCE"))?;
    Ok(BinancePerpConnector::new(config))
}

#[cfg(test)]
mod binance_spot_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_markets() {
        let connector = create_binance_spot_connector();

        let result = timeout(Duration::from_secs(30), connector.get_markets()).await;

        match result {
            Ok(Ok(markets)) => {
                println!("✅ Successfully fetched {} Binance markets", markets.len());
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

                // Check precision settings
                println!(
                    "Precision - Base: {}, Quote: {}",
                    first_market.base_precision, first_market.quote_precision
                );

                // Verify trading limits if available
                if let Some(min_qty) = &first_market.min_qty {
                    println!("Min quantity: {}", min_qty);
                }
                if let Some(min_price) = &first_market.min_price {
                    println!("Min price: {}", min_price);
                }
            }
            Ok(Err(e)) => {
                println!("❌ Failed to fetch Binance markets: {}", e);
                eprintln!("Binance market fetch failed: {}", e);
            }
            Err(_) => {
                panic!("❌ Timeout occurred while fetching Binance markets");
            }
        }
    }

    #[tokio::test]
    async fn test_websocket_url() {
        let connector = create_binance_spot_connector();
        let ws_url = connector.get_websocket_url();

        assert!(
            ws_url.starts_with("wss://"),
            "WebSocket URL should use WSS protocol"
        );
        assert!(
            ws_url.contains("binance"),
            "Should be Binance WebSocket URL"
        );

        println!("✅ Binance WebSocket URL: {}", ws_url);
    }

    #[tokio::test]
    async fn test_klines_functionality() {
        let connector = create_binance_spot_connector();

        let result = timeout(
            Duration::from_secs(30),
            connector.get_klines(
                "BTCUSDT".to_string(),
                "1m".to_string(),
                Some(10),
                None,
                None,
            ),
        )
        .await;

        match result {
            Ok(Ok(klines)) => {
                println!("✅ Successfully fetched {} klines", klines.len());
                assert!(!klines.is_empty(), "Klines should not be empty");

                let first_kline = &klines[0];
                assert!(
                    !first_kline.open_price.is_empty(),
                    "Open price should not be empty"
                );
                assert!(
                    !first_kline.close_price.is_empty(),
                    "Close price should not be empty"
                );
                assert!(
                    !first_kline.high_price.is_empty(),
                    "High price should not be empty"
                );
                assert!(
                    !first_kline.low_price.is_empty(),
                    "Low price should not be empty"
                );

                println!(
                    "First kline: O:{} H:{} L:{} C:{}",
                    first_kline.open_price,
                    first_kline.high_price,
                    first_kline.low_price,
                    first_kline.close_price
                );
            }
            Ok(Err(e)) => {
                println!("❌ Failed to fetch klines: {}", e);
                eprintln!("Klines fetch failed: {}", e);
            }
            Err(_) => {
                panic!("❌ Timeout occurred while fetching klines");
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_market_data_structure() {
        let connector = create_binance_spot_connector();

        let symbols = vec!["btcusdt".to_string(), "ethusdt".to_string()];
        let subscription_types = vec![
            SubscriptionType::Ticker,
            SubscriptionType::OrderBook { depth: Some(10) },
            SubscriptionType::Trades,
            SubscriptionType::Klines {
                interval: "1m".to_string(),
            },
        ];

        let result = timeout(
            Duration::from_secs(10),
            connector.subscribe_market_data(symbols, subscription_types, None),
        )
        .await;

        match result {
            Ok(Ok(_receiver)) => {
                println!("✅ Binance market data subscription created successfully");
            }
            Ok(Err(e)) => {
                println!("⚠️ Binance market data subscription failed: {}", e);
            }
            Err(_) => {
                println!("⚠️ Binance market data subscription timed out");
            }
        }
    }

    #[tokio::test]
    #[ignore = "Requires valid API credentials"]
    async fn test_get_account_balance_with_credentials() {
        if let Ok(connector) = create_binance_spot_from_env() {
            let result = timeout(Duration::from_secs(30), connector.get_account_balance()).await;

            match result {
                Ok(Ok(balances)) => {
                    println!("✅ Successfully fetched Binance account balance");
                    println!("Number of balances: {}", balances.len());

                    // Show non-zero balances
                    let non_zero_balances: Vec<_> = balances
                        .iter()
                        .filter(|b| {
                            b.free.parse::<f64>().unwrap_or(0.0) > 0.0
                                || b.locked.parse::<f64>().unwrap_or(0.0) > 0.0
                        })
                        .collect();

                    println!("Non-zero balances: {}", non_zero_balances.len());
                    for balance in non_zero_balances.iter().take(5) {
                        println!(
                            "  {}: free={}, locked={}",
                            balance.asset, balance.free, balance.locked
                        );
                    }
                }
                Ok(Err(e)) => {
                    println!("❌ Failed to fetch Binance balance: {}", e);
                    panic!("Binance balance fetch failed: {}", e);
                }
                Err(_) => {
                    panic!("❌ Timeout occurred while fetching Binance balance");
                }
            }
        } else {
            println!("⚠️ Skipping Binance balance test - no valid credentials found");
        }
    }
}

#[cfg(test)]
mod binance_perp_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_markets() {
        let connector = create_binance_perp_connector();

        let result = timeout(Duration::from_secs(30), connector.get_markets()).await;

        match result {
            Ok(Ok(markets)) => {
                println!(
                    "✅ Successfully fetched {} Binance perpetual markets",
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

                // Check precision and limits
                println!(
                    "Market details - Base precision: {}, Quote precision: {}",
                    first_market.base_precision, first_market.quote_precision
                );

                if let Some(min_qty) = &first_market.min_qty {
                    println!("Min quantity: {}", min_qty);
                }
            }
            Ok(Err(e)) => {
                println!("❌ Failed to fetch Binance perpetual markets: {}", e);
                eprintln!("Binance perpetual market fetch failed: {}", e);
            }
            Err(_) => {
                panic!("❌ Timeout occurred while fetching Binance perpetual markets");
            }
        }
    }

    #[tokio::test]
    async fn test_websocket_url() {
        let connector = create_binance_perp_connector();
        let ws_url = connector.get_websocket_url();

        assert!(
            ws_url.starts_with("wss://"),
            "WebSocket URL should use WSS protocol"
        );
        assert!(
            ws_url.contains("binance"),
            "Should be Binance WebSocket URL"
        );

        println!("✅ Binance Perpetual WebSocket URL: {}", ws_url);
    }

    #[tokio::test]
    #[ignore = "Requires valid API credentials"]
    async fn test_get_positions() {
        if let Ok(connector) = create_binance_perp_from_env() {
            let result = timeout(Duration::from_secs(30), connector.get_positions()).await;

            match result {
                Ok(Ok(positions)) => {
                    println!("✅ Successfully fetched Binance positions");
                    println!("Number of positions: {}", positions.len());

                    // Show non-zero positions
                    let active_positions: Vec<_> = positions
                        .iter()
                        .filter(|p| p.position_amount.parse::<f64>().unwrap_or(0.0).abs() > 0.0)
                        .collect();

                    println!("Active positions: {}", active_positions.len());
                    for position in active_positions.iter().take(5) {
                        println!(
                            "  {}: amount={}, side={:?}, entry_price={}",
                            position.symbol,
                            position.position_amount,
                            position.position_side,
                            position.entry_price
                        );
                    }
                }
                Ok(Err(e)) => {
                    println!("❌ Failed to fetch Binance positions: {}", e);
                }
                Err(_) => {
                    panic!("❌ Timeout occurred while fetching Binance positions");
                }
            }
        } else {
            println!("⚠️ Skipping Binance positions test - no valid credentials found");
        }
    }
}

#[cfg(test)]
mod binance_comprehensive_tests {
    use super::*;

    #[tokio::test]
    async fn test_spot_vs_perp_differences() {
        let spot_connector = create_binance_spot_connector();
        let perp_connector = create_binance_perp_connector();

        let (spot_result, perp_result) = tokio::join!(
            timeout(Duration::from_secs(30), spot_connector.get_markets()),
            timeout(Duration::from_secs(30), perp_connector.get_markets())
        );

        match (spot_result, perp_result) {
            (Ok(Ok(spot_markets)), Ok(Ok(perp_markets))) => {
                println!(
                    "✅ Fetched Binance markets - Spot: {}, Perpetual: {}",
                    spot_markets.len(),
                    perp_markets.len()
                );

                // Compare WebSocket URLs
                let spot_ws = spot_connector.get_websocket_url();
                let perp_ws = perp_connector.get_websocket_url();

                assert_ne!(
                    spot_ws, perp_ws,
                    "Spot and perpetual should have different WebSocket URLs"
                );

                println!("Spot WS: {}", spot_ws);
                println!("Perp WS: {}", perp_ws);

                // Verify market symbol formats
                if !spot_markets.is_empty() && !perp_markets.is_empty() {
                    println!("Spot symbol example: {}", spot_markets[0].symbol.symbol);
                    println!("Perp symbol example: {}", perp_markets[0].symbol.symbol);
                }
            }
            _ => {
                println!("⚠️ Could not compare Binance markets due to API errors");
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling_with_bad_credentials() {
        // Test with completely invalid credentials
        let config = ExchangeConfig::new(
            "invalid_binance_key".to_string(),
            "invalid_binance_secret".to_string(),
        )
        .testnet(true);

        let connector = BinanceConnector::new(config);

        let result = timeout(Duration::from_secs(15), connector.get_account_balance()).await;

        match result {
            Ok(Err(e)) => {
                println!("✅ Binance error handled gracefully: {}", e);
                let error_str = e.to_string();
                assert!(error_str.len() > 5, "Error message should be descriptive");
                // Binance typically returns specific error codes
                assert!(
                    error_str.contains("API")
                        || error_str.contains("signature")
                        || error_str.contains("key")
                        || error_str.contains("auth"),
                    "Error should indicate authentication issue"
                );
            }
            Ok(Ok(_)) => {
                println!("⚠️ Unexpectedly succeeded with invalid Binance credentials");
            }
            Err(_) => {
                println!("⚠️ Binance request timed out");
            }
        }
    }

    #[tokio::test]
    async fn test_market_data_parsing() {
        let connector = create_binance_spot_connector();

        let result = timeout(Duration::from_secs(30), connector.get_markets()).await;

        if let Ok(Ok(markets)) = result {
            if !markets.is_empty() {
                let market = &markets[0];

                // Test that precision values are reasonable
                assert!(
                    market.base_precision >= 0 && market.base_precision <= 18,
                    "Base precision should be reasonable"
                );
                assert!(
                    market.quote_precision >= 0 && market.quote_precision <= 18,
                    "Quote precision should be reasonable"
                );

                // Test symbol structure
                assert!(
                    !market.symbol.base.is_empty(),
                    "Base currency should not be empty"
                );
                assert!(
                    !market.symbol.quote.is_empty(),
                    "Quote currency should not be empty"
                );
                assert_eq!(
                    market.symbol.symbol,
                    format!("{}{}", market.symbol.base, market.symbol.quote),
                    "Symbol should be base+quote concatenation"
                );

                println!("✅ Market data parsing validation passed");
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_binance_requests() {
        let futures = (0..5).map(|i| {
            let connector = create_binance_spot_connector();
            async move {
                let result = timeout(Duration::from_secs(30), connector.get_markets()).await;
                (i, result)
            }
        });

        let results = futures::future::join_all(futures).await;

        let mut success_count = 0;
        let mut error_count = 0;

        for (i, result) in results {
            match result {
                Ok(Ok(markets)) => {
                    println!(
                        "✅ Binance concurrent request {} succeeded: {} markets",
                        i,
                        markets.len()
                    );
                    success_count += 1;
                }
                Ok(Err(e)) => {
                    println!("⚠️ Binance concurrent request {} failed: {}", i, e);
                    error_count += 1;
                }
                Err(_) => {
                    println!("⚠️ Binance concurrent request {} timed out", i);
                    error_count += 1;
                }
            }
        }

        println!(
            "Binance concurrent test: {}/5 succeeded, {}/5 failed",
            success_count, error_count
        );
    }

    #[tokio::test]
    async fn test_klines_data_quality() {
        let connector = create_binance_spot_connector();

        let result = timeout(
            Duration::from_secs(30),
            connector.get_klines("BTCUSDT".to_string(), "1h".to_string(), Some(5), None, None),
        )
        .await;

        if let Ok(Ok(klines)) = result {
            assert!(!klines.is_empty(), "Should return klines data");

            for (i, kline) in klines.iter().enumerate() {
                // Validate kline data structure
                assert!(
                    !kline.open_price.is_empty(),
                    "Open price should not be empty"
                );
                assert!(
                    !kline.close_price.is_empty(),
                    "Close price should not be empty"
                );
                assert!(
                    !kline.high_price.is_empty(),
                    "High price should not be empty"
                );
                assert!(!kline.low_price.is_empty(), "Low price should not be empty");
                assert!(!kline.volume.is_empty(), "Volume should not be empty");

                // Validate price relationships
                let open: f64 = kline.open_price.parse().unwrap_or(0.0);
                let close: f64 = kline.close_price.parse().unwrap_or(0.0);
                let high: f64 = kline.high_price.parse().unwrap_or(0.0);
                let low: f64 = kline.low_price.parse().unwrap_or(0.0);

                assert!(
                    high >= open && high >= close && high >= low,
                    "High should be >= open, close, low"
                );
                assert!(
                    low <= open && low <= close && low <= high,
                    "Low should be <= open, close, high"
                );
                assert!(
                    open > 0.0 && close > 0.0 && high > 0.0 && low > 0.0,
                    "All prices should be positive"
                );

                if i == 0 {
                    println!(
                        "✅ Kline data quality check passed - O:{} H:{} L:{} C:{}",
                        open, high, low, close
                    );
                }
            }

            println!("✅ All {} klines passed quality validation", klines.len());
        } else {
            println!("⚠️ Could not validate klines data quality");
        }
    }
}

// Configuration and setup tests
#[cfg(test)]
mod binance_config_tests {
    use super::*;

    #[test]
    fn test_binance_testnet_urls() {
        let spot_connector = create_binance_spot_connector();
        let perp_connector = create_binance_perp_connector();

        let spot_ws = spot_connector.get_websocket_url();
        let perp_ws = perp_connector.get_websocket_url();

        // Verify WebSocket URLs are properly configured
        assert!(spot_ws.starts_with("wss://"), "Spot WS should use WSS");
        assert!(perp_ws.starts_with("wss://"), "Perp WS should use WSS");

        // Both should be valid URLs
        assert!(spot_ws.len() > 10, "Spot WS URL should be valid");
        assert!(perp_ws.len() > 10, "Perp WS URL should be valid");

        println!("✅ Binance URL configuration test passed");
        println!("  Spot WS: {}", spot_ws);
        println!("  Perp WS: {}", perp_ws);
    }

    #[test]
    fn test_binance_connector_creation() {
        // Test that connectors can be created with various configurations
        let configs = vec![
            ExchangeConfig::new("test".to_string(), "test".to_string()).testnet(true),
            ExchangeConfig::new("test".to_string(), "test".to_string()).testnet(false),
        ];

        for (i, config) in configs.into_iter().enumerate() {
            let spot = BinanceConnector::new(config.clone());
            let perp = BinancePerpConnector::new(config);

            // Should not panic during creation
            let _spot_ws = spot.get_websocket_url();
            let _perp_ws = perp.get_websocket_url();

            println!("✅ Binance connector creation test {} passed", i);
        }
    }
}
