#[allow(unused_imports)]
use lotusx::{
    core::{config::ExchangeConfig, traits::{AccountInfo, MarketDataSource}},
    exchanges::{bybit::BybitConnector, bybit_perp::BybitPerpConnector},
    OrderRequest, OrderSide, OrderType, SubscriptionType,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Bybit Spot Trading
    println!("=== Bybit Spot Example ===");
    
    // Create configuration (you can also use ExchangeConfig::from_env("BYBIT"))
    let config = ExchangeConfig::from_env_file("BYBIT")?;
    
    let bybit_spot = BybitConnector::new(config.clone());
    
    // Get available markets
    match bybit_spot.get_markets().await {
        Ok(markets) => {
            println!("Found {} spot markets", markets.len());
            if let Some(first_market) = markets.first() {
                println!("First market: {}", first_market.symbol);
            }
        }
        Err(e) => println!("Error getting markets: {}", e),
    }
    
    // Get account balance (requires valid API credentials)
    if config.has_credentials() {
        match bybit_spot.get_account_balance().await {
            Ok(balances) => {
                println!("Account balances:");
                for balance in balances {
                    println!("  {}: free={}, locked={}", balance.asset, balance.free, balance.locked);
                }
            }
            Err(e) => println!("Error getting balance: {}", e),
        }
    }
    
    // Example 2: Bybit Perpetual Futures
    println!("\n=== Bybit Perpetual Futures Example ===");
    
    let bybit_perp = BybitPerpConnector::new(config.clone());
    
    // Get available perpetual markets
    match bybit_perp.get_markets().await {
        Ok(markets) => {
            println!("Found {} perpetual markets", markets.len());
            if let Some(first_market) = markets.first() {
                println!("First perpetual market: {}", first_market.symbol);
            }
        }
        Err(e) => println!("Error getting perpetual markets: {}", e),
    }
    
    // Get positions (requires valid API credentials)
    if config.has_credentials() {
        match bybit_perp.get_positions().await {
            Ok(positions) => {
                println!("Open positions:");
                for position in positions {
                    println!("  {}: side={:?}, size={}, entry_price={}", 
                        position.symbol, position.position_side, 
                        position.position_amount, position.entry_price);
                }
            }
            Err(e) => println!("Error getting positions: {}", e),
        }
    }
    
    // Example 3: Place an order (commented out for safety)
    /*
    let order = OrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit,
        quantity: "0.001".to_string(),
        price: Some("30000.0".to_string()),
        time_in_force: None,
        stop_price: None,
    };
    
    match bybit_spot.place_order(order).await {
        Ok(response) => println!("Order placed: {}", response.order_id),
        Err(e) => println!("Error placing order: {}", e),
    }
    */
    
    // Example 4: WebSocket Market Data (commented out as it runs indefinitely)
    /*
    let symbols = vec!["BTCUSDT".to_string()];
    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::OrderBook { depth: Some(10) },
        SubscriptionType::Trades,
    ];
    
    match bybit_spot.subscribe_market_data(symbols, subscription_types, None).await {
        Ok(mut receiver) => {
            println!("Subscribed to market data, listening for updates...");
            while let Some(data) = receiver.recv().await {
                println!("Received: {:?}", data);
            }
        }
        Err(e) => println!("Error subscribing to market data: {}", e),
    }
    */
    
    println!("\nBybit integration examples completed!");
    Ok(())
} 