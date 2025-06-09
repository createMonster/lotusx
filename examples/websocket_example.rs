use lotusx::core::{
    config::ExchangeConfig,
    traits::ExchangeConnector,
    types::*,
};
use lotusx::exchanges::{
    binance::BinanceConnector,
    binance_perp::BinancePerpConnector,
};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting WebSocket Market Data Example");

    // Configuration for Binance (you would normally load these from environment variables)
    let config = ExchangeConfig {
        api_key: "your_api_key".to_string(),
        secret_key: "your_secret_key".to_string(),
        base_url: None,
        testnet: false
    };

    // Example 1: Binance Spot WebSocket
    println!("\n📊 Setting up Binance Spot WebSocket...");
    let binance_spot = BinanceConnector::new(config.clone());
    
    let symbols = vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()];
    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::OrderBook { depth: Some(5) },
        SubscriptionType::Trades,
        SubscriptionType::Klines { interval: "1m".to_string() },
    ];

    let ws_config = WebSocketConfig {
        auto_reconnect: true,
        ping_interval: Some(30),
        max_reconnect_attempts: Some(5),
    };

    let mut spot_receiver = binance_spot
        .subscribe_market_data(symbols.clone(), subscription_types.clone(), Some(ws_config.clone()))
        .await?;

    // Example 2: Binance Perpetual Futures WebSocket
    println!("📈 Setting up Binance Perpetual Futures WebSocket...");
    let binance_perp = BinancePerpConnector::new(config.clone());
    
    let mut perp_receiver = binance_perp
        .subscribe_market_data(symbols, subscription_types, Some(ws_config))
        .await?;

    // Spawn tasks to handle incoming data
    let spot_handle = tokio::spawn(async move {
        println!("🔄 Listening for Binance Spot market data...");
        let mut count = 0;
        while let Some(data) = spot_receiver.recv().await {
            count += 1;
            match data {
                MarketDataType::Ticker(ticker) => {
                    println!("📊 [SPOT] Ticker: {} - Price: {} ({}%)", 
                        ticker.symbol, ticker.price, ticker.price_change_percent);
                }
                MarketDataType::OrderBook(orderbook) => {
                    println!("📖 [SPOT] OrderBook: {} - Best Bid: {}, Best Ask: {}", 
                        orderbook.symbol,
                        orderbook.bids.first().map(|b| &b.price).unwrap_or(&"N/A".to_string()),
                        orderbook.asks.first().map(|a| &a.price).unwrap_or(&"N/A".to_string())
                    );
                }
                MarketDataType::Trade(trade) => {
                    println!("💰 [SPOT] Trade: {} - Price: {}, Qty: {}, Buyer Maker: {}", 
                        trade.symbol, trade.price, trade.quantity, trade.is_buyer_maker);
                }
                MarketDataType::Kline(kline) => {
                    println!("📈 [SPOT] Kline: {} - O: {}, H: {}, L: {}, C: {}, Final: {}", 
                        kline.symbol, kline.open_price, kline.high_price, 
                        kline.low_price, kline.close_price, kline.final_bar);
                }
            }
            
            // Stop after receiving 50 messages for demo purposes
            if count >= 50 {
                println!("🛑 [SPOT] Received {} messages, stopping...", count);
                break;
            }
        }
    });

    let perp_handle = tokio::spawn(async move {
        println!("🔄 Listening for Binance Perpetual market data...");
        let mut count = 0;
        while let Some(data) = perp_receiver.recv().await {
            count += 1;
            match data {
                MarketDataType::Ticker(ticker) => {
                    println!("📊 [PERP] Ticker: {} - Price: {} ({}%)", 
                        ticker.symbol, ticker.price, ticker.price_change_percent);
                }
                MarketDataType::OrderBook(orderbook) => {
                    println!("📖 [PERP] OrderBook: {} - Best Bid: {}, Best Ask: {}", 
                        orderbook.symbol,
                        orderbook.bids.first().map(|b| &b.price).unwrap_or(&"N/A".to_string()),
                        orderbook.asks.first().map(|a| &a.price).unwrap_or(&"N/A".to_string())
                    );
                }
                MarketDataType::Trade(trade) => {
                    println!("💰 [PERP] Trade: {} - Price: {}, Qty: {}, Buyer Maker: {}", 
                        trade.symbol, trade.price, trade.quantity, trade.is_buyer_maker);
                }
                MarketDataType::Kline(kline) => {
                    println!("📈 [PERP] Kline: {} - O: {}, H: {}, L: {}, C: {}, Final: {}", 
                        kline.symbol, kline.open_price, kline.high_price, 
                        kline.low_price, kline.close_price, kline.final_bar);
                }
            }
            
            // Stop after receiving 50 messages for demo purposes
            if count >= 50 {
                println!("🛑 [PERP] Received {} messages, stopping...", count);
                break;
            }
        }
    });

    // Let the streams run for a while
    println!("⏳ Letting streams run for 30 seconds...");
    sleep(Duration::from_secs(30)).await;

    // Wait for both tasks to complete or timeout
    tokio::select! {
        _ = spot_handle => println!("✅ Spot stream completed"),
        _ = perp_handle => println!("✅ Perpetual stream completed"),
        _ = sleep(Duration::from_secs(60)) => println!("⏰ Timeout reached"),
    }

    println!("🏁 WebSocket example completed!");
    Ok(())
} 