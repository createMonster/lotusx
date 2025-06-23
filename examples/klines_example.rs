use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::MarketDataSource;
use lotusx::core::types::KlineInterval;
use lotusx::exchanges::binance::BinanceConnector;
use lotusx::exchanges::binance_perp::BinancePerpConnector;
use lotusx::exchanges::hyperliquid::HyperliquidClient;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 K-lines Example");
    println!("==================");

    // Example 1: Binance Spot K-lines
    println!("\n📈 Binance Spot K-lines");
    println!("----------------------");

    let binance_config =
        ExchangeConfig::new("your_api_key".to_string(), "your_secret_key".to_string())
            .testnet(true);

    let binance_client = BinanceConnector::new(binance_config);

    // Get last 10 1-minute k-lines for BTCUSDT
    match binance_client
        .get_klines(
            "BTCUSDT".to_string(),
            KlineInterval::Minutes1,
            Some(10),
            None,
            None,
        )
        .await
    {
        Ok(klines) => {
            println!("✅ Retrieved {} k-lines for BTCUSDT:", klines.len());
            for (i, kline) in klines.iter().enumerate() {
                println!(
                    "  {}. Time: {}, O: {}, H: {}, L: {}, C: {}, V: {}",
                    i + 1,
                    kline.open_time,
                    kline.open_price,
                    kline.high_price,
                    kline.low_price,
                    kline.close_price,
                    kline.volume
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to get Binance k-lines: {}", e);
        }
    }

    // Example 2: Binance Perpetual K-lines
    println!("\n📈 Binance Perpetual K-lines");
    println!("----------------------------");

    let binance_perp_config =
        ExchangeConfig::new("your_api_key".to_string(), "your_secret_key".to_string())
            .testnet(true);

    let binance_perp_client = BinancePerpConnector::new(binance_perp_config);

    // Get last 5 5-minute k-lines for BTCUSDT
    match binance_perp_client
        .get_klines(
            "BTCUSDT".to_string(),
            KlineInterval::Minutes5,
            Some(5),
            None,
            None,
        )
        .await
    {
        Ok(klines) => {
            println!("✅ Retrieved {} k-lines for BTCUSDT (Perp):", klines.len());
            for (i, kline) in klines.iter().enumerate() {
                println!(
                    "  {}. Time: {}, O: {}, H: {}, L: {}, C: {}, V: {}",
                    i + 1,
                    kline.open_time,
                    kline.open_price,
                    kline.high_price,
                    kline.low_price,
                    kline.close_price,
                    kline.volume
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to get Binance Perp k-lines: {}", e);
        }
    }

    // Example 3: Hyperliquid K-lines
    println!("\n📈 Hyperliquid K-lines");
    println!("----------------------");

    let hyperliquid_client = HyperliquidClient::read_only(false);

    // Get k-lines for BTC with specific time range
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    let one_hour_ago = now - (60 * 60 * 1000); // 1 hour ago

    match hyperliquid_client
        .get_klines(
            "BTC".to_string(),
            KlineInterval::Minutes1,
            Some(10),
            Some(one_hour_ago),
            Some(now),
        )
        .await
    {
        Ok(klines) => {
            println!(
                "✅ Retrieved {} k-lines for BTC (Hyperliquid):",
                klines.len()
            );
            for (i, kline) in klines.iter().enumerate() {
                println!(
                    "  {}. Time: {}, O: {}, H: {}, L: {}, C: {}, V: {}",
                    i + 1,
                    kline.open_time,
                    kline.open_price,
                    kline.high_price,
                    kline.low_price,
                    kline.close_price,
                    kline.volume
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to get Hyperliquid k-lines: {}", e);
        }
    }

    // Example 4: Demonstrate different intervals
    println!("\n📊 Different Intervals Example");
    println!("------------------------------");

    let intervals = vec![
        (KlineInterval::Minutes1, "1-minute"),
        (KlineInterval::Minutes5, "5-minute"),
        (KlineInterval::Hours1, "1-hour"),
        (KlineInterval::Days1, "1-day"),
    ];

    for (interval, description) in intervals {
        println!("Testing {} interval:", description);
        println!("  - Binance format: {}", interval.to_binance_format());

        if interval.is_supported_by_binance() {
            match binance_client
                .get_klines("BTCUSDT".to_string(), interval, Some(2), None, None)
                .await
            {
                Ok(klines) => {
                    println!("  ✅ Retrieved {} k-lines from Binance", klines.len());
                }
                Err(e) => {
                    println!("  ❌ Failed to get Binance k-lines: {}", e);
                }
            }
        } else {
            println!("  ⚠️ Interval not supported by Binance");
        }
    }

    println!("\n🏁 K-lines example completed!");
    Ok(())
}
