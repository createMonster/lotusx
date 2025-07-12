use crate::core::{
    errors::ExchangeError,
    kernel::{RestClient, WsSession},
    traits::MarketDataSource,
    types::{Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig},
};
use crate::exchanges::binance::{
    codec::BinanceCodec,
    conversions::{convert_binance_market, convert_binance_rest_kline, parse_websocket_message},
    rest::BinanceRestClient,
};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Market data implementation for Binance
pub struct MarketData<R: RestClient, W = ()> {
    rest: BinanceRestClient<R>,
    #[allow(dead_code)] // May be used for future WebSocket functionality
    ws: Option<W>,
    testnet: bool,
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    fn ws_url(&self) -> String {
        if self.testnet {
            "wss://testnet.binance.vision/ws".to_string()
        } else {
            "wss://stream.binance.com:443/ws".to_string()
        }
    }
}

impl<R: RestClient + Clone, W: WsSession<BinanceCodec>> MarketData<R, W> {
    /// Create a new market data source with WebSocket support
    pub fn new(rest: &R, ws: Option<W>, testnet: bool) -> Self {
        Self {
            rest: BinanceRestClient::new(rest.clone()),
            ws,
            testnet,
        }
    }
}

impl<R: RestClient + Clone> MarketData<R, ()> {
    /// Create a new market data source without WebSocket support
    pub fn new(rest: &R, _ws: Option<()>, testnet: bool) -> Self {
        Self {
            rest: BinanceRestClient::new(rest.clone()),
            ws: None,
            testnet,
        }
    }
}

#[async_trait]
impl<R: RestClient + Clone, W: WsSession<BinanceCodec>> MarketDataSource for MarketData<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let exchange_info = self.rest.get_exchange_info().await?;
        let markets = exchange_info
            .symbols
            .into_iter()
            .map(convert_binance_market)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ExchangeError::Other(format!("Failed to convert market: {}", e)))?;
        Ok(markets)
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Use the codec helper to create stream identifiers
        let streams = crate::exchanges::binance::codec::create_binance_stream_identifiers(
            &symbols,
            &subscription_types,
        );

        // Create WebSocket URL
        let ws_url = self.ws_url();
        let full_url = crate::core::websocket::build_binance_stream_url(&ws_url, &streams);

        // Use WebSocket manager to start the stream
        let ws_manager = crate::core::websocket::WebSocketManager::new(full_url);
        ws_manager
            .start_stream(parse_websocket_message)
            .await
            .map_err(|e| {
                ExchangeError::Other(format!(
                    "Failed to start WebSocket stream for symbols: {:?}, error: {}",
                    symbols, e
                ))
            })
    }

    fn get_websocket_url(&self) -> String {
        self.ws_url()
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let klines = self
            .rest
            .get_klines(&symbol, interval, limit, start_time, end_time)
            .await?;

        let converted_klines = klines
            .into_iter()
            .map(|k| convert_binance_rest_kline(&k, &symbol, &interval.to_string()))
            .collect();

        Ok(converted_klines)
    }
}

#[async_trait]
impl<R: RestClient + Clone> MarketDataSource for MarketData<R, ()> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let exchange_info = self.rest.get_exchange_info().await?;
        let markets = exchange_info
            .symbols
            .into_iter()
            .map(convert_binance_market)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ExchangeError::Other(format!("Failed to convert market: {}", e)))?;
        Ok(markets)
    }

    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        Err(ExchangeError::WebSocketError(
            "WebSocket not available in REST-only mode".to_string(),
        ))
    }

    fn get_websocket_url(&self) -> String {
        self.ws_url()
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let klines = self
            .rest
            .get_klines(&symbol, interval, limit, start_time, end_time)
            .await?;

        let converted_klines = klines
            .into_iter()
            .map(|k| convert_binance_rest_kline(&k, &symbol, &interval.to_string()))
            .collect();

        Ok(converted_klines)
    }
}
