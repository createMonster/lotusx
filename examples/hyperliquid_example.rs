use lotusx::core::traits::{AccountInfo, MarketDataSource, OrderPlacer};
use lotusx::core::types::{
    KlineInterval, OrderRequest, OrderSide, OrderType, SubscriptionType, TimeInForce,
    WebSocketConfig,
};
use lotusx::exchanges::hyperliquid::HyperliquidClient;
use std::error::Error;
use tokio::time::{timeout, Duration};

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Hyperliquid API Example");
    println!("========================");

    // Example 1: Read-only client for market data
    println!("=== Read-only Market Data Example ===");
    let client = HyperliquidClient::read_only(true); // Use testnet

    println!("Testnet mode: {}", client.is_testnet());
    println!("Can sign transactions: {}", client.can_sign());
    println!("WebSocket URL: {}", client.get_websocket_url());

    // Get available markets
    match client.get_markets().await {
        Ok(markets) => {
            println!("Available markets: {}", markets.len());
            for (i, market) in markets.iter().take(5).enumerate() {
                println!(
                    "  {}. {} (status: {})",
                    i + 1,
                    market.symbol.symbol,
                    market.status
                );
            }
        }
        Err(e) => println!("Error getting markets: {}", e),
    }

    // Example 2: Authenticated client with private key
    println!("\n=== Authenticated Client Example ===");

    // You would use your actual private key here
    let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";

    match HyperliquidClient::with_private_key(private_key, true) {
        Ok(auth_client) => {
            println!("Authentication successful!");
            println!("Wallet address: {:?}", auth_client.wallet_address());
            println!("Can sign transactions: {}", auth_client.can_sign());

            // Example: Get account balance
            if auth_client.wallet_address().is_some() {
                match auth_client.get_account_balance().await {
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

                // Example: Get positions
                match auth_client.get_positions().await {
                    Ok(positions) => {
                        println!("Open positions: {}", positions.len());
                        for position in positions {
                            println!(
                                "  {}: {:?} {} (PnL: {})",
                                position.symbol,
                                position.position_side,
                                position.position_amount,
                                position.unrealized_pnl
                            );
                        }
                    }
                    Err(e) => println!("Error getting positions: {}", e),
                }
            }

            // Example: Place a limit order (this will likely fail on testnet without funds)
            let order = OrderRequest {
                symbol: "BTC".to_string(),
                side: OrderSide::Buy,
                order_type: OrderType::Limit,
                quantity: "0.001".to_string(),
                price: Some("30000".to_string()),
                time_in_force: Some(TimeInForce::GTC),
                stop_price: None,
            };

            println!("\nAttempting to place test order...");
            match auth_client.place_order(order).await {
                Ok(response) => {
                    println!("Order placed successfully!");
                    println!("Order ID: {}", response.order_id);
                    println!("Status: {}", response.status);

                    // Example: Cancel the order
                    match auth_client
                        .cancel_order("BTC".to_string(), response.order_id)
                        .await
                    {
                        Ok(_) => println!("Order cancelled successfully!"),
                        Err(e) => println!("Error cancelling order: {}", e),
                    }
                }
                Err(e) => println!("Error placing order: {}", e),
            }
        }
        Err(e) => println!("Authentication failed: {}", e),
    }

    // Example 3: WebSocket Market Data Subscription
    println!("\n=== WebSocket Market Data Example ===");

    let ws_client = HyperliquidClient::read_only(true);
    let symbols = vec!["BTC".to_string(), "ETH".to_string()];
    let subscription_types = vec![
        SubscriptionType::Ticker,
        SubscriptionType::OrderBook { depth: Some(10) },
        SubscriptionType::Trades,
        SubscriptionType::Klines {
            interval: KlineInterval::Minutes1,
        },
    ];

    let ws_config = WebSocketConfig {
        auto_reconnect: true,
        max_reconnect_attempts: Some(3),
        ping_interval: Some(30),
    };

    println!("Subscribing to WebSocket market data for BTC and ETH...");
    match ws_client
        .subscribe_market_data(symbols, subscription_types, Some(ws_config))
        .await
    {
        Ok(mut receiver) => {
            println!("✓ WebSocket connection established!");
            println!("Listening for market data (will timeout after 10 seconds)...");

            let mut message_count = 0;
            let listen_duration = Duration::from_secs(10);

            match timeout(listen_duration, async {
                while let Some(market_data) = receiver.recv().await {
                    message_count += 1;
                    match market_data {
                        lotusx::core::types::MarketDataType::Ticker(ticker) => {
                            println!("📊 Ticker - {}: ${}", ticker.symbol, ticker.price);
                        }
                        lotusx::core::types::MarketDataType::OrderBook(book) => {
                            println!(
                                "📖 OrderBook - {}: {} bids, {} asks",
                                book.symbol,
                                book.bids.len(),
                                book.asks.len()
                            );
                        }
                        lotusx::core::types::MarketDataType::Trade(trade) => {
                            println!(
                                "💱 Trade - {}: {} @ ${}",
                                trade.symbol, trade.quantity, trade.price
                            );
                        }
                        lotusx::core::types::MarketDataType::Kline(kline) => {
                            println!(
                                "📈 Kline - {}: O=${} H=${} L=${} C=${}",
                                kline.symbol,
                                kline.open_price,
                                kline.high_price,
                                kline.low_price,
                                kline.close_price
                            );
                        }
                    }

                    // Stop after receiving 5 messages to keep example short
                    if message_count >= 5 {
                        break;
                    }
                }
            })
            .await
            {
                Ok(_) => println!("✓ Received {} market data messages", message_count),
                Err(_) => println!("⏰ WebSocket listening timed out after 10 seconds"),
            }
        }
        Err(e) => println!("❌ WebSocket connection failed: {}", e),
    }

    println!("\n=== Trait Implementation Demo ===");
    println!("✓ MarketDataSource trait implemented (including WebSocket)");
    println!("✓ OrderPlacer trait implemented");
    println!("✓ AccountInfo trait implemented");
    println!("✓ ExchangeConnector trait implemented (composite)");

    println!("\n✨ Example completed!");
    Ok(())
}
