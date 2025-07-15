use crate::core::errors::ExchangeError;
use crate::core::kernel::codec::WsCodec;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, instrument, warn};

/// HFT-optimized WebSocket configuration
#[derive(Debug, Clone)]
pub struct WsConfig {
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,
    /// Message buffer size for high-frequency trading
    pub message_buffer_size: usize,
    /// Max reconnection attempts
    pub max_reconnect_attempts: u32,
    /// Reconnection delay in milliseconds
    pub reconnect_delay_ms: u64,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: 10_000,    // 10 seconds
            heartbeat_interval_ms: 30_000, // 30 seconds
            message_buffer_size: 1024,     // 1024 messages buffer
            max_reconnect_attempts: 5,
            reconnect_delay_ms: 1_000, // 1 second
        }
    }
}

impl WsConfig {
    /// Create HFT-optimized configuration with low latency settings
    pub fn hft_optimized() -> Self {
        Self {
            connect_timeout_ms: 5_000,     // 5 seconds
            heartbeat_interval_ms: 15_000, // 15 seconds
            message_buffer_size: 4096,     // 4096 messages buffer
            max_reconnect_attempts: 10,
            reconnect_delay_ms: 100, // 100ms reconnect delay
        }
    }
}

/// WebSocket session trait - pure transport layer
#[async_trait]
pub trait WsSession<C: WsCodec>: Send + Sync {
    /// Connect to the WebSocket
    async fn connect(&mut self) -> Result<(), ExchangeError>;

    /// Send a raw message
    async fn send_raw(&mut self, msg: Message) -> Result<(), ExchangeError>;

    /// Receive the next raw message
    async fn next_raw(&mut self) -> Option<Result<Message, ExchangeError>>;

    /// Close the connection
    async fn close(&mut self) -> Result<(), ExchangeError>;

    /// Check if the connection is alive
    fn is_connected(&self) -> bool;

    /// Subscribe to streams using the codec
    async fn subscribe(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError>;

    /// Unsubscribe from streams using the codec
    async fn unsubscribe(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError>;

    /// Get the next decoded message
    async fn next_message(&mut self) -> Option<Result<C::Message, ExchangeError>>;

    /// HFT optimization: bulk send messages for reduced syscalls
    async fn send_bulk(&mut self, messages: &[Message]) -> Result<(), ExchangeError>;

    /// HFT optimization: set socket options for low latency
    async fn configure_low_latency(&mut self) -> Result<(), ExchangeError>;
}

/// Tungstenite-based WebSocket implementation with HFT optimizations
pub struct TungsteniteWs<C: WsCodec> {
    url: String,
    write: Option<
        futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
    >,
    read: Option<
        futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
    >,
    connected: bool,
    exchange_name: String,
    codec: C,
    config: WsConfig,
    // HFT optimization: message buffer for batch processing
    message_buffer: Vec<Message>,
    // HFT optimization: connection statistics
    connection_stats: ConnectionStats,
}

/// Connection statistics for monitoring HFT performance
#[derive(Debug, Default)]
pub struct ConnectionStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub reconnection_count: u32,
    pub last_heartbeat: Option<std::time::Instant>,
}

impl<C: WsCodec> TungsteniteWs<C> {
    /// Create a new WebSocket session with the specified codec
    ///
    /// # Arguments
    /// * `url` - The WebSocket URL to connect to
    /// * `exchange_name` - Name of the exchange for logging/tracing
    /// * `codec` - The codec to handle message encoding/decoding
    pub fn new(url: String, exchange_name: String, codec: C) -> Self {
        Self {
            url,
            write: None,
            read: None,
            connected: false,
            exchange_name,
            codec,
            config: WsConfig::default(),
            message_buffer: Vec::new(),
            connection_stats: ConnectionStats::default(),
        }
    }

    /// Create a new WebSocket session with HFT-optimized configuration
    pub fn new_hft_optimized(url: String, exchange_name: String, codec: C) -> Self {
        Self {
            url,
            write: None,
            read: None,
            connected: false,
            exchange_name,
            codec,
            config: WsConfig::hft_optimized(),
            message_buffer: Vec::with_capacity(4096),
            connection_stats: ConnectionStats::default(),
        }
    }

    /// Set custom WebSocket configuration
    pub fn with_config(mut self, config: WsConfig) -> Self {
        self.config = config;
        self
    }

    /// Get connection statistics
    pub fn stats(&self) -> &ConnectionStats {
        &self.connection_stats
    }
}

#[async_trait]
impl<C: WsCodec> WsSession<C> for TungsteniteWs<C> {
    #[instrument(skip(self), fields(exchange = %self.exchange_name, url = %self.url))]
    async fn connect(&mut self) -> Result<(), ExchangeError> {
        let connect_timeout = Duration::from_millis(self.config.connect_timeout_ms);

        // HFT optimization: use timeout for connection
        let connection_future = tokio::time::timeout(connect_timeout, connect_async(&self.url));

        let (ws_stream, _) = connection_future
            .await
            .map_err(|_| {
                ExchangeError::ConnectionTimeout("WebSocket connection timeout".to_string())
            })?
            .map_err(|e| {
                ExchangeError::NetworkError(format!("WebSocket connection failed: {}", e))
            })?;

        let (write, read) = ws_stream.split();
        self.write = Some(write);
        self.read = Some(read);
        self.connected = true;

        // Update connection statistics
        self.connection_stats.reconnection_count += 1;
        self.connection_stats.last_heartbeat = Some(std::time::Instant::now());

        Ok(())
    }

    #[instrument(skip(self, msg), fields(exchange = %self.exchange_name))]
    async fn send_raw(&mut self, msg: Message) -> Result<(), ExchangeError> {
        if !self.connected {
            return Err(ExchangeError::NetworkError(
                "WebSocket not connected".to_string(),
            ));
        }

        let write = self.write.as_mut().ok_or_else(|| {
            ExchangeError::NetworkError("WebSocket write stream not available".to_string())
        })?;

        write.send(msg).await.map_err(|e| {
            self.connected = false;
            ExchangeError::NetworkError(format!("Failed to send WebSocket message: {}", e))
        })?;

        Ok(())
    }

    #[instrument(skip(self), fields(exchange = %self.exchange_name))]
    async fn next_raw(&mut self) -> Option<Result<Message, ExchangeError>> {
        if !self.connected {
            return Some(Err(ExchangeError::NetworkError(
                "WebSocket not connected".to_string(),
            )));
        }

        let read = self.read.as_mut()?;

        match read.next().await {
            Some(Ok(message)) => {
                // Handle control messages at transport level only
                match &message {
                    Message::Close(_) => {
                        self.connected = false;
                        Some(Ok(message))
                    }
                    Message::Ping(data) => {
                        // Auto-respond to pings at transport level
                        let pong = Message::Pong(data.clone());
                        if let Err(e) = self.send_raw(pong).await {
                            warn!("Failed to send pong response: {}", e);
                        }
                        // Continue to next message
                        self.next_raw().await
                    }
                    Message::Pong(_) => {
                        // Ignore pong messages, continue to next
                        self.next_raw().await
                    }
                    _ => Some(Ok(message)),
                }
            }
            Some(Err(e)) => {
                self.connected = false;
                Some(Err(ExchangeError::NetworkError(format!(
                    "WebSocket error: {}",
                    e
                ))))
            }
            None => {
                self.connected = false;
                None
            }
        }
    }

    #[instrument(skip(self), fields(exchange = %self.exchange_name))]
    async fn close(&mut self) -> Result<(), ExchangeError> {
        if let Some(write) = self.write.as_mut() {
            let _ = write.send(Message::Close(None)).await;
        }
        self.connected = false;
        self.write = None;
        self.read = None;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    #[instrument(skip(self, streams), fields(exchange = %self.exchange_name, stream_count = streams.len()))]
    async fn subscribe(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError> {
        if streams.is_empty() {
            return Ok(());
        }

        let message = self.codec.encode_subscription(streams)?;
        self.send_raw(message).await
    }

    #[instrument(skip(self, streams), fields(exchange = %self.exchange_name, stream_count = streams.len()))]
    async fn unsubscribe(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError> {
        if streams.is_empty() {
            return Ok(());
        }

        let message = self.codec.encode_unsubscription(streams)?;
        self.send_raw(message).await
    }

    #[instrument(skip(self), fields(exchange = %self.exchange_name))]
    async fn next_message(&mut self) -> Option<Result<C::Message, ExchangeError>> {
        loop {
            match self.next_raw().await {
                Some(Ok(raw_msg)) => {
                    // Skip control messages - they're handled at transport level
                    if matches!(
                        raw_msg,
                        Message::Ping(_) | Message::Pong(_) | Message::Close(_)
                    ) {
                        continue;
                    }

                    // Decode the message using the codec
                    match self.codec.decode_message(raw_msg) {
                        Ok(Some(decoded)) => return Some(Ok(decoded)),
                        Ok(None) => {} // Codec chose to ignore this message
                        Err(e) => return Some(Err(e)),
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
                None => return None,
            }
        }
    }

    /// HFT optimization: bulk send messages for reduced syscalls
    async fn send_bulk(&mut self, messages: &[Message]) -> Result<(), ExchangeError> {
        if !self.connected {
            return Err(ExchangeError::NetworkError(
                "WebSocket not connected for bulk send".to_string(),
            ));
        }

        let write = self.write.as_mut().ok_or_else(|| {
            ExchangeError::NetworkError(
                "WebSocket write stream not available for bulk send".to_string(),
            )
        })?;

        for msg in messages {
            write.send(msg.clone()).await.map_err(|e| {
                self.connected = false;
                ExchangeError::NetworkError(format!("Failed to send bulk WebSocket message: {}", e))
            })?;

            // Update statistics
            self.connection_stats.messages_sent += 1;
            if let Message::Text(text) = msg {
                self.connection_stats.bytes_sent += text.len() as u64;
            } else if let Message::Binary(data) = msg {
                self.connection_stats.bytes_sent += data.len() as u64;
            }
        }
        Ok(())
    }

    /// HFT optimization: configure for low latency (placeholder - actual socket options would be set at TCP level)
    async fn configure_low_latency(&mut self) -> Result<(), ExchangeError> {
        if !self.connected {
            return Err(ExchangeError::NetworkError(
                "WebSocket not connected for low latency configuration".to_string(),
            ));
        }

        // Note: Actual low-latency socket options would need to be set at the TCP socket level
        // This is a placeholder showing the intent for HFT optimization
        // In a real implementation, you would:
        // 1. Set TCP_NODELAY to disable Nagle's algorithm
        // 2. Set SO_RCVBUF and SO_SNDBUF for optimal buffer sizes
        // 3. Set CPU affinity for the socket processing thread
        // 4. Use SO_REUSEPORT for load balancing across multiple connections

        // For now, just ensure the message buffer is properly sized
        self.message_buffer.reserve(self.config.message_buffer_size);

        Ok(())
    }
}

/// Wrapper that adds automatic reconnection capabilities
pub struct ReconnectWs<C: WsCodec, T: WsSession<C>> {
    inner: T,
    max_reconnect_attempts: u32,
    reconnect_delay: Duration,
    auto_resubscribe: bool,
    subscribed_streams: Vec<String>,
    _codec: std::marker::PhantomData<C>,
}

impl<C: WsCodec, T: WsSession<C>> ReconnectWs<C, T> {
    /// Create a new reconnecting WebSocket wrapper
    ///
    /// # Arguments
    /// * `inner` - The underlying WebSocket session to wrap
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            max_reconnect_attempts: 5,
            reconnect_delay: Duration::from_secs(1),
            auto_resubscribe: true,
            subscribed_streams: Vec::new(),
            _codec: std::marker::PhantomData,
        }
    }

    /// Set the maximum number of reconnection attempts
    pub fn with_max_reconnect_attempts(mut self, max_attempts: u32) -> Self {
        self.max_reconnect_attempts = max_attempts;
        self
    }

    /// Set the initial delay between reconnection attempts
    pub fn with_reconnect_delay(mut self, delay: Duration) -> Self {
        self.reconnect_delay = delay;
        self
    }

    /// Enable or disable automatic resubscription after reconnection
    pub fn with_auto_resubscribe(mut self, auto_resubscribe: bool) -> Self {
        self.auto_resubscribe = auto_resubscribe;
        self
    }

    async fn attempt_reconnect(&mut self) -> Result<(), ExchangeError> {
        let mut attempts = 0;
        let mut delay = self.reconnect_delay;

        while attempts < self.max_reconnect_attempts {
            attempts += 1;

            match self.inner.connect().await {
                Ok(_) => {
                    if self.auto_resubscribe && !self.subscribed_streams.is_empty() {
                        let streams: Vec<&str> =
                            self.subscribed_streams.iter().map(|s| s.as_str()).collect();
                        if let Err(e) = self.inner.subscribe(&streams).await {
                            warn!("Failed to resubscribe after reconnection: {}", e);
                        }
                    }
                    return Ok(());
                }
                Err(e) => {
                    error!("Reconnection attempt {} failed: {}", attempts, e);
                    if attempts < self.max_reconnect_attempts {
                        sleep(delay).await;
                        delay = std::cmp::min(delay * 2, Duration::from_secs(60));
                    }
                }
            }
        }

        Err(ExchangeError::NetworkError(format!(
            "Failed to reconnect after {} attempts",
            self.max_reconnect_attempts
        )))
    }
}

#[async_trait]
impl<C: WsCodec, T: WsSession<C>> WsSession<C> for ReconnectWs<C, T> {
    async fn connect(&mut self) -> Result<(), ExchangeError> {
        self.inner.connect().await
    }

    async fn send_raw(&mut self, msg: Message) -> Result<(), ExchangeError> {
        if !self.inner.is_connected() {
            self.attempt_reconnect().await?;
        }
        self.inner.send_raw(msg).await
    }

    async fn next_raw(&mut self) -> Option<Result<Message, ExchangeError>> {
        loop {
            if !self.inner.is_connected() {
                if let Err(e) = self.attempt_reconnect().await {
                    return Some(Err(e));
                }
            }

            match self.inner.next_raw().await {
                Some(Ok(msg)) => return Some(Ok(msg)),
                Some(Err(_e)) => {
                    // Connection error, try to reconnect
                    if let Err(reconnect_err) = self.attempt_reconnect().await {
                        return Some(Err(reconnect_err));
                    }
                    // Continue the loop to try receiving again
                }
                None => {
                    // Connection closed, try to reconnect
                    if let Err(reconnect_err) = self.attempt_reconnect().await {
                        return Some(Err(reconnect_err));
                    }
                    // Continue the loop to try receiving again
                }
            }
        }
    }

    async fn close(&mut self) -> Result<(), ExchangeError> {
        self.inner.close().await
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    async fn subscribe(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError> {
        // Store streams as strings for resubscription
        self.subscribed_streams = streams.iter().map(|s| s.as_ref().to_string()).collect();
        self.inner.subscribe(streams).await
    }

    async fn unsubscribe(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError> {
        // Remove from subscribed streams
        let streams_to_remove: Vec<String> =
            streams.iter().map(|s| s.as_ref().to_string()).collect();
        self.subscribed_streams
            .retain(|s| !streams_to_remove.contains(s));
        self.inner.unsubscribe(streams).await
    }

    async fn next_message(&mut self) -> Option<Result<C::Message, ExchangeError>> {
        loop {
            if !self.inner.is_connected() {
                if let Err(e) = self.attempt_reconnect().await {
                    return Some(Err(e));
                }
            }

            match self.inner.next_message().await {
                Some(Ok(msg)) => return Some(Ok(msg)),
                Some(Err(_e)) => {
                    // Connection error, try to reconnect
                    if let Err(reconnect_err) = self.attempt_reconnect().await {
                        return Some(Err(reconnect_err));
                    }
                    // Continue the loop to try receiving again
                }
                None => {
                    // Connection closed, try to reconnect
                    if let Err(reconnect_err) = self.attempt_reconnect().await {
                        return Some(Err(reconnect_err));
                    }
                    // Continue the loop to try receiving again
                }
            }
        }
    }

    /// HFT optimization: bulk send messages for reduced syscalls
    async fn send_bulk(&mut self, messages: &[Message]) -> Result<(), ExchangeError> {
        if !self.inner.is_connected() {
            self.attempt_reconnect().await?;
        }
        self.inner.send_bulk(messages).await
    }

    /// HFT optimization: configure for low latency
    async fn configure_low_latency(&mut self) -> Result<(), ExchangeError> {
        if !self.inner.is_connected() {
            self.attempt_reconnect().await?;
        }
        self.inner.configure_low_latency().await
    }
}
