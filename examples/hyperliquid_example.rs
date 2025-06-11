use lotusx::core::traits::{AccountInfo, MarketDataSource, OrderPlacer};
use lotusx::core::types::{OrderRequest, OrderSide, OrderType, TimeInForce};
use lotusx::exchanges::hyperliquid::HyperliquidClient;
use std::error::Error;

#[tokio::main]
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

    println!("\n=== Trait Implementation Demo ===");
    println!("✓ MarketDataSource trait implemented");
    println!("✓ OrderPlacer trait implemented");
    println!("✓ AccountInfo trait implemented");
    println!("✓ ExchangeConnector trait implemented (composite)");

    println!("\n✨ Example completed!");
    Ok(())
}
