use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::MarketDataSource;
use lotusx::BinanceConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the Binance connector with your API credentials
    // For safety, this example uses testnet
    let config = ExchangeConfig::new(
        std::env::var("BINANCE_API_KEY").unwrap_or_else(|_| "your_api_key".to_string()),
        std::env::var("BINANCE_SECRET_KEY").unwrap_or_else(|_| "your_secret_key".to_string()),
    )
    .testnet(true);

    let binance = BinanceConnector::new(config);

    // Example 1: Get all available markets
    println!("=== Getting Markets ===");
    match binance.get_markets().await {
        Ok(markets) => {
            println!("Successfully fetched {} markets", markets.len());

            // Show some example markets
            println!("\nFirst 10 markets:");
            for market in markets.iter().take(10) {
                println!(
                    "  {} ({} -> {}) - Status: {}",
                    market.symbol.symbol, market.symbol.base, market.symbol.quote, market.status
                );
            }

            // Find BTCUSDT market as an example
            if let Some(btc_market) = markets.iter().find(|m| m.symbol.symbol == "BTCUSDT") {
                println!("\nBTCUSDT Market Details:");
                println!("  Base Precision: {}", btc_market.base_precision);
                println!("  Quote Precision: {}", btc_market.quote_precision);
                if let Some(min_qty) = &btc_market.min_qty {
                    println!("  Min Quantity: {}", min_qty);
                }
                if let Some(min_price) = &btc_market.min_price {
                    println!("  Min Price: {}", min_price);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get markets: {}", e);
        }
    }

    // Example 2: Place a limit order (COMMENTED OUT FOR SAFETY)
    // UNCOMMENT AND MODIFY ONLY IF YOU WANT TO PLACE REAL ORDERS
    /*
    println!("\n=== Placing Order ===");
    let order = OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: "0.001".to_string(), // Very small amount for testing
        price: Some("25000.0".to_string()), // Below market price to avoid immediate fill
        time_in_force: Some(TimeInForce::GTC),
        stop_price: None,
    };

    match binance.place_order(order).await {
        Ok(response) => {
            println!("Order placed successfully!");
            println!("  Order ID: {}", response.order_id);
            println!("  Symbol: {}", response.symbol);
            println!("  Side: {:?}", response.side);
            println!("  Type: {:?}", response.order_type);
            println!("  Quantity: {}", response.quantity);
            println!("  Status: {}", response.status);
            if let Some(price) = response.price {
                println!("  Price: {}", price);
            }
        }
        Err(e) => {
            eprintln!("Failed to place order: {}", e);
        }
    }
    */

    println!("\n=== Example completed ===");
    Ok(())
}
