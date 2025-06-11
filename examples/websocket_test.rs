use lotusx::core::{
    config::ExchangeConfig,
    traits::MarketDataSource,
    types::{MarketDataType, SubscriptionType},
};
use lotusx::exchanges::binance::BinanceConnector;
use secrecy::Secret;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing WebSocket Connection");

    let config = ExchangeConfig {
        api_key: Secret::new("test_key".to_string()), // Not needed for market data
        secret_key: Secret::new("test_secret".to_string()), // Not needed for market data
        base_url: None,
        testnet: true, // Try testnet first
    };

    let binance = BinanceConnector::new(config);

    // Test with just one symbol and one subscription type to simplify
    let symbols = vec!["BTCUSDT".to_string()];
    let subscription_types = vec![SubscriptionType::Ticker];

    println!("🌐 WebSocket URL: {}", binance.get_websocket_url());
    println!("📊 Attempting to connect to Binance WebSocket...");

    // No need for WebSocketConfig anymore - the new implementation is simpler
    let mut receiver = binance
        .subscribe_market_data(symbols, subscription_types, None)
        .await?;

    println!("✅ WebSocket connection established! Waiting for data...");

    // Listen for just a few messages
    let mut count = 0;
    while let Some(data) = receiver.recv().await {
        count += 1;
        match data {
            MarketDataType::Ticker(ticker) => {
                println!(
                    "📈 Ticker received: {} - Price: {}",
                    ticker.symbol, ticker.price
                );
            }
            _ => {
                println!("📊 Other data received: {:?}", data);
            }
        }

        if count >= 5 {
            println!("✅ Successfully received {} messages", count);
            break;
        }
    }

    println!("🏁 Test completed successfully!");
    Ok(())
}
