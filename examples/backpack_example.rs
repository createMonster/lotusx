use lotusx::core::{
    config::ExchangeConfig,
    traits::{AccountInfo, MarketDataSource},
    types::KlineInterval,
};
use lotusx::exchanges::backpack::build_connector;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration from environment variables
    // You need to set BACKPACK_API_KEY and BACKPACK_SECRET_KEY
    let config = match ExchangeConfig::from_env_file("BACKPACK") {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Please set BACKPACK_API_KEY and BACKPACK_SECRET_KEY environment variables");
            return Ok(());
        }
    };

    // Create Backpack connector using the new builder
    let backpack = build_connector(config)?;

    println!("🚀 Backpack Exchange Integration Example");
    println!("=========================================");

    // Example 1: Get available markets
    println!("\n📊 Getting available markets...");
    match MarketDataSource::get_markets(&backpack).await {
        Ok(markets) => {
            println!("Found {} markets:", markets.len());
            for (i, market) in markets.iter().take(5).enumerate() {
                println!("  {}. {} ({})", i + 1, market.symbol, market.status);
            }
            if markets.len() > 5 {
                println!("  ... and {} more", markets.len() - 5);
            }
        }
        Err(e) => eprintln!("Error getting markets: {}", e),
    }

    // Example 2: Get historical klines
    println!("\n📈 Getting SOL-USDC 1h klines...");
    match MarketDataSource::get_klines(
        &backpack,
        "SOL_USDC".to_string(),
        KlineInterval::Hours1,
        Some(5),
        None,
        None,
    )
    .await
    {
        Ok(klines) => {
            println!("Recent 1h Klines:");
            for kline in klines.iter().take(5) {
                println!(
                    "  Open: ${}, High: ${}, Low: ${}, Close: ${}, Volume: {}",
                    kline.open_price,
                    kline.high_price,
                    kline.low_price,
                    kline.close_price,
                    kline.volume
                );
            }
        }
        Err(e) => eprintln!("Error getting klines: {}", e),
    }

    // Example 3: Get account balance (requires authentication) - using AccountInfo trait
    println!("\n💼 Getting account balance...");
    match AccountInfo::get_account_balance(&backpack).await {
        Ok(balances) => {
            println!("Account Balances:");
            for balance in balances.iter().take(10) {
                if balance.free.to_string().parse::<f64>().unwrap_or(0.0) > 0.0
                    || balance.locked.to_string().parse::<f64>().unwrap_or(0.0) > 0.0
                {
                    println!(
                        "  {}: Free: {}, Locked: {}",
                        balance.asset, balance.free, balance.locked
                    );
                }
            }
        }
        Err(e) => eprintln!("Error getting account balance: {}", e),
    }

    // Example 4: Get positions (requires authentication) - using AccountInfo trait
    println!("\n📍 Getting positions...");
    match AccountInfo::get_positions(&backpack).await {
        Ok(positions) => {
            if positions.is_empty() {
                println!("No open positions");
            } else {
                println!("Open Positions:");
                for position in positions {
                    println!(
                        "  {}: {:?} {} @ ${}, PnL: ${}",
                        position.symbol,
                        position.position_side,
                        position.position_amount,
                        position.entry_price,
                        position.unrealized_pnl
                    );
                }
            }
        }
        Err(e) => eprintln!("Error getting positions: {}", e),
    }

    // Example 5: WebSocket market data (commented out due to connection requirements)
    /*
    println!("\n🔄 Starting WebSocket market data stream...");
    let symbols = vec!["SOL_USDC".to_string()];
    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::OrderBook { depth: Some(5) },
        SubscriptionType::Trades,
    ];

    match backpack.subscribe_market_data(symbols, subscription_types, None).await {
        Ok(mut receiver) => {
            println!("WebSocket connected! Receiving market data...");
            let mut count = 0;
            while let Some(data) = receiver.recv().await {
                match data {
                    MarketDataType::Ticker(ticker) => {
                        println!("📊 Ticker: {} @ ${}", ticker.symbol, ticker.price);
                    }
                    MarketDataType::Trade(trade) => {
                        println!("🔄 Trade: {} {} @ ${}", trade.symbol, trade.quantity, trade.price);
                    }
                    MarketDataType::OrderBook(orderbook) => {
                        println!("📖 OrderBook: {} (bids: {}, asks: {})",
                            orderbook.symbol, orderbook.bids.len(), orderbook.asks.len());
                    }
                    _ => {}
                }

                count += 1;
                if count >= 10 {
                    println!("Received 10 messages, stopping...");
                    break;
                }
            }
        }
        Err(e) => eprintln!("Error starting WebSocket: {}", e),
    }
    */

    println!("\n✅ Backpack Exchange integration example completed!");
    println!("\nNote: Some operations require valid API credentials.");
    println!("Set BACKPACK_API_KEY and BACKPACK_SECRET_KEY in your .env file or as environment variables to test authenticated endpoints.");

    Ok(())
}
