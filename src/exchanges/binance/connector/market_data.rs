use crate::core::{
    errors::ExchangeError,
    kernel::{RestClient, WsSession},
    traits::MarketDataSource,
    types::{Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig},
};
use crate::exchanges::binance::{
    codec::{BinanceCodec, BinanceMessage},
    conversions::{convert_binance_market, convert_binance_rest_kline},
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
        let full_url = build_binance_stream_url(&ws_url, &streams);

        // Use kernel WebSocket implementation
        let codec = crate::exchanges::binance::codec::BinanceCodec;
        let ws_session =
            crate::core::kernel::ws::TungsteniteWs::new(full_url, "binance".to_string(), codec);

        // Add reconnection wrapper for production reliability
        let mut reconnect_ws = crate::core::kernel::ws::ReconnectWs::new(ws_session)
            .with_auto_resubscribe(true)
            .with_max_reconnect_attempts(u32::MAX);

        // Connect and subscribe
        reconnect_ws.connect().await.map_err(|e| {
            ExchangeError::Other(format!(
                "Failed to connect to WebSocket for symbols: {:?}, error: {}",
                symbols, e
            ))
        })?;

        if !streams.is_empty() {
            let stream_refs: Vec<&str> = streams.iter().map(|s| s.as_str()).collect();
            reconnect_ws.subscribe(&stream_refs).await.map_err(|e| {
                ExchangeError::Other(format!(
                    "Failed to subscribe to streams: {:?}, error: {}",
                    streams, e
                ))
            })?;
        }

        // Create channel for messages
        let (tx, rx) = mpsc::channel(1000);

        // Spawn task to handle messages
        tokio::spawn(async move {
            while let Some(result) = reconnect_ws.next_message().await {
                match result {
                    Ok(binance_message) => {
                        // Convert BinanceMessage to MarketDataType
                        if let Some(market_data) =
                            convert_binance_message_to_market_data(binance_message)
                        {
                            if tx.send(market_data).await.is_err() {
                                break; // Receiver dropped
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {:?}", e);
                        // Continue processing to handle reconnection
                    }
                }
            }
        });

        Ok(rx)
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

/// Helper function to build Binance WebSocket URLs for combined streams
fn build_binance_stream_url(base_url: &str, streams: &[String]) -> String {
    if streams.is_empty() {
        return base_url.to_string();
    }

    // For combined streams, Binance expects /ws/stream?streams=...
    let base = base_url
        .strip_suffix("/ws")
        .map_or(base_url, |stripped| stripped);
    format!("{}/stream?streams={}", base, streams.join("/"))
}

/// Convert `BinanceMessage` to `MarketDataType`
fn convert_binance_message_to_market_data(message: BinanceMessage) -> Option<MarketDataType> {
    use crate::core::types::conversion;

    match message {
        BinanceMessage::Ticker(ticker) => {
            let symbol = conversion::string_to_symbol(&ticker.symbol);
            let price = conversion::string_to_price(&ticker.price);
            let price_change = conversion::string_to_price(&ticker.price_change);
            let price_change_percent = conversion::string_to_decimal(&ticker.price_change_percent);
            let high_price = conversion::string_to_price(&ticker.high_price);
            let low_price = conversion::string_to_price(&ticker.low_price);
            let volume = conversion::string_to_volume(&ticker.volume);
            let quote_volume = conversion::string_to_volume(&ticker.quote_volume);

            Some(MarketDataType::Ticker(crate::core::types::Ticker {
                symbol,
                price,
                price_change,
                price_change_percent,
                high_price,
                low_price,
                volume,
                quote_volume,
                open_time: ticker.open_time,
                close_time: ticker.close_time,
                count: ticker.count,
            }))
        }
        BinanceMessage::OrderBook(orderbook) => {
            let symbol = conversion::string_to_symbol(&orderbook.symbol);

            let bids = orderbook
                .bids
                .iter()
                .map(|bid| crate::core::types::OrderBookEntry {
                    price: conversion::string_to_price(&bid[0]),
                    quantity: conversion::string_to_quantity(&bid[1]),
                })
                .collect();
            let asks = orderbook
                .asks
                .iter()
                .map(|ask| crate::core::types::OrderBookEntry {
                    price: conversion::string_to_price(&ask[0]),
                    quantity: conversion::string_to_quantity(&ask[1]),
                })
                .collect();

            Some(MarketDataType::OrderBook(crate::core::types::OrderBook {
                symbol,
                bids,
                asks,
                last_update_id: orderbook.final_update_id,
            }))
        }
        BinanceMessage::Trade(trade) => {
            let symbol = conversion::string_to_symbol(&trade.symbol);
            let price = conversion::string_to_price(&trade.price);
            let quantity = conversion::string_to_quantity(&trade.quantity);

            Some(MarketDataType::Trade(crate::core::types::Trade {
                symbol,
                id: trade.id,
                price,
                quantity,
                time: trade.time,
                is_buyer_maker: trade.is_buyer_maker,
            }))
        }
        BinanceMessage::Kline(kline) => {
            let symbol = conversion::string_to_symbol(&kline.symbol);
            let open_price = conversion::string_to_price(&kline.kline.open_price);
            let high_price = conversion::string_to_price(&kline.kline.high_price);
            let low_price = conversion::string_to_price(&kline.kline.low_price);
            let close_price = conversion::string_to_price(&kline.kline.close_price);
            let volume = conversion::string_to_volume(&kline.kline.volume);

            Some(MarketDataType::Kline(crate::core::types::Kline {
                symbol,
                open_time: kline.kline.open_time,
                close_time: kline.kline.close_time,
                interval: kline.kline.interval,
                open_price,
                high_price,
                low_price,
                close_price,
                volume,
                number_of_trades: kline.kline.number_of_trades,
                final_bar: kline.kline.final_bar,
            }))
        }
        BinanceMessage::Unknown => None,
    }
}
