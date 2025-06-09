use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing Binance WebSocket Connectivity");
    
    let urls = vec![
        "wss://stream.binance.com:443/ws/btcusdt@ticker",
        "wss://stream.binance.com:9443/ws/btcusdt@ticker", 
        "wss://stream.binance.com/ws/btcusdt@ticker", // Without port
        "wss://data-stream.binance.vision/ws/btcusdt@ticker", // Alternative endpoint
    ];
    
    for url in urls {
        println!("\n🌐 Testing connection to: {}", url);
        
        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                println!("✅ Connection successful!");
                let (mut write, mut read) = ws_stream.split();
                
                // Try to receive a few messages
                let mut count = 0;
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            println!("📊 Received message: {}", &text[..100.min(text.len())]);
                            count += 1;
                            if count >= 2 {
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            println!("🔒 Connection closed by server");
                            break;
                        }
                        Err(e) => {
                            println!("❌ Error receiving message: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
                
                println!("✅ Successfully received {} messages from {}", count, url);
                return Ok(()); // Exit on first successful connection
            }
            Err(e) => {
                println!("❌ Connection failed: {}", e);
                continue;
            }
        }
    }
    
    println!("\n❌ All connection attempts failed. Possible issues:");
    println!("1. Firewall or network restrictions");
    println!("2. Regional blocking (some countries block Binance)");
    println!("3. ISP blocking WebSocket connections");
    println!("4. Corporate network restrictions");
    println!("\n💡 Try using a VPN or different network");
    
    Ok(())
} 