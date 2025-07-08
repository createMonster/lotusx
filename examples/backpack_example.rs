use lotusx::core::{
    config::ExchangeConfig,
    traits::{AccountInfo, MarketDataSource},
    types::KlineInterval,
};
use lotusx::exchanges::backpack::BackpackConnector;

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

    // Create Backpack connector
    let backpack = BackpackConnector::new(config)?;

    println!("🚀 Backpack Exchange Integration Example");
    println!("=========================================");

    // Example 1: Get available markets
    println!("\n📊 Getting available markets...");
    match backpack.get_markets().await {
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

    // Example 2: Get ticker for SOL-USDC
    println!("\n💰 Getting SOL-USDC ticker...");
    match backpack.get_ticker("SOL_USDC").await {
        Ok(ticker) => {
            println!("SOL-USDC Ticker:");
            println!("  Price: ${}", ticker.price);
            println!("  24h Change: {}%", ticker.price_change_percent);
            println!("  24h Volume: {}", ticker.volume);
            println!("  High: ${}", ticker.high_price);
            println!("  Low: ${}", ticker.low_price);
        }
        Err(e) => eprintln!("Error getting ticker: {}", e),
    }

    // Example 3: Get order book
    println!("\n📖 Getting SOL-USDC order book...");
    match backpack.get_order_book("SOL_USDC", Some(5)).await {
        Ok(order_book) => {
            println!("Order Book (Top 5):");
            println!("  Asks:");
            for ask in order_book.asks.iter().take(5) {
                println!("    ${} x {}", ask.price, ask.quantity);
            }
            println!("  Bids:");
            for bid in order_book.bids.iter().take(5) {
                println!("    ${} x {}", bid.price, bid.quantity);
            }
        }
        Err(e) => eprintln!("Error getting order book: {}", e),
    }

    // Example 4: Get recent trades
    println!("\n🔄 Getting recent SOL-USDC trades...");
    match backpack.get_trades("SOL_USDC", Some(5)).await {
        Ok(trades) => {
            println!("Recent Trades:");
            for trade in trades.iter().take(5) {
                println!("  ${} x {} at {}", trade.price, trade.quantity, trade.time);
            }
        }
        Err(e) => eprintln!("Error getting trades: {}", e),
    }

    // Example 5: Get historical klines
    println!("\n📈 Getting SOL-USDC 1h klines...");
    match backpack
        .get_klines(
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

    // Example 6: Get account balance (requires authentication)
    println!("\n💼 Getting account balance...");
    match backpack.get_account_balance().await {
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

    // Example 7: Get positions (requires authentication)
    println!("\n📍 Getting positions...");
    match backpack.get_positions().await {
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

    // Example 8: WebSocket market data (commented out due to connection requirements)
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
