use super::client::BinancePerpConnector;
use super::converters::{convert_binance_perp_market, parse_websocket_message};
use super::types::{
    self as binance_perp_types, BinancePerpError, BinancePerpFundingRate, BinancePerpPremiumIndex,
};
use crate::core::errors::ExchangeError;
use crate::core::traits::{FundingRateSource, MarketDataSource};
use crate::core::types::{
    FundingRate, Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig,
};
use crate::core::websocket::{build_binance_stream_url, WebSocketManager};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{error, instrument};

#[async_trait]
impl MarketDataSource for BinancePerpConnector {
    #[instrument(skip(self))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/fapi/v1/exchangeInfo", self.base_url);

        let exchange_info: binance_perp_types::BinancePerpExchangeInfo = self
            .request_with_retry(|| self.client.get(&url), &url)
            .await
            .map_err(ExchangeError::from)?;

        // Use iterator chain to avoid intermediate allocations
        let markets: Vec<Market> = exchange_info
            .symbols
            .into_iter()
            .map(convert_binance_perp_market)
            .collect();

        Ok(markets)
    }

    #[instrument(
        skip(self, _config),
        fields(
            symbols = ?symbols,
            subscription_types = ?subscription_types
        )
    )]
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Pre-allocate with estimated capacity to avoid reallocations
        let estimated_streams = symbols.len() * subscription_types.len();
        let mut streams = Vec::with_capacity(estimated_streams);

        for symbol in &symbols {
            // Convert to lowercase once per symbol to avoid repeated allocations
            let lower_symbol = symbol.to_lowercase();

            for sub_type in &subscription_types {
                let stream = match sub_type {
                    SubscriptionType::Ticker => {
                        format!("{}@ticker", lower_symbol)
                    }
                    SubscriptionType::OrderBook { depth } => depth.as_ref().map_or_else(
                        || format!("{}@depth@100ms", lower_symbol),
                        |d| format!("{}@depth{}@100ms", lower_symbol, d),
                    ),
                    SubscriptionType::Trades => {
                        format!("{}@aggTrade", lower_symbol)
                    }
                    SubscriptionType::Klines { interval } => {
                        format!("{}@kline_{}", lower_symbol, interval.to_binance_format())
                    }
                };
                streams.push(stream);
            }
        }

        let ws_url = self.get_websocket_url();
        let full_url = build_binance_stream_url(&ws_url, &streams);

        let ws_manager = WebSocketManager::new(full_url);
        ws_manager
            .start_stream(parse_websocket_message)
            .await
            .map_err(|e| {
                error!(
                    symbols = ?symbols,
                    error = %e,
                    "Failed to start WebSocket stream"
                );
                ExchangeError::NetworkError(format!("WebSocket connection failed: {}", e))
            })
    }

    fn get_websocket_url(&self) -> String {
        if self.config.testnet {
            "wss://stream.binancefuture.com/ws".to_string()
        } else {
            "wss://fstream.binance.com:443/ws".to_string()
        }
    }

    #[instrument(
        skip(self),
        fields(
            symbol = %symbol,
            interval = %interval,
            limit = ?limit
        )
    )]
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = interval.to_binance_format();
        let url = format!("{}/fapi/v1/klines", self.base_url);

        // Pre-allocate query params with known capacity
        let mut query_params = Vec::with_capacity(5);
        query_params.extend_from_slice(&[
            ("symbol", symbol.as_str()),
            ("interval", interval_str.as_str()),
        ]);

        let limit_str;
        if let Some(limit_val) = limit {
            limit_str = limit_val.to_string();
            query_params.push(("limit", limit_str.as_str()));
        }

        let start_str;
        if let Some(start) = start_time {
            start_str = start.to_string();
            query_params.push(("startTime", start_str.as_str()));
        }

        let end_str;
        if let Some(end) = end_time {
            end_str = end.to_string();
            query_params.push(("endTime", end_str.as_str()));
        }

        let response = self
            .client
            .get(&url)
            .query(&query_params)
            .send()
            .await
            .map_err(|e| {
                error!(
                    symbol = %symbol,
                    interval = %interval,
                    url = %url,
                    error = %e,
                    "Failed to fetch klines"
                );
                BinancePerpError::market_data_error(
                    format!("Klines request failed: {}", e),
                    Some(symbol.clone()),
                )
            })?;

        self.handle_klines_response(response, symbol, interval_str)
            .await
    }
}

impl BinancePerpConnector {
    #[cold]
    #[inline(never)]
    async fn handle_klines_response(
        &self,
        response: reqwest::Response,
        symbol: String,
        interval: String,
    ) -> Result<Vec<Kline>, ExchangeError> {
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| {
                BinancePerpError::network_error(format!("Failed to read error response: {}", e))
            })?;

            error!(
                symbol = %symbol,
                interval = %interval,
                status = %status,
                error_text = %error_text,
                "Klines request failed"
            );

            return Err(BinancePerpError::market_data_error(
                format!("K-lines request failed: {}", error_text),
                Some(symbol),
            )
            .into());
        }

        let klines_data: Vec<Vec<serde_json::Value>> = response.json().await.map_err(|e| {
            BinancePerpError::parse_error(
                format!("Failed to parse klines response: {}", e),
                Some(symbol.clone()),
            )
        })?;

        // Use iterator with known capacity for better performance
        let mut klines = Vec::with_capacity(klines_data.len());

        for kline_array in klines_data {
            // Parse values directly without intermediate allocations where possible
            let open_time = kline_array.first().and_then(|v| v.as_i64()).unwrap_or(0);
            let open_price = kline_array
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let high_price = kline_array
                .get(2)
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let low_price = kline_array
                .get(3)
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let close_price = kline_array
                .get(4)
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let volume = kline_array
                .get(5)
                .and_then(|v| v.as_str())
                .unwrap_or("0")
                .to_string();
            let close_time = kline_array.get(6).and_then(|v| v.as_i64()).unwrap_or(0);
            let number_of_trades = kline_array.get(8).and_then(|v| v.as_i64()).unwrap_or(0);

            klines.push(Kline {
                symbol: symbol.clone(),
                open_time,
                close_time,
                interval: interval.clone(),
                open_price,
                high_price,
                low_price,
                close_price,
                volume,
                number_of_trades,
                final_bar: true, // Historical k-lines are always final
            });
        }

        Ok(klines)
    }
}

// Funding Rate Implementation for Binance Perpetual
#[async_trait]
impl FundingRateSource for BinancePerpConnector {
    #[instrument(skip(self), fields(symbols = ?symbols))]
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        match symbols {
            Some(symbol_list) if symbol_list.len() == 1 => self
                .get_single_funding_rate(&symbol_list[0])
                .await
                .map(|rate| vec![rate]),
            Some(_) => {
                // For multiple symbols, get premium index for all and extract funding rates
                self.get_all_funding_rates_internal().await
            }
            None => {
                // Get all funding rates
                self.get_all_funding_rates_internal().await
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
        let url = format!("{}/fapi/v1/fundingRate", self.base_url);

        let mut url_with_params = self.client.get(&url).query(&[("symbol", symbol.as_str())]);

        if let Some(limit_val) = limit {
            url_with_params = url_with_params.query(&[("limit", &limit_val.to_string())]);
        } else {
            url_with_params = url_with_params.query(&[("limit", "100")]);
        }

        if let Some(start) = start_time {
            url_with_params = url_with_params.query(&[("startTime", &start.to_string())]);
        }

        if let Some(end) = end_time {
            url_with_params = url_with_params.query(&[("endTime", &end.to_string())]);
        }

        let response: reqwest::Response =
            url_with_params.send().await.map_err(|e| -> ExchangeError {
                error!(
                    symbol = %symbol,
                    url = %url,
                    error = %e,
                    "Failed to fetch funding rate history"
                );
                BinancePerpError::market_data_error(
                    format!("Funding rate history request failed: {}", e),
                    Some(symbol.clone()),
                )
                .into()
            })?;

        let funding_rates: Vec<BinancePerpFundingRate> = response.json().await.map_err(|e| {
            BinancePerpError::parse_error(
                format!("Failed to parse funding rate history: {}", e),
                Some(symbol.clone()),
            )
        })?;

        let mut result = Vec::with_capacity(funding_rates.len());
        for rate in funding_rates {
            result.push(FundingRate {
                symbol: rate.symbol,
                funding_rate: Some(rate.funding_rate),
                previous_funding_rate: None,
                next_funding_rate: None,
                funding_time: Some(rate.funding_time),
                next_funding_time: None,
                mark_price: None,
                index_price: None,
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
        }

        Ok(result)
    }
}

impl BinancePerpConnector {
    async fn get_single_funding_rate(&self, symbol: &str) -> Result<FundingRate, ExchangeError> {
        let url = format!("{}/fapi/v1/premiumIndex", self.base_url);

        let premium_index: BinancePerpPremiumIndex = self
            .request_with_retry(|| self.client.get(&url).query(&[("symbol", symbol)]), &url)
            .await
            .map_err(|e| -> ExchangeError {
                BinancePerpError::market_data_error(
                    format!("Single funding rate request failed: {}", e),
                    Some(symbol.to_string()),
                )
                .into()
            })?;

        Ok(FundingRate {
            symbol: premium_index.symbol,
            funding_rate: Some(premium_index.last_funding_rate),
            previous_funding_rate: None,
            next_funding_rate: None,
            funding_time: None,
            next_funding_time: Some(premium_index.next_funding_time),
            mark_price: Some(premium_index.mark_price),
            index_price: Some(premium_index.index_price),
            timestamp: premium_index.time,
        })
    }

    async fn get_all_funding_rates_internal(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        let url = format!("{}/fapi/v1/premiumIndex", self.base_url);

        let premium_indices: Vec<BinancePerpPremiumIndex> = self
            .request_with_retry(|| self.client.get(&url), &url)
            .await
            .map_err(|e| -> ExchangeError {
                BinancePerpError::market_data_error(
                    format!("All funding rates request failed: {}", e),
                    None,
                )
                .into()
            })?;

        let mut result = Vec::with_capacity(premium_indices.len());
        for premium_index in premium_indices {
            result.push(FundingRate {
                symbol: premium_index.symbol,
                funding_rate: Some(premium_index.last_funding_rate),
                previous_funding_rate: None,
                next_funding_rate: None,
                funding_time: None,
                next_funding_time: Some(premium_index.next_funding_time),
                mark_price: Some(premium_index.mark_price),
                index_price: Some(premium_index.index_price),
                timestamp: premium_index.time,
            });
        }

        Ok(result)
    }
}
