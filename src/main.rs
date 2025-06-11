use lotusx::core::config::ExchangeConfig;
use lotusx::core::traits::MarketDataSource;
use lotusx::BinanceConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example usage - replace with your actual API credentials
    let config = ExchangeConfig::new("your_api_key".to_string(), "your_secret_key".to_string())
        .testnet(true); // Use testnet for safety

    let binance = BinanceConnector::new(config);

    // Get all markets
    println!("Fetching markets...");
    match binance.get_markets().await {
        Ok(markets) => {
            println!("Found {} markets", markets.len());
            // Print first 5 markets as example
            for market in markets.iter().take(5) {
                println!(
                    "Market: {} ({}->{}), Status: {}",
                    market.symbol.symbol, market.symbol.base, market.symbol.quote, market.status
                );
            }
        }
        Err(e) => {
            println!("Error fetching markets: {}", e);
        }
    }

    // Example order (commented out for safety)
    /*
    let order = OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: "0.001".to_string(),
        price: Some("30000.0".to_string()),
        time_in_force: Some(TimeInForce::GTC),
        stop_price: None,
    };

    match binance.place_order(order).await {
        Ok(response) => {
            println!("Order placed successfully: {:?}", response);
        }
        Err(e) => {
            println!("Error placing order: {}", e);
        }
    }
    */

    Ok(())
}
