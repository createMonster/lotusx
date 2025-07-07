use super::client::ParadexConnector;
use super::types::{ParadexError, ParadexFundingRate, ParadexFundingRateHistory, ParadexMarket};
use crate::core::errors::ExchangeError;
use crate::core::traits::{FundingRateSource, MarketDataSource};
use crate::core::types::{
    conversion, FundingRate, Kline, KlineInterval, Market, MarketDataType, SubscriptionType,
    WebSocketConfig,
};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde_json;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, instrument, warn};

#[async_trait]
impl MarketDataSource for ParadexConnector {
    #[instrument(skip(self), fields(exchange = "paradex"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/v1/markets", self.base_url);

        // First let's see what the raw response looks like
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Other(format!("Markets request failed: {}", e)))?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16() as i32;
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExchangeError::ApiError {
                code: status_code,
                message: format!("Markets request failed: {}", error_text),
            });
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Other(format!("Failed to read response text: {}", e)))?;

        tracing::info!("Raw markets response: {}", response_text);

        // Try to parse as different formats
        if let Ok(markets_array) = serde_json::from_str::<Vec<ParadexMarket>>(&response_text) {
            Ok(markets_array.into_iter().map(Into::into).collect())
        } else if let Ok(response_obj) = serde_json::from_str::<serde_json::Value>(&response_text) {
            // Check if it's wrapped in a response object
            if let Some(markets_data) = response_obj.get("markets") {
                if let Ok(markets) =
                    serde_json::from_value::<Vec<ParadexMarket>>(markets_data.clone())
                {
                    return Ok(markets.into_iter().map(Into::into).collect());
                }
            }

            // Check if it's a data field
            if let Some(data) = response_obj.get("data") {
                if let Ok(markets) = serde_json::from_value::<Vec<ParadexMarket>>(data.clone()) {
                    return Ok(markets.into_iter().map(Into::into).collect());
                }
            }

            // Return an error with more details about the structure
            Err(ExchangeError::Other(format!(
                "Unexpected response format. Response structure: {:?}",
                response_obj
            )))
        } else {
            Err(ExchangeError::Other(format!(
                "Failed to parse markets response: {}",
                response_text
            )))
        }
    }

    #[instrument(
        skip(self, _config),
        fields(
            exchange = "paradex",
            symbols_count = symbols.len(),
            subscription_types = ?subscription_types
        )
    )]
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        let url = self.get_websocket_url();
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| ExchangeError::WebSocketError(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();
        let (tx, rx) = mpsc::channel(1000);

        // Send subscription messages
        let subscription_messages = self.build_subscription_messages(&symbols, &subscription_types);
        for message in subscription_messages {
            if let Err(e) = write.send(Message::Text(message)).await {
                error!("Failed to send WebSocket subscription: {}", e);
                return Err(ExchangeError::WebSocketError(format!(
                    "Subscription failed: {}",
                    e
                )));
            }
        }

        // Spawn task to handle incoming messages
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(text)) => {
                        if let Ok(parsed_data) = Self::parse_websocket_message(&text) {
                            if tx_clone.send(parsed_data).await.is_err() {
                                warn!("Receiver dropped, stopping WebSocket task");
                                break;
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        warn!("WebSocket connection closed by server");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(rx)
    }

    fn get_websocket_url(&self) -> String {
        self.get_websocket_url()
    }

    #[instrument(skip(self), fields(exchange = "paradex", symbol = %symbol))]
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let url = format!("{}/v1/klines", self.base_url);

        let mut params = vec![
            ("symbol", symbol.clone()),
            ("interval", interval.to_paradex_format()),
        ];

        if let Some(limit_val) = limit {
            params.push(("limit", limit_val.to_string()));
        }

        if let Some(start) = start_time {
            params.push(("startTime", start.to_string()));
        }

        if let Some(end) = end_time {
            params.push(("endTime", end.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                error!(
                    symbol = %symbol,
                    interval = ?interval,
                    error = %e,
                    "Failed to fetch klines"
                );
                ExchangeError::Other(format!("Klines request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16() as i32;
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExchangeError::ApiError {
                code: status_code,
                message: format!("Klines request failed: {}", error_text),
            });
        }

        // Parse klines response - this would need to be adapted based on Paradex's actual API
        let klines_data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| ExchangeError::Other(format!("Failed to parse klines response: {}", e)))?;

        let mut klines = Vec::with_capacity(klines_data.len());
        for kline_data in klines_data {
            // This parsing would need to be adapted based on Paradex's actual kline format
            if let Some(kline) = Self::parse_kline_data(&kline_data, &symbol, interval) {
                klines.push(kline);
            }
        }

        Ok(klines)
    }
}

// Funding Rate Implementation for Paradex Perpetual
#[async_trait]
impl FundingRateSource for ParadexConnector {
    #[instrument(skip(self), fields(exchange = "paradex", symbols = ?symbols))]
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        match symbols {
            Some(symbol_list) if symbol_list.len() == 1 => self
                .get_single_funding_rate(&symbol_list[0])
                .await
                .map(|rate| vec![rate]),
            Some(_) | None => self.get_all_funding_rates().await,
        }
    }

    #[instrument(skip(self), fields(exchange = "paradex"))]
    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        let url = format!("{}/v1/funding-rates", self.base_url);

        let funding_rates: Vec<ParadexFundingRate> = self
            .request_with_retry(|| self.client.get(&url), &url)
            .await
            .map_err(|e| -> ExchangeError {
                error!(error = %e, "Failed to fetch all funding rates");
                ExchangeError::Other(format!("All funding rates request failed: {}", e))
            })?;

        let mut result = Vec::with_capacity(funding_rates.len());
        for rate in funding_rates {
            result.push(FundingRate {
                symbol: conversion::string_to_symbol(&rate.symbol),
                funding_rate: Some(conversion::string_to_decimal(&rate.funding_rate)),
                previous_funding_rate: None,
                next_funding_rate: None,
                funding_time: None,
                next_funding_time: Some(rate.next_funding_time),
                mark_price: Some(conversion::string_to_price(&rate.mark_price)),
                index_price: Some(conversion::string_to_price(&rate.index_price)),
                timestamp: rate.timestamp,
            });
        }

        Ok(result)
    }

    #[instrument(skip(self), fields(exchange = "paradex", symbol = %symbol))]
    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let url = format!("{}/v1/funding-rate-history", self.base_url);

        let mut params = vec![("symbol", symbol.clone())];

        if let Some(limit_val) = limit {
            params.push(("limit", limit_val.to_string()));
        }

        if let Some(start) = start_time {
            params.push(("startTime", start.to_string()));
        }

        if let Some(end) = end_time {
            params.push(("endTime", end.to_string()));
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                error!(
                    symbol = %symbol,
                    error = %e,
                    "Failed to fetch funding rate history"
                );
                ExchangeError::Other(format!("Funding rate history request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16() as i32;
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExchangeError::ApiError {
                code: status_code,
                message: format!("Funding rate history request failed: {}", error_text),
            });
        }

        let funding_rates: Vec<ParadexFundingRateHistory> = response.json().await.map_err(|e| {
            ExchangeError::Other(format!("Failed to parse funding rate history: {}", e))
        })?;

        let mut result = Vec::with_capacity(funding_rates.len());
        for rate in funding_rates {
            result.push(FundingRate {
                symbol: conversion::string_to_symbol(&rate.symbol),
                funding_rate: Some(conversion::string_to_decimal(&rate.funding_rate)),
                previous_funding_rate: None,
                next_funding_rate: None,
                funding_time: None,
                next_funding_time: None,
                mark_price: None,
                index_price: None,
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
        }

        Ok(result)
    }
}

impl ParadexConnector {
    async fn get_single_funding_rate(&self, symbol: &str) -> Result<FundingRate, ExchangeError> {
        let url = format!("{}/v1/funding-rate", self.base_url);

        let params = vec![("symbol", symbol.to_string())];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                error!(
                    symbol = %symbol,
                    error = %e,
                    "Failed to fetch single funding rate"
                );
                ExchangeError::Other(format!("Single funding rate request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16() as i32;
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(ExchangeError::ApiError {
                code: status_code,
                message: format!("Single funding rate request failed: {}", error_text),
            });
        }

        let funding_rate: ParadexFundingRate = response.json().await.map_err(|e| {
            ExchangeError::Other(format!("Failed to parse funding rate response: {}", e))
        })?;

        Ok(FundingRate {
            symbol: conversion::string_to_symbol(&funding_rate.symbol),
            funding_rate: Some(conversion::string_to_decimal(&funding_rate.funding_rate)),
            previous_funding_rate: None,
            next_funding_rate: None,
            funding_time: None,
            next_funding_time: Some(funding_rate.next_funding_time),
            mark_price: Some(conversion::string_to_price(&funding_rate.mark_price)),
            index_price: Some(conversion::string_to_price(&funding_rate.index_price)),
            timestamp: funding_rate.timestamp,
        })
    }

    fn build_subscription_messages(
        &self,
        symbols: &[String],
        subscription_types: &[SubscriptionType],
    ) -> Vec<String> {
        let mut messages = Vec::new();

        for symbol in symbols {
            for sub_type in subscription_types {
                let channel = match sub_type {
                    SubscriptionType::Ticker => format!("ticker@{}", symbol),
                    SubscriptionType::OrderBook { depth } => depth.as_ref().map_or_else(
                        || format!("depth@{}", symbol),
                        |d| format!("depth{}@{}", d, symbol),
                    ),
                    SubscriptionType::Trades => format!("trade@{}", symbol),
                    SubscriptionType::Klines { interval } => {
                        format!("kline_{}@{}", interval.to_paradex_format(), symbol)
                    }
                };

                let subscription = serde_json::json!({
                    "method": "SUBSCRIBE",
                    "params": [channel],
                    "id": messages.len() + 1
                });

                messages.push(subscription.to_string());
            }
        }

        messages
    }

    fn parse_websocket_message(_text: &str) -> Result<MarketDataType, ExchangeError> {
        // This would need to be implemented based on Paradex's actual WebSocket message format
        // For now, return a placeholder error
        Err(ExchangeError::Other(
            "WebSocket message parsing not yet implemented".to_string(),
        ))
    }

    fn parse_kline_data(
        _data: &serde_json::Value,
        _symbol: &str,
        _interval: KlineInterval,
    ) -> Option<Kline> {
        // This would need to be implemented based on Paradex's actual kline data format
        // For now, return None
        None
    }
}

// Extend KlineInterval for Paradex format
trait ParadexKlineInterval {
    fn to_paradex_format(&self) -> String;
}

impl ParadexKlineInterval for KlineInterval {
    fn to_paradex_format(&self) -> String {
        match self {
            Self::Seconds1 | Self::Minutes1 => "1m".to_string(),
            Self::Minutes3 => "3m".to_string(),
            Self::Minutes5 => "5m".to_string(),
            Self::Minutes15 => "15m".to_string(),
            Self::Minutes30 => "30m".to_string(),
            Self::Hours1 => "1h".to_string(),
            Self::Hours2 => "2h".to_string(),
            Self::Hours4 => "4h".to_string(),
            Self::Hours6 => "6h".to_string(),
            Self::Hours8 => "8h".to_string(),
            Self::Hours12 => "12h".to_string(),
            Self::Days1 => "1d".to_string(),
            Self::Days3 => "3d".to_string(),
            Self::Weeks1 => "1w".to_string(),
            Self::Months1 => "1M".to_string(),
        }
    }
}

impl From<ParadexError> for ExchangeError {
    fn from(error: ParadexError) -> Self {
        match error {
            ParadexError::ApiError { code, message } => Self::ApiError { code, message },
            ParadexError::AuthError { reason } => Self::AuthError(reason),
            ParadexError::NetworkError(e) => Self::NetworkError(e.to_string()),
            ParadexError::JsonError(e) => Self::Other(e.to_string()),
            ParadexError::WebSocketError { reason } => Self::WebSocketError(reason),
            _ => Self::Other(error.to_string()),
        }
    }
}
