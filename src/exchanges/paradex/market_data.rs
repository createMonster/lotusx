use crate::core::errors::ExchangeError;
use crate::core::traits::MarketDataSource;
use crate::core::types::{
    Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig,
};
use crate::exchanges::paradex::types::ParadexMarket;
use crate::exchanges::paradex::ParadexConnector;
use async_trait::async_trait;
use futures_util::StreamExt;
use reqwest;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[async_trait]
impl MarketDataSource for ParadexConnector {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.paradex.trade/v1/markets")
            .send()
            .await?;
        let markets: Vec<ParadexMarket> = response.json().await?;
        Ok(markets.into_iter().map(Into::into).collect())
    }

    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        let url = self.get_websocket_url();
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| ExchangeError::WebSocketError(e.to_string()))?;
        let (mut _write, mut read) = ws_stream.split();

        let (_tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(_text)) => {
                        // Process the message and send it to the channel
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(rx)
    }

    fn get_websocket_url(&self) -> String {
        "wss://ws.paradex.trade/v1".to_string()
    }

    async fn get_klines(
        &self,
        _symbol: String,
        _interval: KlineInterval,
        _limit: Option<u32>,
        _start_time: Option<i64>,
        _end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        // Implementation of get_klines will be added here
        Ok(vec![])
    }
}
