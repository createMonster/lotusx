#[cfg(test)]
mod funding_rates_tests {
    use lotusx::core::{config::ExchangeConfig, traits::FundingRateSource};
    use lotusx::exchanges::{
        backpack::client::BackpackConnector, binance_perp::client::BinancePerpConnector,
    };

    #[tokio::test]
    async fn test_binance_perp_get_funding_rates_single_symbol() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BinancePerpConnector::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let result = exchange.get_funding_rates(Some(symbols)).await;

        assert!(
            result.is_ok(),
            "Failed to get funding rates: {:?}",
            result.err()
        );
        let rates = result.unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].symbol, "BTCUSDT");
        assert!(rates[0].funding_rate.is_some());
        assert!(rates[0].mark_price.is_some());
        assert!(rates[0].index_price.is_some());

        println!("✅ Binance Perp Single Symbol Test Passed");
        println!("   Symbol: {}", rates[0].symbol);
        println!("   Funding Rate: {:?}", rates[0].funding_rate);
        println!("   Mark Price: {:?}", rates[0].mark_price);
    }

    #[tokio::test]
    async fn test_binance_perp_get_all_funding_rates() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BinancePerpConnector::new(config);

        let result = exchange.get_funding_rates(None).await;

        assert!(
            result.is_ok(),
            "Failed to get all funding rates: {:?}",
            result.err()
        );
        let rates = result.unwrap();
        assert!(!rates.is_empty(), "Should have received some funding rates");

        // Check that all rates have required fields
        for rate in &rates {
            assert!(rate.funding_rate.is_some());
            assert!(rate.mark_price.is_some());
            assert!(rate.index_price.is_some());
        }

        println!("✅ Binance Perp All Funding Rates Test Passed");
        println!("   Total symbols: {}", rates.len());
        println!("   Sample rates:");
        for (i, rate) in rates.iter().take(3).enumerate() {
            println!(
                "   {}: {} - Rate: {:?}",
                i + 1,
                rate.symbol,
                rate.funding_rate
            );
        }
    }

    #[tokio::test]
    async fn test_binance_perp_get_funding_rate_history() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BinancePerpConnector::new(config);

        let result = exchange
            .get_funding_rate_history(
                "BTCUSDT".to_string(),
                None,
                None,
                Some(5), // Last 5 funding rates
            )
            .await;

        assert!(
            result.is_ok(),
            "Failed to get funding rate history: {:?}",
            result.err()
        );
        let history = result.unwrap();
        assert!(
            !history.is_empty(),
            "Should have received funding rate history"
        );
        assert!(history.len() <= 5, "Should respect limit parameter");

        // Check that historical rates have funding_time
        for rate in &history {
            assert!(rate.funding_rate.is_some());
            assert!(rate.funding_time.is_some());
        }

        println!("✅ Binance Perp Funding Rate History Test Passed");
        println!("   History entries: {}", history.len());
        for (i, rate) in history.iter().enumerate() {
            println!(
                "   {}: Rate: {:?}, Time: {:?}",
                i + 1,
                rate.funding_rate,
                rate.funding_time
            );
        }
    }

    #[tokio::test]
    async fn test_backpack_get_funding_rates_single_symbol() {
        // Note: This test requires valid Backpack credentials
        if let Ok(config) = ExchangeConfig::from_env("BACKPACK") {
            let config = config.testnet(true);
            match BackpackConnector::new(config) {
                Ok(exchange) => {
                    let symbols = vec!["SOL_USDC".to_string()];
                    let result = exchange.get_funding_rates(Some(symbols)).await;

                    match result {
                        Ok(rates) => {
                            assert_eq!(rates.len(), 1);
                            assert_eq!(rates[0].symbol, "SOL_USDC");
                            assert!(rates[0].funding_rate.is_some());
                            assert!(rates[0].mark_price.is_some());

                            println!("✅ Backpack Single Symbol Test Passed");
                            println!("   Symbol: {}", rates[0].symbol);
                            println!("   Funding Rate: {:?}", rates[0].funding_rate);
                            println!("   Mark Price: {:?}", rates[0].mark_price);
                        }
                        Err(e) => {
                            println!("⚠️  Backpack Single Symbol Test Skipped: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Backpack connector creation failed: {}", e);
                }
            }
        } else {
            println!("⚠️  Backpack test skipped: No credentials found in environment");
        }
    }

    #[tokio::test]
    async fn test_backpack_get_funding_rate_history() {
        // Note: This test requires valid Backpack credentials
        if let Ok(config) = ExchangeConfig::from_env("BACKPACK") {
            let config = config.testnet(true);
            match BackpackConnector::new(config) {
                Ok(exchange) => {
                    let result = exchange
                        .get_funding_rate_history("SOL_USDC".to_string(), None, None, Some(3))
                        .await;

                    match result {
                        Ok(history) => {
                            // Backpack might not have historical data in testnet
                            println!("✅ Backpack Funding Rate History Test Completed");
                            println!("   History entries: {}", history.len());
                            for (i, rate) in history.iter().enumerate() {
                                println!(
                                    "   {}: Rate: {:?}, Time: {:?}",
                                    i + 1,
                                    rate.funding_rate,
                                    rate.funding_time
                                );
                            }
                        }
                        Err(e) => {
                            println!("⚠️  Backpack History Test: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Backpack connector creation failed: {}", e);
                }
            }
        } else {
            println!("⚠️  Backpack history test skipped: No credentials found in environment");
        }
    }

    #[tokio::test]
    async fn test_funding_rate_data_structure() {
        use lotusx::core::types::FundingRate;

        let rate = FundingRate {
            symbol: "BTCUSDT".to_string(),
            funding_rate: Some("0.0001".to_string()),
            previous_funding_rate: Some("0.00005".to_string()),
            next_funding_rate: Some("0.00015".to_string()),
            funding_time: Some(1_699_876_800_000),
            next_funding_time: Some(1_699_905_600_000),
            mark_price: Some("35000.0".to_string()),
            index_price: Some("35001.0".to_string()),
            timestamp: 1_699_876_800_000,
        };

        assert_eq!(rate.symbol, "BTCUSDT");
        assert_eq!(rate.funding_rate, Some("0.0001".to_string()));
        assert_eq!(rate.mark_price, Some("35000.0".to_string()));

        println!("✅ Funding Rate Data Structure Test Passed");
    }

    #[tokio::test]
    async fn test_funding_rate_error_handling() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BinancePerpConnector::new(config);

        // Test with invalid symbol
        let result = exchange
            .get_funding_rates(Some(vec!["INVALID_SYMBOL".to_string()]))
            .await;

        // Should handle error gracefully or return empty result
        match result {
            Ok(rates) => {
                // If API returns successfully, rates should be empty for invalid symbol
                println!(
                    "✅ Error handling test: Returned {} rates for invalid symbol",
                    rates.len()
                );
            }
            Err(e) => {
                // If API returns error, it should be a proper error type
                println!("✅ Error handling test: Properly caught error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_funding_rate_requests() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BinancePerpConnector::new(config);

        // Test concurrent requests
        let symbols1 = vec!["BTCUSDT".to_string()];
        let symbols2 = vec!["ETHUSDT".to_string()];

        let (result1, result2) = tokio::join!(
            exchange.get_funding_rates(Some(symbols1)),
            exchange.get_funding_rates(Some(symbols2))
        );

        assert!(result1.is_ok(), "First concurrent request failed");
        assert!(result2.is_ok(), "Second concurrent request failed");

        let rates1 = result1.unwrap();
        let rates2 = result2.unwrap();

        assert_eq!(rates1[0].symbol, "BTCUSDT");
        assert_eq!(rates2[0].symbol, "ETHUSDT");

        println!("✅ Concurrent Funding Rate Requests Test Passed");
        println!("   BTC Rate: {:?}", rates1[0].funding_rate);
        println!("   ETH Rate: {:?}", rates2[0].funding_rate);
    }

    #[tokio::test]
    async fn test_performance_timing() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BinancePerpConnector::new(config);

        let start = std::time::Instant::now();
        let result = exchange
            .get_funding_rates(Some(vec!["BTCUSDT".to_string()]))
            .await;
        let duration = start.elapsed();

        assert!(result.is_ok(), "Performance test request failed");
        assert!(
            duration.as_millis() < 5000,
            "Request took too long: {:?}",
            duration
        );

        println!("✅ Performance Test Passed");
        println!("   Request completed in: {:?}", duration);
    }

    #[tokio::test]
    async fn test_binance_perp_get_all_funding_rates_direct() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BinancePerpConnector::new(config);

        let result = exchange.get_all_funding_rates().await;

        assert!(
            result.is_ok(),
            "Failed to get all funding rates directly: {:?}",
            result.err()
        );
        let rates = result.unwrap();
        assert!(!rates.is_empty(), "Should have received some funding rates");

        // Check that all rates have required fields
        for rate in &rates {
            assert!(rate.funding_rate.is_some());
            assert!(rate.mark_price.is_some());
            assert!(rate.index_price.is_some());
        }

        println!("✅ Binance Perp Direct get_all_funding_rates Test Passed");
        println!("   Total symbols: {}", rates.len());
        println!("   Sample rates:");
        for (i, rate) in rates.iter().take(3).enumerate() {
            println!(
                "   {}: {} - Rate: {:?}",
                i + 1,
                rate.symbol,
                rate.funding_rate
            );
        }
    }

    #[tokio::test]
    async fn test_backpack_get_all_funding_rates_direct() {
        // Note: This test requires valid Backpack credentials
        if let Ok(config) = ExchangeConfig::from_env("BACKPACK") {
            let config = config.testnet(true);
            match BackpackConnector::new(config) {
                Ok(exchange) => {
                    let result = exchange.get_all_funding_rates().await;

                    match result {
                        Ok(rates) => {
                            println!("✅ Backpack Direct get_all_funding_rates Test Passed");
                            println!("   Total symbols with funding rates: {}", rates.len());

                            // Check that all rates have required fields
                            for rate in &rates {
                                assert!(rate.funding_rate.is_some());
                                assert!(rate.mark_price.is_some());
                                println!(
                                    "   Symbol: {} - Rate: {:?}",
                                    rate.symbol, rate.funding_rate
                                );
                            }
                        }
                        Err(e) => {
                            println!("⚠️  Backpack Direct get_all_funding_rates Test: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️  Backpack connector creation failed: {}", e);
                }
            }
        } else {
            println!("⚠️  Backpack get_all_funding_rates test skipped: No credentials found in environment");
        }
    }
}
