use futures_util::{SinkExt, StreamExt};
use lotusx::core::config::ExchangeConfig;
use lotusx::exchanges::backpack::BackpackConnector;
use serde_json::{json, Value};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Backpack Exchange WebSocket Streams Example");
    println!("==============================================");
    println!("📚 Based on: https://docs.backpack.exchange/#tag/Streams");

    // Example 1: Public Streams (No authentication required)
    println!("\n📡 Attempting Public WebSocket Streams...");

    match run_public_streams().await {
        Ok(_) => println!("✅ Public streams completed successfully"),
        Err(e) => {
            println!("⚠️  WebSocket connection failed: {}", e);
            println!("📖 Showing example message formats instead...");
            demonstrate_public_message_formats();
        }
    }

    // Example 2: Private Streams (Authentication required)
    println!("\n🔐 Attempting Private WebSocket Streams...");

    match ExchangeConfig::from_env_file("BACKPACK") {
        Ok(config) => match run_private_streams(config).await {
            Ok(_) => println!("✅ Private streams completed successfully"),
            Err(e) => {
                println!("⚠️  Private WebSocket connection failed: {}", e);
                println!("📖 Showing example private message formats instead...");
                demonstrate_private_message_formats();
            }
        },
        Err(e) => {
            println!("⚠️  No credentials found for private streams: {}", e);
            println!("📖 Showing example private message formats instead...");
            demonstrate_private_message_formats();
        }
    }

    println!("\n✅ WebSocket streams example completed!");
    println!("\n📋 Summary:");
    println!(
        "   • Public streams: ticker, bookTicker, depth, trade, kline, markPrice, openInterest"
    );
    println!("   • Private streams: orderUpdate, positionUpdate, rfqUpdate");
    println!("   • WebSocket URL: wss://ws.backpack.exchange");
    println!("   • Subscription format: SUBSCRIBE method with params array");
    println!("   • Private streams require ED25519 signature authentication");

    Ok(())
}

/// Demonstrates public WebSocket streams that don't require authentication
async fn run_public_streams() -> Result<(), Box<dyn std::error::Error>> {
    let ws_url = "wss://ws.backpack.exchange";

    println!("🔗 Connecting to: {}", ws_url);

    // Create a more robust connection with better error handling
    let (ws_stream, _response) = match connect_async(ws_url).await {
        Ok((stream, response)) => {
            println!("✅ Connected successfully, status: {}", response.status());
            (stream, response)
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            println!("💡 This might be due to:");
            println!("   • Network connectivity issues");
            println!("   • Firewall blocking the connection");
            println!("   • The exchange endpoint being temporarily unavailable");
            println!("   • TLS configuration issues");
            return Err(e.into());
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to multiple public streams based on official documentation
    let subscription = json!({
        "method": "SUBSCRIBE",
        "params": [
            "ticker.SOL_USDC",           // 24hr ticker statistics
            "bookTicker.SOL_USDC",       // Best bid/ask updates
            "depth.SOL_USDC",            // Order book depth updates
            "trade.SOL_USDC",            // Public trade data
            "kline.1m.SOL_USDC",         // 1-minute kline data
            "markPrice.SOL_USDC",        // Mark price updates
            "openInterest.SOL_USDC_PERP" // Open interest updates
        ],
        "id": 1
    });

    let subscription_msg = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_msg)).await?;

    println!("📨 Subscribed to public streams for SOL_USDC");
    println!("📊 Receiving live market data...\n");

    let mut message_count = 0;
    let mut timeout_count = 0;

    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                message_count += 1;

                if let Ok(data) = serde_json::from_str::<Value>(&text) {
                    handle_public_message(data, message_count);

                    // Limit output for demo purposes
                    if message_count >= 20 {
                        println!(
                            "📈 Received {} messages, stopping public streams...",
                            message_count
                        );
                        break;
                    }
                }
            }
            Message::Close(_) => {
                println!("🔌 WebSocket connection closed");
                break;
            }
            Message::Ping(_) => {
                println!("🏓 Received ping, sending pong");
                write.send(Message::Pong(vec![])).await?;
            }
            _ => {}
        }

        // Add timeout to prevent hanging
        timeout_count += 1;
        if timeout_count > 100 {
            println!("⏰ Timeout reached, stopping...");
            break;
        }
    }

    Ok(())
}

/// Handles and displays public WebSocket messages according to Backpack API format
#[allow(clippy::too_many_lines)]
fn handle_public_message(data: Value, count: usize) {
    if let Some(event_type) = data.get("e").and_then(|e| e.as_str()) {
        match event_type {
            "ticker" => {
                // Format: {"e":"ticker","E":1694687692980000,"s":"SOL_USD","o":"18.75","c":"19.24","h":"19.80","l":"18.50","v":"32123","V":"928190","n":93828}
                if let (Some(symbol), Some(last_price), Some(volume)) = (
                    data.get("s").and_then(|s| s.as_str()),
                    data.get("c").and_then(|p| p.as_str()),
                    data.get("v").and_then(|v| v.as_str()),
                ) {
                    let high = data.get("h").and_then(|h| h.as_str()).unwrap_or("N/A");
                    let low = data.get("l").and_then(|l| l.as_str()).unwrap_or("N/A");
                    println!(
                        "📊 #{:02} Ticker: {} Last: ${} High: ${} Low: ${} Vol: {}",
                        count, symbol, last_price, high, low, volume
                    );
                }
            }
            "bookTicker" => {
                // Format: {"e":"bookTicker","E":1694687965941000,"s":"SOL_USDC","a":"18.70","A":"1.000","b":"18.67","B":"2.000","u":"111063070525358080","T":1694687965940999}
                if let (Some(symbol), Some(bid), Some(ask)) = (
                    data.get("s").and_then(|s| s.as_str()),
                    data.get("b").and_then(|b| b.as_str()),
                    data.get("a").and_then(|a| a.as_str()),
                ) {
                    let bid_qty = data.get("B").and_then(|b| b.as_str()).unwrap_or("0");
                    let ask_qty = data.get("A").and_then(|a| a.as_str()).unwrap_or("0");
                    println!(
                        "📖 #{:02} BookTicker: {} Bid: ${} ({}) Ask: ${} ({})",
                        count, symbol, bid, bid_qty, ask, ask_qty
                    );
                }
            }
            "depth" => {
                // Format: {"e":"depth","E":1694687965941000,"s":"SOL_USDC","a":[["18.70","0.000"]],"b":[["18.67","0.832"],["18.68","0.000"]],"U":94978271,"u":94978271,"T":1694687965940999}
                if let Some(symbol) = data.get("s").and_then(|s| s.as_str()) {
                    let asks_count = data
                        .get("a")
                        .and_then(|a| a.as_array())
                        .map_or(0, |a| a.len());
                    let bids_count = data
                        .get("b")
                        .and_then(|b| b.as_array())
                        .map_or(0, |b| b.len());
                    let update_id = data.get("u").and_then(|u| u.as_str()).unwrap_or("N/A");
                    println!(
                        "📋 #{:02} Depth: {} Updates: {} ({} asks, {} bids)",
                        count, symbol, update_id, asks_count, bids_count
                    );
                }
            }
            "trade" => {
                // Format: {"e":"trade","E":1694688638091000,"s":"SOL_USDC","p":"18.68","q":"0.122","b":"111063114377265150","a":"111063114585735170","t":12345,"T":1694688638089000,"m":true}
                if let (Some(symbol), Some(price), Some(quantity)) = (
                    data.get("s").and_then(|s| s.as_str()),
                    data.get("p").and_then(|p| p.as_str()),
                    data.get("q").and_then(|q| q.as_str()),
                ) {
                    let is_buyer_maker = data.get("m").and_then(|m| m.as_bool()).unwrap_or(false);
                    let trade_id = data.get("t").and_then(|t| t.as_u64()).unwrap_or(0);
                    let side = if is_buyer_maker { "Sell" } else { "Buy" };
                    println!(
                        "🔄 #{:02} Trade: {} {} {} @ ${} ID: {}",
                        count, symbol, side, quantity, price, trade_id
                    );
                }
            }
            "kline" => {
                // Format: {"e":"kline","E":1694687692980000,"s":"SOL_USD","t":123400000,"T":123460000,"o":"18.75","c":"19.25","h":"19.80","l":"18.50","v":"32123","n":93828,"X":false}
                if let (Some(symbol), Some(open), Some(close), Some(high), Some(low)) = (
                    data.get("s").and_then(|s| s.as_str()),
                    data.get("o").and_then(|o| o.as_str()),
                    data.get("c").and_then(|c| c.as_str()),
                    data.get("h").and_then(|h| h.as_str()),
                    data.get("l").and_then(|l| l.as_str()),
                ) {
                    let is_closed = data.get("X").and_then(|x| x.as_bool()).unwrap_or(false);
                    let status = if is_closed { "Closed" } else { "Open" };
                    println!(
                        "📈 #{:02} Kline: {} OHLC: ${}/{}/{}/{} Status: {}",
                        count, symbol, open, high, low, close, status
                    );
                }
            }
            "markPrice" => {
                // Format: {"e":"markPrice","E":1694687965941000,"s":"SOL_USDC","p":"18.70","f":"1.70","i":"19.70","n":1694687965941000}
                if let (Some(symbol), Some(mark_price)) = (
                    data.get("s").and_then(|s| s.as_str()),
                    data.get("p").and_then(|p| p.as_str()),
                ) {
                    let funding_rate = data.get("f").and_then(|f| f.as_str()).unwrap_or("N/A");
                    let index_price = data.get("i").and_then(|i| i.as_str()).unwrap_or("N/A");
                    println!(
                        "💰 #{:02} MarkPrice: {} Mark: ${} Index: ${} Funding: {}%",
                        count, symbol, mark_price, index_price, funding_rate
                    );
                }
            }
            "openInterest" => {
                // Format: {"e":"openInterest","E":1694687965941000,"s":"SOL_USDC_PERP","o":"100"}
                if let (Some(symbol), Some(open_interest)) = (
                    data.get("s").and_then(|s| s.as_str()),
                    data.get("o").and_then(|o| o.as_str()),
                ) {
                    println!(
                        "📊 #{:02} OpenInterest: {} {}",
                        count, symbol, open_interest
                    );
                }
            }
            _ => {
                println!("🔔 #{:02} Unknown Event: {}", count, event_type);
            }
        }
    } else if data.get("result").is_some() {
        println!("✅ Subscription confirmed: {:?}", data.get("result"));
    } else if data.get("error").is_some() {
        println!("❌ Subscription error: {:?}", data.get("error"));
    }
}

/// Demonstrates private WebSocket streams that require authentication
async fn run_private_streams(config: ExchangeConfig) -> Result<(), Box<dyn std::error::Error>> {
    let connector = BackpackConnector::new(config)?;
    let ws_url = "wss://ws.backpack.exchange";

    println!("🔗 Connecting to authenticated WebSocket: {}", ws_url);

    // Create a more robust connection with better error handling
    let (ws_stream, _response) = match connect_async(ws_url).await {
        Ok((stream, response)) => {
            println!(
                "✅ Authenticated connection successful, status: {}",
                response.status()
            );
            (stream, response)
        }
        Err(e) => {
            println!("❌ Authenticated connection failed: {}", e);
            println!("💡 This might be due to:");
            println!("   • Network connectivity issues");
            println!("   • Firewall blocking the connection");
            println!("   • The exchange endpoint being temporarily unavailable");
            println!("   • TLS configuration issues");
            return Err(e.into());
        }
    };

    let (mut write, mut read) = ws_stream.split();

    // Create authenticated subscription for private streams
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as i64;
    let window = 5000;

    // Sign the subscription request according to Backpack API docs
    let instruction = "subscribe";
    let params_str = "";
    let signature = connector.generate_signature(instruction, params_str, timestamp, window)?;

    let subscription = json!({
        "method": "SUBSCRIBE",
        "params": [
            "account.orderUpdate",           // All order updates
            "account.orderUpdate.SOL_USDC",  // Order updates for specific symbol
            "account.position"               // Position updates
        ],
        "signature": {
            "instruction": instruction,
            "timestamp": timestamp,
            "window": window,
            "signature": signature
        },
        "id": 2
    });

    let subscription_msg = serde_json::to_string(&subscription)?;
    write.send(Message::Text(subscription_msg)).await?;

    println!("📨 Subscribed to private streams");
    println!("🔐 Receiving account updates...\n");

    let mut message_count = 0;
    let mut timeout_count = 0;

    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                message_count += 1;

                if let Ok(data) = serde_json::from_str::<Value>(&text) {
                    handle_private_message(data, message_count);

                    // Limit output for demo purposes
                    if message_count >= 10 {
                        println!(
                            "🔐 Received {} private messages, stopping...",
                            message_count
                        );
                        break;
                    }
                }
            }
            Message::Close(_) => {
                println!("🔌 Private WebSocket connection closed");
                break;
            }
            Message::Ping(_) => {
                println!("🏓 Received ping, sending pong");
                write.send(Message::Pong(vec![])).await?;
            }
            _ => {}
        }

        // Add timeout to prevent hanging
        timeout_count += 1;
        if timeout_count > 50 {
            println!("⏰ Timeout reached for private streams, stopping...");
            break;
        }
    }

    Ok(())
}

/// Handles and displays private WebSocket messages according to Backpack API format
fn handle_private_message(data: Value, count: usize) {
    if let Some(event_type) = data.get("e").and_then(|e| e.as_str()) {
        match event_type {
            "orderUpdate" => {
                // Order update format from documentation
                if let (Some(symbol), Some(side), Some(status)) = (
                    data.get("s").and_then(|s| s.as_str()),
                    data.get("S").and_then(|s| s.as_str()),
                    data.get("X").and_then(|x| x.as_str()),
                ) {
                    let price = data.get("p").and_then(|p| p.as_str()).unwrap_or("Market");
                    let quantity = data.get("q").and_then(|q| q.as_str()).unwrap_or("0");
                    let order_id = data.get("i").and_then(|i| i.as_str()).unwrap_or("N/A");
                    println!(
                        "📋 #{:02} OrderUpdate: {} {} {} @ {} Status: {} ID: {}",
                        count, symbol, side, quantity, price, status, order_id
                    );
                }
            }
            "positionUpdate" => {
                // Position update format from documentation
                if let (Some(symbol), Some(side)) = (
                    data.get("s").and_then(|s| s.as_str()),
                    data.get("S").and_then(|s| s.as_str()),
                ) {
                    let size = data.get("q").and_then(|q| q.as_str()).unwrap_or("0");
                    let entry_price = data.get("ep").and_then(|ep| ep.as_str()).unwrap_or("0");
                    let unrealized_pnl = data.get("up").and_then(|up| up.as_str()).unwrap_or("0");
                    println!(
                        "📍 #{:02} PositionUpdate: {} {} {} @ ${} PnL: ${}",
                        count, symbol, side, size, entry_price, unrealized_pnl
                    );
                }
            }
            "rfqUpdate" | "rfqActive" | "rfqAccepted" | "rfqFilled" => {
                // RFQ update formats from documentation
                if let Some(symbol) = data.get("s").and_then(|s| s.as_str()) {
                    let rfq_id = data.get("R").and_then(|r| r.as_str()).unwrap_or("N/A");
                    let side = data.get("S").and_then(|s| s.as_str()).unwrap_or("N/A");
                    let status = data.get("X").and_then(|x| x.as_str()).unwrap_or("N/A");
                    println!(
                        "🎯 #{:02} RFQ: {} {} ID: {} Side: {} Status: {}",
                        count, event_type, symbol, rfq_id, side, status
                    );
                }
            }
            _ => {
                println!("🔔 #{:02} Private Event: {}", count, event_type);
            }
        }
    } else if data.get("result").is_some() {
        println!(
            "✅ Private subscription confirmed: {:?}",
            data.get("result")
        );
    } else if data.get("error").is_some() {
        println!("❌ Private subscription error: {:?}", data.get("error"));
    }
}

/// Shows example public message formats when WebSocket is not available
fn demonstrate_public_message_formats() {
    println!("\n📖 Example Public WebSocket Message Formats:");
    println!("════════════════════════════════════════════");

    println!("\n📊 Ticker Stream (ticker.SOL_USDC):");
    println!("   {{\"e\":\"ticker\",\"E\":1694687692980000,\"s\":\"SOL_USDC\",\"o\":\"18.75\",\"c\":\"19.24\",\"h\":\"19.80\",\"l\":\"18.50\",\"v\":\"32123\",\"V\":\"928190\",\"n\":93828}}");

    println!("\n📖 Book Ticker Stream (bookTicker.SOL_USDC):");
    println!("   {{\"e\":\"bookTicker\",\"E\":1694687965941000,\"s\":\"SOL_USDC\",\"a\":\"18.70\",\"A\":\"1.000\",\"b\":\"18.67\",\"B\":\"2.000\",\"u\":\"111063070525358080\"}}");

    println!("\n📋 Depth Stream (depth.SOL_USDC):");
    println!("   {{\"e\":\"depth\",\"E\":1694687965941000,\"s\":\"SOL_USDC\",\"a\":[[\"18.70\",\"0.000\"]],\"b\":[[\"18.67\",\"0.832\"]],\"U\":94978271,\"u\":94978271}}");

    println!("\n🔄 Trade Stream (trade.SOL_USDC):");
    println!("   {{\"e\":\"trade\",\"E\":1694688638091000,\"s\":\"SOL_USDC\",\"p\":\"18.68\",\"q\":\"0.122\",\"t\":12345,\"m\":true}}");

    println!("\n📈 Kline Stream (kline.1m.SOL_USDC):");
    println!("   {{\"e\":\"kline\",\"E\":1694687692980000,\"s\":\"SOL_USDC\",\"o\":\"18.75\",\"c\":\"19.25\",\"h\":\"19.80\",\"l\":\"18.50\",\"v\":\"32123\",\"X\":false}}");

    println!("\n💰 Mark Price Stream (markPrice.SOL_USDC):");
    println!("   {{\"e\":\"markPrice\",\"E\":1694687965941000,\"s\":\"SOL_USDC\",\"p\":\"18.70\",\"f\":\"1.70\",\"i\":\"19.70\"}}");
}

/// Shows example private message formats when WebSocket is not available
fn demonstrate_private_message_formats() {
    println!("\n🔐 Example Private WebSocket Message Formats:");
    println!("═════════════════════════════════════════════");

    println!("\n📋 Order Update Stream (account.orderUpdate):");
    println!("   {{\"e\":\"orderUpdate\",\"E\":1694688638091000,\"s\":\"SOL_USDC\",\"S\":\"Bid\",\"q\":\"1.5\",\"p\":\"18.50\",\"X\":\"New\",\"i\":\"123456789\"}}");

    println!("\n📍 Position Update Stream (account.position):");
    println!("   {{\"e\":\"positionUpdate\",\"E\":1694688638091000,\"s\":\"SOL_USDC_PERP\",\"S\":\"Long\",\"q\":\"10.0\",\"ep\":\"18.50\",\"up\":\"25.00\"}}");

    println!("\n🎯 RFQ Update Stream (account.rfqUpdate):");
    println!("   {{\"e\":\"rfqActive\",\"E\":1694688638091000,\"s\":\"SOL_USDC_RFQ\",\"R\":\"113392053149171712\",\"S\":\"Bid\",\"X\":\"Active\"}}");

    println!("\n🔑 Authentication:");
    println!("   Private streams require signature in subscription:");
    println!("   {{\"method\":\"SUBSCRIBE\",\"params\":[\"account.orderUpdate\"],\"signature\":{{\"instruction\":\"subscribe\",\"timestamp\":1694688638091,\"window\":5000,\"signature\":\"base64_signature\"}}}}");
}
