use lotusx::core::{config::ExchangeConfig, types::SubscriptionType};
use lotusx::exchanges::backpack::{
    codec::BackpackMessage, create_backpack_connector, create_backpack_connector_with_reconnection,
    create_backpack_stream_identifiers,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create configuration
    let config = ExchangeConfig::new(
        String::new(), // No API key needed for public endpoints
        String::new(), // No secret key needed for public endpoints
    );

    // Example 1: REST API usage
    println!("=== REST API Example ===");

    let backpack = create_backpack_connector(config.clone(), false)?;

    // Get markets
    let markets = backpack.get_markets().await?;
    let market_count = markets.len();
    println!("Found {} markets", market_count);

    // Extract a valid symbol from the markets response
    let valid_symbol = markets
        .first()
        .map_or("SOL_USDC", |market| market.symbol.as_str());

    println!("Using symbol: {}", valid_symbol);

    // Get ticker for a specific symbol
    match backpack.get_ticker(valid_symbol).await {
        Ok(ticker) => println!("Ticker response: {:?}", ticker),
        Err(e) => println!("Ticker error: {:?}", e),
    }

    // Get order book
    match backpack.get_order_book(valid_symbol, Some(10)).await {
        Ok(order_book) => println!("Order book response: {:?}", order_book),
        Err(e) => println!("Order book error: {:?}", e),
    }

    // Get recent trades
    match backpack.get_trades(valid_symbol, Some(5)).await {
        Ok(trades) => println!("Recent trades: {:?}", trades),
        Err(e) => println!("Trades error: {:?}", e),
    }

    // Example 2: WebSocket usage
    println!("\n=== WebSocket Example ===");

    let mut backpack_ws = create_backpack_connector_with_reconnection(config.clone(), true)?;

    // Create subscription streams
    let symbols = vec![valid_symbol.to_string(), "ETH_USDC".to_string()];
    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::OrderBook { depth: Some(10) },
        SubscriptionType::Trades,
    ];

    let streams = create_backpack_stream_identifiers(&symbols, &subscription_types);
    println!("Subscription streams: {:?}", streams);

    // Subscribe to streams
    match backpack_ws.subscribe_websocket(&streams).await {
        Ok(_) => println!("Subscribed to WebSocket streams"),
        Err(e) => {
            println!("WebSocket subscription error: {:?}", e);
            return Ok(());
        }
    }

    // Process messages for a short time
    let mut message_count = 0;
    let max_messages = 10;

    while message_count < max_messages {
        if let Some(message_result) = backpack_ws.next_websocket_message().await {
            match message_result {
                Ok(message) => {
                    match message {
                        BackpackMessage::Ticker(ticker) => {
                            println!("Ticker: {} = {}", ticker.s, ticker.c);
                        }
                        BackpackMessage::OrderBook(order_book) => {
                            println!(
                                "OrderBook: {} - {} bids, {} asks",
                                order_book.s,
                                order_book.b.len(),
                                order_book.a.len()
                            );
                        }
                        BackpackMessage::Trade(trade) => {
                            println!("Trade: {} - {} @ {}", trade.s, trade.q, trade.p);
                        }
                        BackpackMessage::Subscription { status, params, .. } => {
                            println!("Subscription {}: {:?}", status, params);
                        }
                        _ => {
                            println!("Other message: {:?}", message);
                        }
                    }
                    message_count += 1;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {:?}", e);
                    break;
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Clean up
    backpack_ws.close_websocket().await?;
    println!("WebSocket connection closed");

    Ok(())
}
