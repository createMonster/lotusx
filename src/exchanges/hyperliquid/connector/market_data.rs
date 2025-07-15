use crate::core::{
    errors::ExchangeError,
    kernel::{rest::RestClient, ws::WsSession, ReconnectWs, TungsteniteWs},
    traits::MarketDataSource,
    types::{Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig},
};
use crate::exchanges::hyperliquid::{codec::HyperliquidCodec, conversions, rest::HyperliquidRest};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tracing::{error, info, instrument, warn};

/// Message for WebSocket subscription management
#[derive(Debug)]
#[allow(dead_code)] // Unsubscribe may be used in future implementations
enum SubscriptionCommand {
    Subscribe {
        streams: Vec<String>,
        response: oneshot::Sender<Result<(), ExchangeError>>,
    },
    Unsubscribe {
        streams: Vec<String>,
        response: oneshot::Sender<Result<(), ExchangeError>>,
    },
}

/// WebSocket subscription manager that handles the actual WebSocket connection
struct WebSocketManager {
    ws_session: ReconnectWs<HyperliquidCodec, TungsteniteWs<HyperliquidCodec>>,
    subscribers: HashMap<String, Vec<mpsc::Sender<MarketDataType>>>,
    command_rx: mpsc::Receiver<SubscriptionCommand>,
    active_subscriptions: Vec<String>,
}

impl WebSocketManager {
    async fn run(mut self) {
        // Connect to WebSocket
        if let Err(e) = self.ws_session.connect().await {
            error!("Failed to connect to WebSocket: {}", e);
            return;
        }

        info!("WebSocket manager started and connected");

        loop {
            tokio::select! {
                // Handle subscription commands
                cmd = self.command_rx.recv() => {
                    match cmd {
                        Some(SubscriptionCommand::Subscribe { streams, response }) => {
                            let result = self.handle_subscribe(streams).await;
                            let _ = response.send(result);
                        }
                        Some(SubscriptionCommand::Unsubscribe { streams, response }) => {
                            let result = self.handle_unsubscribe(streams).await;
                            let _ = response.send(result);
                        }
                        None => {
                            warn!("Subscription command channel closed");
                            break;
                        }
                    }
                }

                // Handle incoming WebSocket messages
                msg = self.ws_session.next_message() => {
                    match msg {
                        Some(Ok(message)) => {
                            self.handle_message(message).await;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            // The ReconnectWs will handle reconnection automatically
                        }
                        None => {
                            warn!("WebSocket stream ended");
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn handle_subscribe(&mut self, streams: Vec<String>) -> Result<(), ExchangeError> {
        // Filter out streams that are already subscribed
        let new_streams: Vec<String> = streams
            .into_iter()
            .filter(|stream| !self.active_subscriptions.contains(stream))
            .collect();

        if new_streams.is_empty() {
            return Ok(());
        }

        // Subscribe to new streams
        if let Err(e) = self.ws_session.subscribe(&new_streams).await {
            error!("Failed to subscribe to streams: {}", e);
            return Err(e);
        }

        // Add to active subscriptions
        self.active_subscriptions.extend(new_streams.clone());
        info!("Subscribed to {} streams", new_streams.len());

        Ok(())
    }

    async fn handle_unsubscribe(&mut self, streams: Vec<String>) -> Result<(), ExchangeError> {
        // Filter streams that are actually subscribed
        let subscribed_streams: Vec<String> = streams
            .into_iter()
            .filter(|stream| self.active_subscriptions.contains(stream))
            .collect();

        if subscribed_streams.is_empty() {
            return Ok(());
        }

        // Unsubscribe from streams
        if let Err(e) = self.ws_session.unsubscribe(&subscribed_streams).await {
            error!("Failed to unsubscribe from streams: {}", e);
            return Err(e);
        }

        // Remove from active subscriptions
        self.active_subscriptions
            .retain(|stream| !subscribed_streams.contains(stream));
        info!("Unsubscribed from {} streams", subscribed_streams.len());

        Ok(())
    }

    async fn handle_message(
        &self,
        message: crate::exchanges::hyperliquid::codec::HyperliquidWsMessage,
    ) {
        // Convert message to MarketDataType
        let market_data = MarketDataType::from(message);

        // Get the symbol from the message to determine routing
        let symbol = match &market_data {
            MarketDataType::Ticker(ticker) => ticker.symbol.as_str(),
            MarketDataType::OrderBook(book) => book.symbol.as_str(),
            MarketDataType::Trade(trade) => trade.symbol.as_str(),
            MarketDataType::Kline(kline) => kline.symbol.as_str(),
        };

        // Send to symbol-specific subscribers
        if let Some(senders) = self.subscribers.get(symbol.as_str()) {
            for sender in senders {
                if let Err(e) = sender.send(market_data.clone()).await {
                    warn!("Failed to send message to subscriber for {}: {}", symbol, e);
                }
            }
        }

        // Send to wildcard subscribers (symbol "*")
        if let Some(senders) = self.subscribers.get("*") {
            for sender in senders {
                if let Err(e) = sender.send(market_data.clone()).await {
                    warn!("Failed to send message to wildcard subscriber: {}", e);
                }
            }
        }
    }

    #[allow(dead_code)] // May be used in future implementations
    fn add_subscriber(&mut self, symbol: String, sender: mpsc::Sender<MarketDataType>) {
        self.subscribers.entry(symbol).or_default().push(sender);
    }
}

/// Shared WebSocket subscription manager
type SharedSubscriptionManager = Arc<RwLock<HashMap<String, Vec<mpsc::Sender<MarketDataType>>>>>;

/// Internal state for WebSocket management
struct WebSocketState {
    command_tx: Option<mpsc::Sender<SubscriptionCommand>>,
    handler_started: bool,
}

pub struct MarketData<R: RestClient, W = ()> {
    rest: HyperliquidRest<R>,
    #[allow(dead_code)] // May be used in future implementations
    ws: Option<W>,
    subscription_manager: Option<SharedSubscriptionManager>,
    ws_state: Arc<Mutex<WebSocketState>>,
}

impl<R: RestClient + Clone> MarketData<R, ()> {
    pub fn new(rest: HyperliquidRest<R>) -> Self {
        Self {
            rest,
            ws: None,
            subscription_manager: None,
            ws_state: Arc::new(Mutex::new(WebSocketState {
                command_tx: None,
                handler_started: false,
            })),
        }
    }
}

impl<R: RestClient + Clone, W: WsSession<HyperliquidCodec> + Send + Sync> MarketData<R, W> {
    pub fn new_with_ws(rest: HyperliquidRest<R>, ws: W) -> Self {
        Self {
            rest,
            ws: Some(ws),
            subscription_manager: Some(Arc::new(RwLock::new(HashMap::new()))),
            ws_state: Arc::new(Mutex::new(WebSocketState {
                command_tx: None,
                handler_started: false,
            })),
        }
    }

    /// Start WebSocket manager in a background task (only once)
    #[allow(clippy::significant_drop_tightening)]
    async fn ensure_websocket_handler_started(&self) -> Result<(), ExchangeError> {
        let mut state = self.ws_state.lock().await;
        if state.handler_started {
            return Ok(());
        }

        // Create WebSocket manager
        let ws_url = self.rest.get_websocket_url();
        let codec = HyperliquidCodec::new();
        let base_ws = TungsteniteWs::new(ws_url, "hyperliquid".to_string(), codec);
        let reconnect_ws = ReconnectWs::new(base_ws)
            .with_max_reconnect_attempts(5)
            .with_reconnect_delay(std::time::Duration::from_secs(2))
            .with_auto_resubscribe(true);

        // Create command channel
        let (command_tx, command_rx) = mpsc::channel(100);
        state.command_tx = Some(command_tx);

        // Create WebSocket manager
        let manager = WebSocketManager {
            ws_session: reconnect_ws,
            subscribers: HashMap::new(),
            command_rx,
            active_subscriptions: Vec::new(),
        };

        // Start the manager in a background task
        tokio::spawn(async move {
            manager.run().await;
        });

        state.handler_started = true;
        Ok(())
    }

    /// Subscribe to market data streams
    async fn subscribe_to_streams(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Ensure WebSocket handler is started
        self.ensure_websocket_handler_started().await?;

        // Create a channel for this subscription
        let (tx, rx) = mpsc::channel(1000);

        // Build subscription streams in Hyperliquid format
        let mut streams = Vec::new();
        for symbol in &symbols {
            for sub_type in &subscription_types {
                let stream = match sub_type {
                    SubscriptionType::Ticker => format!("{}@ticker", symbol),
                    SubscriptionType::OrderBook { .. } => format!("{}@orderbook", symbol),
                    SubscriptionType::Trades => format!("{}@trade", symbol),
                    SubscriptionType::Klines { .. } => format!("{}@kline", symbol),
                };
                streams.push(stream);
            }
        }

        // Send subscription command
        {
            let state = self.ws_state.lock().await;
            if let Some(command_tx) = &state.command_tx {
                let (response_tx, response_rx) = oneshot::channel();
                let subscribe_cmd = SubscriptionCommand::Subscribe {
                    streams: streams.clone(),
                    response: response_tx,
                };

                command_tx.send(subscribe_cmd).await.map_err(|_| {
                    ExchangeError::Other("Failed to send subscription command".to_string())
                })?;

                response_rx.await.map_err(|_| {
                    ExchangeError::Other("Failed to receive subscription response".to_string())
                })??;
            }
        }

        // Register subscribers for each symbol
        if let Some(subscription_manager) = &self.subscription_manager {
            let mut subscribers = subscription_manager.write().await;
            for symbol in &symbols {
                subscribers
                    .entry(symbol.clone())
                    .or_default()
                    .push(tx.clone());
            }
        }

        info!(
            "WebSocket subscriptions registered for {} symbols",
            symbols.len()
        );
        Ok(rx)
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync> MarketDataSource for MarketData<R, ()> {
    /// Get all available markets/trading pairs
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let assets = self.rest.get_markets().await?;
        Ok(assets
            .into_iter()
            .map(conversions::convert_asset_to_market)
            .collect())
    }

    /// Subscribe to market data via WebSocket
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // For now, return an error as we don't have WebSocket support in this implementation
        Err(ExchangeError::Other(
            "WebSocket subscriptions require WebSocket session".to_string(),
        ))
    }

    /// Get WebSocket endpoint URL for market data
    fn get_websocket_url(&self) -> String {
        self.rest.get_websocket_url()
    }

    /// Get historical k-lines/candlestick data
    #[instrument(skip(self), fields(exchange = "hyperliquid", symbol = %symbol, interval = ?interval))]
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = conversions::convert_kline_interval_to_hyperliquid(interval);
        let candles = self
            .rest
            .get_candlestick_snapshot(&symbol, &interval_str, start_time, end_time)
            .await?;

        // Apply limit if specified
        let mut klines: Vec<Kline> = candles
            .into_iter()
            .map(|c| conversions::convert_candle_to_kline(&c, &symbol, interval))
            .collect();

        if let Some(limit) = limit {
            klines.truncate(limit as usize);
        }

        Ok(klines)
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: WsSession<HyperliquidCodec> + Send + Sync>
    MarketDataSource for MarketData<R, W>
{
    /// Get all available markets/trading pairs
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let assets = self.rest.get_markets().await?;
        Ok(assets
            .into_iter()
            .map(conversions::convert_asset_to_market)
            .collect())
    }

    /// Subscribe to market data via WebSocket
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Use the safe implementation with interior mutability
        self.subscribe_to_streams(symbols, subscription_types).await
    }

    /// Get WebSocket endpoint URL for market data
    fn get_websocket_url(&self) -> String {
        self.rest.get_websocket_url()
    }

    /// Get historical k-lines/candlestick data
    #[instrument(skip(self), fields(exchange = "hyperliquid", symbol = %symbol, interval = ?interval))]
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = conversions::convert_kline_interval_to_hyperliquid(interval);
        let candles = self
            .rest
            .get_candlestick_snapshot(&symbol, &interval_str, start_time, end_time)
            .await?;

        // Apply limit if specified
        let mut klines: Vec<Kline> = candles
            .into_iter()
            .map(|c| conversions::convert_candle_to_kline(&c, &symbol, interval))
            .collect();

        if let Some(limit) = limit {
            klines.truncate(limit as usize);
        }

        Ok(klines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::kernel::rest::ReqwestRest;
    use crate::exchanges::hyperliquid::rest::HyperliquidRest;

    #[test]
    fn test_market_data_creation() {
        let rest_client = ReqwestRest::new(
            "https://api.hyperliquid.xyz".to_string(),
            "hyperliquid".to_string(),
            None,
        )
        .unwrap();
        let hyperliquid_rest = HyperliquidRest::new(rest_client, None, false);
        let market_data = MarketData::new(hyperliquid_rest);

        // Test basic functionality
        assert!(market_data.get_websocket_url().contains("hyperliquid"));
    }

    #[test]
    fn test_market_data_with_websocket() {
        let rest_client = ReqwestRest::new(
            "https://api.hyperliquid.xyz".to_string(),
            "hyperliquid".to_string(),
            None,
        )
        .unwrap();
        let hyperliquid_rest = HyperliquidRest::new(rest_client, None, false);

        // Create a mock WebSocket session
        let ws_url = "wss://api.hyperliquid.xyz/ws".to_string();
        let codec = HyperliquidCodec::new();
        let ws_session = TungsteniteWs::new(ws_url, "hyperliquid".to_string(), codec);

        let market_data = MarketData::new_with_ws(hyperliquid_rest, ws_session);

        // Test that WebSocket functionality is available
        assert!(market_data.subscription_manager.is_some());
        assert!(market_data.ws.is_some());
    }
}
