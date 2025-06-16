#[allow(unused_imports)]
use lotusx::{
    core::{
        config::ExchangeConfig,
        traits::{AccountInfo, MarketDataSource},
    },
    exchanges::{bybit::BybitConnector, bybit_perp::BybitPerpConnector},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Bybit Spot Trading
    println!("=== Bybit Spot Example ===");

    // Create configuration (you can also use ExchangeConfig::from_env("BYBIT"))
    let config = ExchangeConfig::from_env_file("BYBIT")?;

    let bybit_spot = BybitConnector::new(config.clone());

    let markets = bybit_spot.get_markets().await?;
    println!("Found {} markets", markets.len());
    println!("First market: {}", markets[0].symbol);

    // Get account balance (requires valid API credentials)
    match bybit_spot.get_account_balance().await {
        Ok(balances) => {
            println!("Account balances:");
            for balance in balances {
                println!(
                    "  {}: free={}, locked={}",
                    balance.asset, balance.free, balance.locked
                );
            }
        }
        Err(e) => println!("Error getting balance: {}", e),
    }

    // Example 2: Bybit Perpetual Futures
    println!("\n=== Bybit Perpetual Futures Example ===");

    let bybit_perp = BybitPerpConnector::new(config.clone());

    let markets = bybit_perp.get_markets().await?;
    println!("Found {} markets", markets.len());
    println!("First market: {}", markets[0].symbol);

    // Get positions (requires valid API credentials)
    match bybit_perp.get_positions().await {
        Ok(positions) => {
            println!("Open positions:");
            for position in positions {
                println!(
                    "  {}: side={:?}, size={}, entry_price={}",
                    position.symbol,
                    position.position_side,
                    position.position_amount,
                    position.entry_price
                );
            }
        }
        Err(e) => println!("Error getting positions: {}", e),
    }

    println!("\nBybit integration examples completed!");
    Ok(())
}
