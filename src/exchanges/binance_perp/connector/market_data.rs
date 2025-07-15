use crate::core::{
    errors::ExchangeError,
    kernel::{RestClient, WsSession},
    traits::{FundingRateSource, MarketDataSource},
    types::{
        FundingRate, Kline, KlineInterval, Market, MarketDataType, SubscriptionType,
        WebSocketConfig,
    },
};
use crate::exchanges::binance_perp::{
    codec::BinancePerpCodec,
    conversions::{convert_binance_perp_market, convert_binance_perp_rest_kline},
    rest::BinancePerpRestClient,
};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::instrument;

/// Market data implementation for Binance Perpetual
pub struct MarketData<R: RestClient, W = ()> {
    rest: BinancePerpRestClient<R>,
    #[allow(dead_code)] // May be used for future WebSocket functionality
    ws: Option<W>,
    testnet: bool,
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    fn ws_url(&self) -> String {
        if self.testnet {
            "wss://stream.binancefuture.com/ws".to_string()
        } else {
            "wss://fstream.binance.com/ws".to_string()
        }
    }

    /// Convert Binance Perpetual funding rate to core type
    fn convert_funding_rate(
        &self,
        binance_rate: &crate::exchanges::binance_perp::types::BinancePerpFundingRate,
    ) -> FundingRate {
        FundingRate {
            symbol: crate::core::types::conversion::string_to_symbol(&binance_rate.symbol),
            funding_rate: Some(crate::core::types::conversion::string_to_decimal(
                &binance_rate.funding_rate,
            )),
            previous_funding_rate: None,
            next_funding_rate: None,
            funding_time: Some(binance_rate.funding_time),
            next_funding_time: None,
            mark_price: None,
            index_price: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        }
    }

    /// Convert Binance Perpetual funding rate with premium index data to core type
    fn convert_funding_rate_with_premium(
        &self,
        binance_rate: &crate::exchanges::binance_perp::types::BinancePerpFundingRate,
        premium_index: &crate::exchanges::binance_perp::types::BinancePerpPremiumIndex,
    ) -> FundingRate {
        FundingRate {
            symbol: crate::core::types::conversion::string_to_symbol(&binance_rate.symbol),
            funding_rate: Some(crate::core::types::conversion::string_to_decimal(
                &binance_rate.funding_rate,
            )),
            previous_funding_rate: None,
            next_funding_rate: None,
            funding_time: Some(binance_rate.funding_time),
            next_funding_time: Some(premium_index.next_funding_time),
            mark_price: Some(crate::core::types::conversion::string_to_price(
                &premium_index.mark_price,
            )),
            index_price: Some(crate::core::types::conversion::string_to_price(
                &premium_index.index_price,
            )),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        }
    }
}

impl<R: RestClient + Clone, W: WsSession<BinancePerpCodec>> MarketData<R, W> {
    /// Create a new market data source with WebSocket support
    pub fn new(rest: &R, ws: Option<W>, testnet: bool) -> Self {
        Self {
            rest: BinancePerpRestClient::new(rest.clone()),
            ws,
            testnet,
        }
    }
}

impl<R: RestClient + Clone> MarketData<R, ()> {
    /// Create a new market data source without WebSocket support
    pub fn new(rest: &R, _ws: Option<()>, testnet: bool) -> Self {
        Self {
            rest: BinancePerpRestClient::new(rest.clone()),
            ws: None,
            testnet,
        }
    }
}

#[async_trait]
impl<R: RestClient + Clone, W: WsSession<BinancePerpCodec>> MarketDataSource for MarketData<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let exchange_info = self.rest.get_exchange_info().await?;
        let markets = exchange_info
            .symbols
            .into_iter()
            .map(convert_binance_perp_market)
            .collect();
        Ok(markets)
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Use the codec helper to create stream identifiers
        let streams = crate::exchanges::binance_perp::codec::create_binance_perp_stream_identifiers(
            &symbols,
            &subscription_types,
        );

        // Create WebSocket URL
        let ws_url = self.ws_url();
        let full_url = build_binance_stream_url(&ws_url, &streams);

        // Use kernel WebSocket implementation with BinancePerpCodec
        let codec = crate::exchanges::binance_perp::codec::BinancePerpCodec;
        let ws_session = crate::core::kernel::ws::TungsteniteWs::new(
            full_url,
            "binance_perp".to_string(),
            codec,
        );

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
                    Ok(binance_perp_message) => {
                        // Convert BinancePerpMessage to MarketDataType
                        if let Some(market_data) =
                            convert_binance_perp_message_to_market_data(binance_perp_message)
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
            .map(|k| {
                let mut kline = convert_binance_perp_rest_kline(&k);
                kline.symbol = crate::core::types::conversion::string_to_symbol(&symbol);
                kline.interval = interval.to_string();
                kline
            })
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
            .map(convert_binance_perp_market)
            .collect();
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
            .map(|k| {
                let mut kline = convert_binance_perp_rest_kline(&k);
                kline.symbol = crate::core::types::conversion::string_to_symbol(&symbol);
                kline.interval = interval.to_string();
                kline
            })
            .collect();

        Ok(converted_klines)
    }
}

#[async_trait]
impl<R: RestClient + Clone, W: Send + Sync> FundingRateSource for MarketData<R, W> {
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        if let Some(symbols) = symbols {
            let mut all_rates = Vec::new();
            for symbol in symbols {
                // Get both funding rate and premium index for complete data
                let (funding_rate, premium_index) = tokio::try_join!(
                    self.rest.get_funding_rate(&symbol),
                    self.rest.get_premium_index(&symbol)
                )?;
                all_rates
                    .push(self.convert_funding_rate_with_premium(&funding_rate, &premium_index));
            }
            Ok(all_rates)
        } else {
            let rates = self.rest.get_all_funding_rates().await?;
            // For all funding rates, we can't efficiently get premium index for each
            // So we'll just use the basic conversion for now
            Ok(rates
                .iter()
                .map(|rate| self.convert_funding_rate(rate))
                .collect())
        }
    }

    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        let rates = self.rest.get_all_funding_rates().await?;
        // For performance reasons with getting all funding rates, we'll use basic conversion
        // Individual funding rate requests will use the premium index for complete data
        Ok(rates
            .iter()
            .map(|rate| self.convert_funding_rate(rate))
            .collect())
    }

    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol))]
    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let rates = self
            .rest
            .get_funding_rate_history(&symbol, start_time, end_time, limit)
            .await?;

        Ok(rates
            .iter()
            .map(|rate| self.convert_funding_rate(rate))
            .collect())
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

/// Convert `BinancePerpMessage` to `MarketDataType`
fn convert_binance_perp_message_to_market_data(
    message: crate::exchanges::binance_perp::codec::BinancePerpMessage,
) -> Option<MarketDataType> {
    use crate::core::types::conversion;

    match message {
        crate::exchanges::binance_perp::codec::BinancePerpMessage::Ticker(ticker) => {
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
        crate::exchanges::binance_perp::codec::BinancePerpMessage::OrderBook(orderbook) => {
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
        crate::exchanges::binance_perp::codec::BinancePerpMessage::Trade(trade) => {
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
        crate::exchanges::binance_perp::codec::BinancePerpMessage::Kline(kline) => {
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
        _ => None, // Ignore unknown and funding rate messages for market data
    }
}
