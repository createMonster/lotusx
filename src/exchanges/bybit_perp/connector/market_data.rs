#![allow(clippy::or_fun_call)]
#![allow(clippy::future_not_send)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::use_self)]

use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::{FundingRateSource, MarketDataSource};
use crate::core::types::{
    conversion, FundingRate, Kline, KlineInterval, Market, MarketDataType, SubscriptionType,
    WebSocketConfig,
};
use crate::exchanges::bybit_perp::conversions::{
    convert_bybit_perp_market, parse_websocket_message,
};
use crate::exchanges::bybit_perp::rest::BybitPerpRestClient;
use crate::exchanges::bybit_perp::types::{self as bybit_perp_types, BybitPerpResultExt};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{instrument, warn};

/// Market data implementation for Bybit Perpetual
pub struct MarketData<R: RestClient, W = ()> {
    rest: BybitPerpRestClient<R>,
    #[allow(dead_code)]
    ws: Option<W>,
    testnet: bool,
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    pub fn new(rest: &R, ws: Option<W>) -> Self {
        Self {
            rest: BybitPerpRestClient::new(rest.clone()),
            ws,
            testnet: false, // Default to mainnet
        }
    }

    pub fn with_testnet(rest: &R, ws: Option<W>, testnet: bool) -> Self {
        Self {
            rest: BybitPerpRestClient::new(rest.clone()),
            ws,
            testnet,
        }
    }
}

// Safety: MarketData is Sync if its fields are Sync
unsafe impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> Sync for MarketData<R, W> {}

/// Helper to check API response status and convert to proper error
#[cold]
#[inline(never)]
fn handle_api_response_error(ret_code: i32, ret_msg: String) -> bybit_perp_types::BybitPerpError {
    bybit_perp_types::BybitPerpError::api_error(ret_code, ret_msg)
}

#[async_trait]
impl<R: RestClient + Clone, W: Send + Sync> MarketDataSource for MarketData<R, W> {
    #[instrument(skip(self), fields(exchange = "bybit_perp"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let api_response = self.rest.get_markets().await.with_contract_context("*")?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::Other(
                handle_api_response_error(api_response.ret_code, api_response.ret_msg).to_string(),
            ));
        }

        let markets = api_response
            .result
            .list
            .into_iter()
            .map(convert_bybit_perp_market)
            .collect();

        Ok(markets)
    }

    #[instrument(skip(self, _config), fields(exchange = "bybit_perp", symbols_count = symbols.len()))]
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Build streams for Bybit V5 WebSocket format
        let mut streams = Vec::new();

        for symbol in &symbols {
            for sub_type in &subscription_types {
                match sub_type {
                    SubscriptionType::Ticker => {
                        streams.push(format!("tickers.{}", symbol));
                    }
                    SubscriptionType::OrderBook { depth } => {
                        if let Some(d) = depth {
                            streams.push(format!("orderbook.{}.{}", d, symbol));
                        } else {
                            streams.push(format!("orderbook.1.{}", symbol));
                        }
                    }
                    SubscriptionType::Trades => {
                        streams.push(format!("publicTrade.{}", symbol));
                    }
                    SubscriptionType::Klines { interval } => {
                        streams.push(format!("kline.{}.{}", interval.to_bybit_format(), symbol));
                    }
                }
            }
        }

        let ws_url = self.get_websocket_url();
        let ws_manager = crate::core::websocket::BybitWebSocketManager::new(ws_url);
        ws_manager
            .start_stream_with_subscriptions(streams, parse_websocket_message)
            .await
    }

    fn get_websocket_url(&self) -> String {
        if self.testnet {
            "wss://stream-testnet.bybit.com/v5/public/linear".to_string()
        } else {
            "wss://stream.bybit.com/v5/public/linear".to_string()
        }
    }

    #[instrument(skip(self), fields(exchange = "bybit_perp", contract = %symbol, interval = %interval))]
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = interval.to_bybit_format();
        let klines_response = self
            .rest
            .get_klines(&symbol, &interval_str, limit, start_time, end_time)
            .await
            .with_contract_context(&symbol)?;

        if klines_response.ret_code != 0 {
            return Err(ExchangeError::Other(format!(
                "Bybit Perp API error for {}: {} - {}",
                symbol, klines_response.ret_code, klines_response.ret_msg
            )));
        }

        let klines = klines_response
            .result
            .list
            .into_iter()
            .map(|kline_vec| {
                // Bybit V5 API returns klines in format:
                // [startTime, openPrice, highPrice, lowPrice, closePrice, volume, turnover]
                let start_time: i64 = kline_vec
                    .first()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or_else(|| {
                        warn!(contract = %symbol, "Failed to parse kline start_time");
                        0
                    });

                // Calculate close time based on interval
                let interval_ms = match interval {
                    KlineInterval::Seconds1 => 1000,
                    KlineInterval::Minutes1 => 60_000,
                    KlineInterval::Minutes3 => 180_000,
                    KlineInterval::Minutes5 => 300_000,
                    KlineInterval::Minutes15 => 900_000,
                    KlineInterval::Minutes30 => 1_800_000,
                    KlineInterval::Hours1 => 3_600_000,
                    KlineInterval::Hours2 => 7_200_000,
                    KlineInterval::Hours4 => 14_400_000,
                    KlineInterval::Hours6 => 21_600_000,
                    KlineInterval::Hours8 => 28_800_000,
                    KlineInterval::Hours12 => 43_200_000,
                    KlineInterval::Days1 => 86_400_000,
                    KlineInterval::Days3 => 259_200_000,
                    KlineInterval::Weeks1 => 604_800_000,
                    KlineInterval::Months1 => 2_592_000_000, // Approximate
                };

                let close_time = start_time + interval_ms;

                Kline {
                    symbol: conversion::string_to_symbol(&symbol),
                    open_time: start_time,
                    close_time,
                    interval: interval_str.clone(),
                    open_price: conversion::string_to_price(
                        kline_vec.get(1).unwrap_or(&"0".to_string()),
                    ),
                    high_price: conversion::string_to_price(
                        kline_vec.get(2).unwrap_or(&"0".to_string()),
                    ),
                    low_price: conversion::string_to_price(
                        kline_vec.get(3).unwrap_or(&"0".to_string()),
                    ),
                    close_price: conversion::string_to_price(
                        kline_vec.get(4).unwrap_or(&"0".to_string()),
                    ),
                    volume: conversion::string_to_volume(
                        kline_vec.get(5).unwrap_or(&"0".to_string()),
                    ),
                    number_of_trades: 0, // Bybit doesn't provide this in REST API
                    final_bar: true,
                }
            })
            .collect();

        Ok(klines)
    }
}

#[async_trait]
impl<R: RestClient + Clone, W: Send + Sync> FundingRateSource for MarketData<R, W> {
    #[instrument(skip(self), fields(exchange = "bybit_perp"))]
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        match symbols {
            Some(symbol_list) => {
                let mut funding_rates = Vec::new();
                for symbol in symbol_list {
                    match self.get_single_funding_rate(&symbol).await {
                        Ok(rate) => funding_rates.push(rate),
                        Err(e) => {
                            warn!(contract = %symbol, error = %e, "Failed to get funding rate");
                        }
                    }
                }
                Ok(funding_rates)
            }
            None => self.get_all_funding_rates().await,
        }
    }

    #[instrument(skip(self), fields(exchange = "bybit_perp"))]
    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        self.get_all_funding_rates_internal().await
    }

    #[instrument(skip(self), fields(exchange = "bybit_perp", contract = %symbol))]
    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let _ = (start_time, end_time, limit); // Suppress unused warnings for now
                                               // For now, return single funding rate - extend later for history
        let rate = self.get_single_funding_rate(&symbol).await?;
        Ok(vec![rate])
    }
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    async fn get_single_funding_rate(&self, symbol: &str) -> Result<FundingRate, ExchangeError> {
        // Get ticker data which includes current funding rate and mark/index prices
        let ticker_response = self.rest.get_tickers(Some(symbol)).await?;

        if ticker_response.ret_code != 0 {
            return Err(ExchangeError::Other(format!(
                "Bybit Perp ticker API error for {}: {} - {}",
                symbol, ticker_response.ret_code, ticker_response.ret_msg
            )));
        }

        let ticker_info = ticker_response.result.list.first().ok_or_else(|| {
            ExchangeError::Other(format!("No ticker data found for symbol: {}", symbol))
        })?;

        // Parse next funding time from string to timestamp
        let next_funding_time = ticker_info.next_funding_time.parse::<i64>().ok();

        Ok(FundingRate {
            symbol: conversion::string_to_symbol(&ticker_info.symbol),
            funding_rate: Some(conversion::string_to_decimal(&ticker_info.funding_rate)),
            previous_funding_rate: None,
            next_funding_rate: None,
            funding_time: None, // Current funding rate doesn't have historical timestamp
            next_funding_time,
            mark_price: Some(conversion::string_to_price(&ticker_info.mark_price)),
            index_price: Some(conversion::string_to_price(&ticker_info.index_price)),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    async fn get_all_funding_rates_internal(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        // Get all tickers which include funding rates and mark/index prices
        let ticker_response = self.rest.get_tickers(None).await?;

        if ticker_response.ret_code != 0 {
            return Err(ExchangeError::Other(format!(
                "Bybit Perp tickers API error: {} - {}",
                ticker_response.ret_code, ticker_response.ret_msg
            )));
        }

        let funding_rates = ticker_response
            .result
            .list
            .into_iter()
            .map(|ticker_info| {
                // Parse next funding time from string to timestamp
                let next_funding_time = ticker_info.next_funding_time.parse::<i64>().ok();

                FundingRate {
                    symbol: conversion::string_to_symbol(&ticker_info.symbol),
                    funding_rate: Some(conversion::string_to_decimal(&ticker_info.funding_rate)),
                    previous_funding_rate: None,
                    next_funding_rate: None,
                    funding_time: None, // Current funding rate doesn't have historical timestamp
                    next_funding_time,
                    mark_price: Some(conversion::string_to_price(&ticker_info.mark_price)),
                    index_price: Some(conversion::string_to_price(&ticker_info.index_price)),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                }
            })
            .collect();

        Ok(funding_rates)
    }
}

// Extension trait for KlineInterval to convert to Bybit format
#[allow(dead_code)]
trait BybitFormat {
    fn to_bybit_format(&self) -> String;
}

impl BybitFormat for KlineInterval {
    fn to_bybit_format(&self) -> String {
        match self {
            KlineInterval::Seconds1 => "1s",
            KlineInterval::Minutes1 => "1",
            KlineInterval::Minutes3 => "3",
            KlineInterval::Minutes5 => "5",
            KlineInterval::Minutes15 => "15",
            KlineInterval::Minutes30 => "30",
            KlineInterval::Hours1 => "60",
            KlineInterval::Hours2 => "120",
            KlineInterval::Hours4 => "240",
            KlineInterval::Hours6 => "360",
            KlineInterval::Hours8 => "480",
            KlineInterval::Hours12 => "720",
            KlineInterval::Days1 => "D",
            KlineInterval::Days3 => "3D",
            KlineInterval::Weeks1 => "W",
            KlineInterval::Months1 => "M",
        }
        .to_string()
    }
}
