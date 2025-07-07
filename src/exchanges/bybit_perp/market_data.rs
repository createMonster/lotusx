use super::client::BybitPerpConnector;
use super::converters::{convert_bybit_perp_market, parse_websocket_message};
use super::types::{self as bybit_perp_types, BybitPerpResultExt};
use crate::core::errors::ExchangeError;
use crate::core::traits::{FundingRateSource, MarketDataSource};
use crate::core::types::{
    FundingRate, Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig,
    conversion,
};
use crate::core::websocket::BybitWebSocketManager;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{instrument, warn};

/// Helper to check API response status and convert to proper error
#[cold]
#[inline(never)]
fn handle_api_response_error(ret_code: i32, ret_msg: String) -> bybit_perp_types::BybitPerpError {
    bybit_perp_types::BybitPerpError::api_error(ret_code, ret_msg)
}

#[async_trait]
impl MarketDataSource for BybitPerpConnector {
    #[instrument(skip(self), fields(exchange = "bybit_perp"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!(
            "{}/v5/market/instruments-info?category=linear",
            self.base_url
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_contract_context("*")?;

        let api_response: bybit_perp_types::BybitPerpApiResponse<
            bybit_perp_types::BybitPerpExchangeInfo,
        > = response.json().await.with_contract_context("*")?;

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
        let ws_manager = BybitWebSocketManager::new(ws_url);
        ws_manager
            .start_stream_with_subscriptions(streams, parse_websocket_message)
            .await
    }

    fn get_websocket_url(&self) -> String {
        if self.config.testnet {
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
        let url = format!(
            "{}/v5/market/kline?category=linear&symbol={}&interval={}",
            self.base_url, symbol, interval_str
        );

        let mut query_params = vec![];

        if let Some(limit_val) = limit {
            query_params.push(("limit", limit_val.to_string()));
        }

        if let Some(start) = start_time {
            query_params.push(("start", start.to_string()));
        }

        if let Some(end) = end_time {
            query_params.push(("end", end.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .with_contract_context(&symbol)?;

        if !response.status().is_success() {
            let error_text = response.text().await.with_contract_context(&symbol)?;
            return Err(ExchangeError::Other(format!(
                "K-lines request failed for contract {}: {}",
                symbol, error_text
            )));
        }

        let klines_response: bybit_perp_types::BybitPerpKlineResponse =
            response.json().await.with_contract_context(&symbol)?;

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
                    interval: interval.to_bybit_format(),
                    open_price: conversion::string_to_price(kline_vec.get(1).unwrap_or(&"0".to_string())),
                    high_price: conversion::string_to_price(kline_vec.get(2).unwrap_or(&"0".to_string())),
                    low_price: conversion::string_to_price(kline_vec.get(3).unwrap_or(&"0".to_string())),
                    close_price: conversion::string_to_price(kline_vec.get(4).unwrap_or(&"0".to_string())),
                    volume: conversion::string_to_volume(kline_vec.get(5).unwrap_or(&"0".to_string())),
                    number_of_trades: 0,
                    final_bar: true,
                }
            })
            .collect();

        Ok(klines)
    }
}

// Funding Rate Implementation for Bybit Perpetual
#[async_trait]
impl FundingRateSource for BybitPerpConnector {
    #[instrument(skip(self), fields(symbols = ?symbols))]
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        match symbols {
            Some(symbol_list) if symbol_list.len() == 1 => {
                // Get funding rate for single symbol using tickers endpoint
                self.get_single_funding_rate(&symbol_list[0])
                    .await
                    .map(|rate| vec![rate])
            }
            Some(_) | None => {
                // Get all funding rates using tickers endpoint
                self.get_all_funding_rates().await
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        self.get_all_funding_rates_internal().await
    }

    #[instrument(skip(self), fields(symbol = %symbol))]
    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let url = format!("{}/v5/market/funding/history", self.base_url);

        let mut query_params = vec![
            ("category", "linear".to_string()),
            ("symbol", symbol.clone()),
        ];

        if let Some(limit_val) = limit {
            query_params.push(("limit", limit_val.to_string()));
        } else {
            query_params.push(("limit", "100".to_string()));
        }

        if let Some(start) = start_time {
            query_params.push(("startTime", start.to_string()));
        }

        if let Some(end) = end_time {
            query_params.push(("endTime", end.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .with_contract_context(&symbol)?;

        let api_response: bybit_perp_types::BybitPerpFundingRateResponse =
            response.json().await.with_contract_context(&symbol)?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::Other(
                bybit_perp_types::BybitPerpError::funding_rate_error(
                    format!("{} - {}", api_response.ret_code, api_response.ret_msg),
                    Some(symbol),
                )
                .to_string(),
            ));
        }

        let mut result = Vec::with_capacity(api_response.result.list.len());
        for rate_info in api_response.result.list {
            result.push(FundingRate {
                symbol: conversion::string_to_symbol(&rate_info.symbol),
                funding_rate: Some(crate::core::types::conversion::string_to_decimal(
                    &rate_info.funding_rate,
                )),
                previous_funding_rate: None,
                next_funding_rate: None,
                funding_time: Some(rate_info.funding_rate_timestamp),
                next_funding_time: None,
                mark_price: None,
                index_price: None,
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
        }

        Ok(result)
    }
}

impl BybitPerpConnector {
    async fn get_single_funding_rate(&self, symbol: &str) -> Result<FundingRate, ExchangeError> {
        let url = format!("{}/v5/market/tickers", self.base_url);

        let query_params = vec![("category", "linear"), ("symbol", symbol)];

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .with_contract_context(symbol)?;

        let api_response: bybit_perp_types::BybitPerpTickerResponse =
            response.json().await.with_contract_context(symbol)?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::Other(
                bybit_perp_types::BybitPerpError::funding_rate_error(
                    format!("{} - {}", api_response.ret_code, api_response.ret_msg),
                    Some(symbol.to_string()),
                )
                .to_string(),
            ));
        }

        api_response.result.list.first().map_or_else(
            || {
                Err(ExchangeError::Other(
                    bybit_perp_types::BybitPerpError::funding_rate_error(
                        "No ticker data found".to_string(),
                        Some(symbol.to_string()),
                    )
                    .to_string(),
                ))
            },
            |ticker_info| {
                let next_funding_time = ticker_info
                    .next_funding_time
                    .parse::<i64>()
                    .unwrap_or_else(|_| {
                        warn!(symbol = %symbol, "Failed to parse next_funding_time");
                        0
                    });

                Ok(FundingRate {
                    symbol: conversion::string_to_symbol(&ticker_info.symbol),
                    funding_rate: Some(crate::core::types::conversion::string_to_decimal(
                        &ticker_info.funding_rate,
                    )),
                    previous_funding_rate: None,
                    next_funding_rate: None,
                    funding_time: None,
                    next_funding_time: Some(next_funding_time),
                    mark_price: Some(crate::core::types::conversion::string_to_price(
                        &ticker_info.mark_price,
                    )),
                    index_price: Some(crate::core::types::conversion::string_to_price(
                        &ticker_info.index_price,
                    )),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                })
            },
        )
    }

    async fn get_all_funding_rates_internal(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        let url = format!("{}/v5/market/tickers", self.base_url);

        let query_params = vec![("category", "linear")];

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .with_contract_context("*")?;

        let api_response: bybit_perp_types::BybitPerpTickerResponse =
            response.json().await.with_contract_context("*")?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::Other(
                bybit_perp_types::BybitPerpError::funding_rate_error(
                    format!("{} - {}", api_response.ret_code, api_response.ret_msg),
                    None,
                )
                .to_string(),
            ));
        }

        let mut result = Vec::with_capacity(api_response.result.list.len());
        for ticker_info in api_response.result.list {
            let next_funding_time =
                ticker_info
                    .next_funding_time
                    .parse::<i64>()
                    .unwrap_or_else(|_| {
                        warn!(symbol = %ticker_info.symbol, "Failed to parse next_funding_time");
                        0
                    });

            result.push(FundingRate {
                symbol: conversion::string_to_symbol(&ticker_info.symbol),
                funding_rate: Some(crate::core::types::conversion::string_to_decimal(
                    &ticker_info.funding_rate,
                )),
                previous_funding_rate: None,
                next_funding_rate: None,
                funding_time: None,
                next_funding_time: Some(next_funding_time),
                mark_price: Some(crate::core::types::conversion::string_to_price(
                    &ticker_info.mark_price,
                )),
                index_price: Some(crate::core::types::conversion::string_to_price(
                    &ticker_info.index_price,
                )),
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
        }

        Ok(result)
    }
}
