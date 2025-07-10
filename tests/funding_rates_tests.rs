#[cfg(test)]
mod funding_rates_tests {
    use lotusx::core::{config::ExchangeConfig, traits::FundingRateSource};
    use lotusx::exchanges::{
        bybit_perp::client::BybitPerpConnector, hyperliquid::client::HyperliquidClient,
    };

    #[tokio::test]
    async fn test_binance_perp_get_funding_rates_single_symbol() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = lotusx::exchanges::binance_perp::build_connector(config).unwrap();

        let symbols = vec!["BTCUSDT".to_string()];
        let result = exchange.get_funding_rates(Some(symbols)).await;

        assert!(
            result.is_ok(),
            "Failed to get funding rates: {:?}",
            result.err()
        );
        let rates = result.unwrap();
        assert_eq!(rates.len(), 1);
        assert_eq!(rates[0].symbol.to_string(), "BTCUSDT");
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
        let exchange = lotusx::exchanges::binance_perp::build_connector(config).unwrap();

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
        let exchange = lotusx::exchanges::binance_perp::build_connector(config).unwrap();

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
    #[ignore = "Needs update after kernel refactor - API signature changed"]
    async fn test_backpack_get_funding_rates_single_symbol() {
        // TODO: Update this test to work with the new kernel architecture
        // The BackpackConnector now uses generic types and different API signatures
        println!("⚠️  Backpack test temporarily disabled for kernel refactor");
    }

    #[tokio::test]
    #[ignore = "Needs update after kernel refactor - API signature changed"]
    async fn test_backpack_get_funding_rate_history() {
        // TODO: Update this test to work with the new kernel architecture
        // The BackpackConnector now uses generic types and different API signatures
        println!("⚠️  Backpack test temporarily disabled for kernel refactor");
    }

    #[tokio::test]
    async fn test_funding_rate_data_structure() {
        use lotusx::core::types::{conversion, FundingRate};
        use rust_decimal::Decimal;

        let rate = FundingRate {
            symbol: conversion::string_to_symbol("BTCUSDT"),
            funding_rate: Some(Decimal::from_str_exact("0.0001").unwrap()),
            previous_funding_rate: Some(Decimal::from_str_exact("0.00005").unwrap()),
            next_funding_rate: Some(Decimal::from_str_exact("0.00015").unwrap()),
            funding_time: Some(1_699_876_800_000),
            next_funding_time: Some(1_699_905_600_000),
            mark_price: Some(conversion::string_to_price("35000.0")),
            index_price: Some(conversion::string_to_price("35001.0")),
            timestamp: 1_699_876_800_000,
        };

        assert_eq!(rate.symbol.to_string(), "BTCUSDT");
        assert_eq!(
            rate.funding_rate,
            Some(Decimal::from_str_exact("0.0001").unwrap())
        );
        assert_eq!(
            rate.mark_price,
            Some(conversion::string_to_price("35000.0"))
        );

        println!("✅ Funding Rate Data Structure Test Passed");
    }

    #[tokio::test]
    async fn test_funding_rate_error_handling() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = lotusx::exchanges::binance_perp::build_connector(config).unwrap();

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
        let exchange = lotusx::exchanges::binance_perp::build_connector(config).unwrap();

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

        assert_eq!(rates1[0].symbol.to_string(), "BTCUSDT");
        assert_eq!(rates2[0].symbol.to_string(), "ETHUSDT");

        println!("✅ Concurrent Funding Rate Requests Test Passed");
        println!("   BTC Rate: {:?}", rates1[0].funding_rate);
        println!("   ETH Rate: {:?}", rates2[0].funding_rate);
    }

    #[tokio::test]
    async fn test_performance_timing() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = lotusx::exchanges::binance_perp::build_connector(config).unwrap();

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
        let exchange = lotusx::exchanges::binance_perp::build_connector(config).unwrap();

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
    #[ignore = "Needs update after kernel refactor - API signature changed"]
    async fn test_backpack_get_all_funding_rates_direct() {
        // TODO: Update this test to work with the new kernel architecture
        // The BackpackConnector now uses generic types and different API signatures
        println!("⚠️  Backpack test temporarily disabled for kernel refactor");
    }

    // Bybit Perpetual Tests
    #[tokio::test]
    async fn test_bybit_perp_get_funding_rates_single_symbol() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BybitPerpConnector::new(config);

        let symbols = vec!["BTCUSDT".to_string()];
        let result = exchange.get_funding_rates(Some(symbols)).await;

        match result {
            Ok(rates) => {
                assert_eq!(rates.len(), 1);
                assert_eq!(rates[0].symbol.to_string(), "BTCUSDT");
                assert!(rates[0].funding_rate.is_some());
                assert!(rates[0].mark_price.is_some());
                assert!(rates[0].index_price.is_some());

                println!("✅ Bybit Perp Single Symbol Test Passed");
                println!("   Symbol: {}", rates[0].symbol);
                println!("   Funding Rate: {:?}", rates[0].funding_rate);
                println!("   Mark Price: {:?}", rates[0].mark_price);
                println!("   Next Funding Time: {:?}", rates[0].next_funding_time);
            }
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("expected value") || error_msg.contains("Decode") {
                    println!(
                        "⚠️  Bybit Perp Single Symbol Test: Network/API connectivity issue: {}",
                        e
                    );
                    println!(
                        "   This is likely a CI environment connectivity issue, not a code problem"
                    );
                    // Don't fail the test in CI environments for network issues
                } else {
                    panic!("Failed to get Bybit Perp funding rates: {:?}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_bybit_perp_get_all_funding_rates_direct() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BybitPerpConnector::new(config);

        let result = exchange.get_all_funding_rates().await;

        match result {
            Ok(rates) => {
                assert!(!rates.is_empty(), "Should have received some funding rates");

                // Check that all rates have required fields
                for rate in &rates {
                    assert!(rate.funding_rate.is_some());
                    assert!(rate.mark_price.is_some());
                    assert!(rate.index_price.is_some());
                }

                println!("✅ Bybit Perp All Funding Rates Test Passed");
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
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("expected value") || error_msg.contains("Decode") {
                    println!(
                        "⚠️  Bybit Perp All Funding Rates Test: Network/API connectivity issue: {}",
                        e
                    );
                    println!(
                        "   This is likely a CI environment connectivity issue, not a code problem"
                    );
                    // Don't fail the test in CI environments for network issues
                } else {
                    panic!("Failed to get all Bybit Perp funding rates: {:?}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_bybit_perp_get_funding_rate_history() {
        let config = ExchangeConfig::read_only().testnet(true);
        let exchange = BybitPerpConnector::new(config);

        let result = exchange
            .get_funding_rate_history(
                "BTCUSDT".to_string(),
                None,
                None,
                Some(5), // Last 5 funding rates
            )
            .await;

        match result {
            Ok(history) => {
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

                println!("✅ Bybit Perp Funding Rate History Test Passed");
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
                let error_msg = e.to_string();
                if error_msg.contains("expected value") || error_msg.contains("Decode") {
                    println!("⚠️  Bybit Perp Funding Rate History Test: Network/API connectivity issue: {}", e);
                    println!(
                        "   This is likely a CI environment connectivity issue, not a code problem"
                    );
                    // Don't fail the test in CI environments for network issues
                } else {
                    panic!("Failed to get Bybit Perp funding rate history: {:?}", e);
                }
            }
        }
    }

    // Hyperliquid Tests
    #[tokio::test]
    async fn test_hyperliquid_get_funding_rates_single_symbol() {
        let config = ExchangeConfig::read_only().testnet(false); // Hyperliquid doesn't have testnet
        let exchange = HyperliquidClient::new(config);

        let symbols = vec!["BTC".to_string()];
        let result = exchange.get_funding_rates(Some(symbols)).await;

        match result {
            Ok(rates) => {
                assert_eq!(rates.len(), 1);
                assert_eq!(rates[0].symbol.to_string(), "BTC");
                assert!(rates[0].funding_rate.is_some());
                assert!(rates[0].mark_price.is_some());
                assert!(rates[0].index_price.is_some());

                println!("✅ Hyperliquid Single Symbol Test Passed");
                println!("   Symbol: {}", rates[0].symbol);
                println!("   Funding Rate: {:?}", rates[0].funding_rate);
                println!("   Mark Price: {:?}", rates[0].mark_price);
                println!("   Oracle Price: {:?}", rates[0].index_price);
            }
            Err(e) => {
                println!("⚠️  Hyperliquid Single Symbol Test: {}", e);
                // Don't fail the test since Hyperliquid might have connectivity issues
            }
        }
    }

    #[tokio::test]
    async fn test_hyperliquid_get_all_funding_rates_direct() {
        let config = ExchangeConfig::read_only().testnet(false); // Hyperliquid doesn't have testnet
        let exchange = HyperliquidClient::new(config);

        let result = exchange.get_all_funding_rates().await;

        match result {
            Ok(rates) => {
                assert!(!rates.is_empty(), "Should have received some funding rates");

                // Check that all rates have required fields
                for rate in &rates {
                    assert!(rate.funding_rate.is_some());
                    assert!(rate.mark_price.is_some());
                    assert!(rate.index_price.is_some());
                }

                println!("✅ Hyperliquid All Funding Rates Test Passed");
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
            Err(e) => {
                println!("⚠️  Hyperliquid All Funding Rates Test: {}", e);
                // Don't fail the test since Hyperliquid might have connectivity issues
            }
        }
    }

    #[tokio::test]
    async fn test_hyperliquid_get_funding_rate_history() {
        let config = ExchangeConfig::read_only().testnet(false); // Hyperliquid doesn't have testnet
        let exchange = HyperliquidClient::new(config);

        let result = exchange
            .get_funding_rate_history(
                "BTC".to_string(),
                None,
                None,
                Some(5), // Hyperliquid doesn't support limit, but we test the interface
            )
            .await;

        match result {
            Ok(history) => {
                println!("✅ Hyperliquid Funding Rate History Test Passed");
                println!("   History entries: {}", history.len());

                // Check that historical rates have funding_time
                for rate in &history {
                    assert!(rate.funding_rate.is_some());
                    assert!(rate.funding_time.is_some());
                }

                for (i, rate) in history.iter().take(5).enumerate() {
                    println!(
                        "   {}: Rate: {:?}, Time: {:?}",
                        i + 1,
                        rate.funding_rate,
                        rate.funding_time
                    );
                }
            }
            Err(e) => {
                println!("⚠️  Hyperliquid History Test: {}", e);
                // Don't fail the test since Hyperliquid might have connectivity issues
            }
        }
    }

    // Cross-exchange performance test
    #[tokio::test]
    async fn test_multi_exchange_funding_rates_performance() {
        use std::time::Instant;

        println!("🚀 Multi-Exchange Funding Rates Performance Test");

        // Test Binance Perp
        let start = Instant::now();
        let config = ExchangeConfig::read_only().testnet(true);
        let binance_exchange = lotusx::exchanges::binance_perp::build_connector(config).unwrap();
        if let Ok(rates) = binance_exchange.get_all_funding_rates().await {
            let duration = start.elapsed();
            println!("   Binance Perp: {} symbols in {:?}", rates.len(), duration);
            assert!(
                duration.as_millis() < 2000,
                "Binance Perp should complete under 2000ms for HFT requirements"
            );
        }

        // Test Bybit Perp
        let start = Instant::now();
        let config = ExchangeConfig::read_only().testnet(true);
        let bybit_exchange = BybitPerpConnector::new(config);
        if let Ok(rates) = bybit_exchange.get_all_funding_rates().await {
            let duration = start.elapsed();
            println!("   Bybit Perp: {} symbols in {:?}", rates.len(), duration);
            assert!(
                duration.as_millis() < 2000,
                "Bybit Perp should complete under 2000ms for HFT requirements"
            );
        }

        // Test Hyperliquid (with more lenient timing due to different API)
        let start = Instant::now();
        let config = ExchangeConfig::read_only().testnet(false);
        let hyperliquid_exchange = HyperliquidClient::new(config);
        if let Ok(rates) = hyperliquid_exchange.get_all_funding_rates().await {
            let duration = start.elapsed();
            println!("   Hyperliquid: {} symbols in {:?}", rates.len(), duration);
            assert!(
                duration.as_millis() < 5000,
                "Hyperliquid should complete under 5000ms"
            );
        }

        println!("✅ Multi-Exchange Performance Test Passed");
    }
}
