use super::client::BinanceConnector;
use super::converters::{convert_binance_market, parse_websocket_message};
use super::types as binance_types;
use crate::core::errors::{ExchangeError, ResultExt};
use crate::core::traits::MarketDataSource;
use crate::core::types::{
    Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig,
};
use crate::core::websocket::{build_binance_stream_url, WebSocketManager};
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
impl MarketDataSource for BinanceConnector {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/api/v3/exchangeInfo", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_exchange_context(|| format!("Failed to send exchange info request to {}", url))?;
        let exchange_info: binance_types::BinanceExchangeInfo = response
            .json()
            .await
            .with_exchange_context(|| "Failed to parse exchange info response".to_string())?;

        let markets = exchange_info
            .symbols
            .into_iter()
            .map(convert_binance_market)
            .collect();

        Ok(markets)
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Build streams for combined stream format
        let mut streams = Vec::new();

        for symbol in &symbols {
            let lower_symbol = symbol.to_lowercase();
            for sub_type in &subscription_types {
                match sub_type {
                    SubscriptionType::Ticker => {
                        streams.push(format!("{}@ticker", lower_symbol));
                    }
                    SubscriptionType::OrderBook { depth } => {
                        if let Some(d) = depth {
                            streams.push(format!("{}@depth{}@100ms", lower_symbol, d));
                        } else {
                            streams.push(format!("{}@depth@100ms", lower_symbol));
                        }
                    }
                    SubscriptionType::Trades => {
                        streams.push(format!("{}@trade", lower_symbol));
                    }
                    SubscriptionType::Klines { interval } => {
                        streams.push(format!(
                            "{}@kline_{}",
                            lower_symbol,
                            interval.to_binance_format()
                        ));
                    }
                }
            }
        }

        let ws_url = self.get_websocket_url();
        let full_url = build_binance_stream_url(&ws_url, &streams);

        let ws_manager = WebSocketManager::new(full_url);
        ws_manager
            .start_stream(parse_websocket_message)
            .await
            .with_exchange_context(|| {
                format!(
                    "Failed to start WebSocket stream for symbols: {:?}",
                    symbols
                )
            })
    }

    fn get_websocket_url(&self) -> String {
        if self.config.testnet {
            "wss://testnet.binance.vision/ws".to_string()
        } else {
            "wss://stream.binance.com:443/ws".to_string()
        }
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = interval.to_binance_format();
        let url = format!("{}/api/v3/klines", self.base_url);

        let mut query_params = vec![
            ("symbol", symbol.clone()),
            ("interval", interval_str.clone()),
        ];

        if let Some(limit_val) = limit {
            query_params.push(("limit", limit_val.to_string()));
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
            .with_exchange_context(|| {
                format!(
                    "Failed to send klines request: url={}, symbol={}",
                    url, symbol
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.with_exchange_context(|| {
                format!("Failed to read klines error response for symbol {}", symbol)
            })?;
            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("K-lines request failed: {}", error_text),
            });
        }

        let klines_data: Vec<Vec<serde_json::Value>> =
            response.json().await.with_exchange_context(|| {
                format!("Failed to parse klines response for symbol {}", symbol)
            })?;

        let klines = klines_data
            .into_iter()
            .map(|kline_array| {
                // Binance returns k-lines as arrays, we need to parse them safely
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

                Kline {
                    symbol: symbol.clone(),
                    open_time,
                    close_time,
                    interval: interval_str.clone(),
                    open_price,
                    high_price,
                    low_price,
                    close_price,
                    volume,
                    number_of_trades,
                    final_bar: true, // Historical k-lines are always final
                }
            })
            .collect();

        Ok(klines)
    }
}
