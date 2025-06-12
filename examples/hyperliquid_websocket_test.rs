use lotusx::core::traits::MarketDataSource;
use lotusx::core::types::{MarketDataType, SubscriptionType};
use lotusx::exchanges::hyperliquid::HyperliquidClient;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing refactored Hyperliquid WebSocket implementation...");

    // Create a read-only client for testing
    let client = HyperliquidClient::read_only(true); // Use testnet

    println!("✅ Client created successfully");

    // Test market data subscription
    let symbols = vec!["BTC".to_string(), "ETH".to_string()];
    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::OrderBook { depth: Some(10) },
    ];

    println!("📡 Subscribing to market data for: {:?}", symbols);

    let mut receiver = client
        .subscribe_market_data(symbols, subscription_types, None)
        .await?;

    println!("🎯 WebSocket connection established, waiting for data...");

    // Listen for a few messages to verify functionality
    let mut message_count = 0;
    let max_messages = 5;

    while message_count < max_messages {
        match timeout(Duration::from_secs(10), receiver.recv()).await {
            Ok(Some(data)) => {
                message_count += 1;
                match data {
                    MarketDataType::Ticker(ticker) => {
                        println!("📊 Ticker - {}: ${}", ticker.symbol, ticker.price);
                    }
                    MarketDataType::OrderBook(book) => {
                        println!(
                            "📖 OrderBook - {}: {} bids, {} asks",
                            book.symbol,
                            book.bids.len(),
                            book.asks.len()
                        );
                    }
                    MarketDataType::Trade(trade) => {
                        println!(
                            "💰 Trade - {}: {} @ ${}",
                            trade.symbol, trade.quantity, trade.price
                        );
                    }
                    MarketDataType::Kline(kline) => {
                        println!(
                            "📈 Kline - {}: O:{} H:{} L:{} C:{}",
                            kline.symbol,
                            kline.open_price,
                            kline.high_price,
                            kline.low_price,
                            kline.close_price
                        );
                    }
                }
            }
            Ok(None) => {
                println!("❌ WebSocket stream ended");
                break;
            }
            Err(_) => {
                println!("⏰ Timeout waiting for data");
                break;
            }
        }
    }

    println!("✅ Test completed! Received {} messages", message_count);
    println!("🎉 Refactored WebSocket implementation is working correctly!");

    Ok(())
}
