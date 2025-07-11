use crate::core::errors::ExchangeError;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub struct WebSocketManager {
    url: String,
}

impl WebSocketManager {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Start a WebSocket stream with automatic reconnection
    pub async fn start_stream<F, T>(
        &self,
        message_parser: F,
    ) -> Result<mpsc::Receiver<T>, ExchangeError>
    where
        F: Fn(Value) -> Option<T> + Send + Sync + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel(1000);
        let url = self.url.clone();

        tokio::spawn(async move {
            let mut reconnect_delay = 1;

            loop {
                match Self::connect_and_listen(&url, &message_parser, &tx).await {
                    Ok(_) => {
                        reconnect_delay = 1; // Reset delay on successful connection
                    }
                    Err(e) => {
                        eprintln!("WebSocket connection failed: {:?}", e);
                        eprintln!("Reconnecting in {} seconds...", reconnect_delay);

                        sleep(Duration::from_secs(reconnect_delay)).await;
                        reconnect_delay = std::cmp::min(reconnect_delay * 2, 60);
                        // Exponential backoff, max 60s
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Subscribe to additional streams on an existing connection
    pub async fn subscribe_streams(&self, streams: Vec<String>) -> Result<(), ExchangeError> {
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("Failed to connect: {}", e)))?;

        let (mut write, _) = ws_stream.split();

        let subscription = json!({
            "method": "SUBSCRIBE",
            "params": streams,
            "id": 1
        });

        write
            .send(Message::Text(subscription.to_string()))
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("Failed to subscribe: {}", e)))?;

        Ok(())
    }

    /// Unsubscribe from streams
    pub async fn unsubscribe_streams(&self, streams: Vec<String>) -> Result<(), ExchangeError> {
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("Failed to connect: {}", e)))?;

        let (mut write, _) = ws_stream.split();

        let unsubscription = json!({
            "method": "UNSUBSCRIBE",
            "params": streams,
            "id": 1
        });

        write
            .send(Message::Text(unsubscription.to_string()))
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("Failed to unsubscribe: {}", e)))?;

        Ok(())
    }

    async fn connect_and_listen<F, T>(
        url: &str,
        message_parser: &F,
        tx: &mpsc::Sender<T>,
    ) -> Result<(), ExchangeError>
    where
        F: Fn(Value) -> Option<T> + Send + Sync,
        T: Send,
    {
        let (ws_stream, _) = connect_async(url).await.map_err(|e| {
            ExchangeError::NetworkError(format!("Failed to connect to {}: {}", url, e))
        })?;

        let (mut write, mut read) = ws_stream.split();

        // Message processing loop
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(json_value) = serde_json::from_str::<Value>(&text) {
                        if let Some(parsed_data) = message_parser(json_value) {
                            if tx.send(parsed_data).await.is_err() {
                                break; // Receiver dropped
                            }
                        }
                    }
                }
                Ok(Message::Ping(payload)) => {
                    // Respond to ping with pong (Binance requirement)
                    if write.send(Message::Pong(payload)).await.is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Err(e) => {
                    return Err(ExchangeError::NetworkError(format!(
                        "WebSocket error: {}",
                        e
                    )));
                }
                _ => {}
            }
        }

        Ok(())
    }
}

/// Specialized WebSocket manager for Bybit V5 API
pub struct BybitWebSocketManager {
    url: String,
}

impl BybitWebSocketManager {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Start a Bybit WebSocket stream with V5 subscription protocol
    pub async fn start_stream_with_subscriptions<F, T>(
        &self,
        streams: Vec<String>,
        message_parser: F,
    ) -> Result<mpsc::Receiver<T>, ExchangeError>
    where
        F: Fn(Value) -> Option<T> + Send + Sync + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel(1000);
        let url = self.url.clone();

        tokio::spawn(async move {
            let mut reconnect_delay = 1;

            loop {
                match Self::connect_and_subscribe(&url, &streams, &message_parser, &tx).await {
                    Ok(_) => {
                        reconnect_delay = 1; // Reset delay on successful connection
                    }
                    Err(e) => {
                        eprintln!("WebSocket connection failed: {:?}", e);
                        eprintln!("Reconnecting in {} seconds...", reconnect_delay);

                        sleep(Duration::from_secs(reconnect_delay)).await;
                        reconnect_delay = std::cmp::min(reconnect_delay * 2, 60);
                    }
                }
            }
        });

        Ok(rx)
    }

    async fn connect_and_subscribe<F, T>(
        url: &str,
        streams: &[String],
        message_parser: &F,
        tx: &mpsc::Sender<T>,
    ) -> Result<(), ExchangeError>
    where
        F: Fn(Value) -> Option<T> + Send + Sync,
        T: Send,
    {
        let (ws_stream, _) = connect_async(url).await.map_err(|e| {
            ExchangeError::NetworkError(format!("Failed to connect to {}: {}", url, e))
        })?;

        let (mut write, mut read) = ws_stream.split();

        // Send subscription message for Bybit V5
        if !streams.is_empty() {
            let subscription = json!({
                "op": "subscribe",
                "args": streams
            });

            write
                .send(Message::Text(subscription.to_string()))
                .await
                .map_err(|e| {
                    ExchangeError::NetworkError(format!("Failed to send subscription: {}", e))
                })?;
        }

        // Setup ping interval for Bybit V5 keep-alive
        let mut ping_interval = tokio::time::interval(Duration::from_secs(20));
        ping_interval.tick().await; // Skip the first immediate tick

        // Message processing loop with periodic pings
        loop {
            tokio::select! {
                // Handle incoming messages
                message = read.next() => {
                    match message {
                        Some(Ok(Message::Text(text))) => {
                            if let Ok(json_value) = serde_json::from_str::<Value>(&text) {
                                // Handle subscription confirmation
                                if let Some(op) = json_value.get("op") {
                                    if op == "subscribe" && json_value.get("success").is_some_and(|s| s.as_bool().unwrap_or(false)) {
                                        continue; // Subscription confirmed, continue listening
                                    }
                                    if op == "pong" {
                                        continue; // Pong response, continue listening
                                    }
                                }

                                if let Some(parsed_data) = message_parser(json_value) {
                                    if tx.send(parsed_data).await.is_err() {
                                        break; // Receiver dropped
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Ping(payload))) => {
                            if write.send(Message::Pong(payload)).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            break;
                        }
                        Some(Err(e)) => {
                            return Err(ExchangeError::NetworkError(format!(
                                "WebSocket error: {}",
                                e
                            )));
                        }
                        None => {
                            break; // Stream ended
                        }
                        _ => {}
                    }
                }
                // Send periodic ping
                _ = ping_interval.tick() => {
                    let ping_msg = json!({"op": "ping"});
                    if write.send(Message::Text(ping_msg.to_string())).await.is_err() {
                        break; // Connection closed
                    }
                }
            }
        }

        Ok(())
    }
}

/// Helper function to build Binance WebSocket URLs for combined streams
pub fn build_binance_stream_url(base_url: &str, streams: &[String]) -> String {
    if streams.is_empty() {
        return base_url.to_string();
    }

    // For combined streams, Binance expects /ws/stream?streams=...
    let base = base_url
        .strip_suffix("/ws")
        .map_or(base_url, |stripped| stripped);
    format!("{}/stream?streams={}", base, streams.join("/"))
}

/// Helper function to build Binance WebSocket URL for a single raw stream
pub fn build_binance_raw_stream_url(base_url: &str, stream: &str) -> String {
    format!("{}/ws/{}", base_url, stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_manager_creation() {
        let manager = WebSocketManager::new("wss://stream.binance.com:9443".to_string());
        assert_eq!(manager.url, "wss://stream.binance.com:9443");
    }

    #[test]
    fn test_build_combined_stream_url() {
        let base_url = "wss://stream.binance.com:9443/ws";
        let streams = vec!["btcusdt@ticker".to_string(), "ethusdt@ticker".to_string()];
        let url = build_binance_stream_url(base_url, &streams);
        assert_eq!(
            url,
            "wss://stream.binance.com:9443/stream?streams=btcusdt@ticker/ethusdt@ticker"
        );
    }

    #[test]
    fn test_build_combined_stream_url_without_ws() {
        let base_url = "wss://stream.binance.com:9443";
        let streams = vec!["btcusdt@ticker".to_string(), "ethusdt@ticker".to_string()];
        let url = build_binance_stream_url(base_url, &streams);
        assert_eq!(
            url,
            "wss://stream.binance.com:9443/stream?streams=btcusdt@ticker/ethusdt@ticker"
        );
    }

    #[test]
    fn test_build_raw_stream_url() {
        let base_url = "wss://stream.binance.com:9443";
        let stream = "btcusdt@ticker";
        let url = build_binance_raw_stream_url(base_url, stream);
        assert_eq!(url, "wss://stream.binance.com:9443/ws/btcusdt@ticker");
    }

    #[test]
    fn test_empty_streams() {
        let base_url = "wss://stream.binance.com:9443";
        let streams: Vec<String> = vec![];
        let url = build_binance_stream_url(base_url, &streams);
        assert_eq!(url, "wss://stream.binance.com:9443");
    }

    #[test]
    fn test_message_parser() {
        let parser = |value: Value| -> Option<String> {
            // Handle combined stream format: {"stream":"streamName","data":{...}}
            if let Some(stream_name) = value.get("stream") {
                return stream_name.as_str().map(|s| s.to_string());
            }
            // Handle raw stream format: direct data
            value
                .get("s")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        };

        // Test combined stream format
        let combined_json = json!({"stream": "btcusdt@ticker", "data": {"s": "BTCUSDT"}});
        assert_eq!(parser(combined_json), Some("btcusdt@ticker".to_string()));

        // Test raw stream format
        let raw_json = json!({"s": "BTCUSDT", "c": "50000"});
        assert_eq!(parser(raw_json), Some("BTCUSDT".to_string()));
    }
}
