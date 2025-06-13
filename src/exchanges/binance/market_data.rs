use super::client::BinanceConnector;
use super::converters::{convert_binance_market, parse_websocket_message};
use super::types as binance_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::MarketDataSource;
use crate::core::types::{Kline, Market, MarketDataType, SubscriptionType, WebSocketConfig};
use crate::core::websocket::{build_binance_stream_url, WebSocketManager};
use async_trait::async_trait;
use tokio::sync::mpsc;

#[async_trait]
impl MarketDataSource for BinanceConnector {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/api/v3/exchangeInfo", self.base_url);

        let response = self.client.get(&url).send().await?;
        let exchange_info: binance_types::BinanceExchangeInfo = response.json().await?;

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
                        streams.push(format!("{}@kline_{}", lower_symbol, interval));
                    }
                }
            }
        }

        let ws_url = self.get_websocket_url();
        let full_url = build_binance_stream_url(&ws_url, &streams);

        let ws_manager = WebSocketManager::new(full_url);
        ws_manager.start_stream(parse_websocket_message).await
    }

    fn get_websocket_url(&self) -> String {
        if self.config.testnet {
            "wss://testnet.binance.vision/ws".to_string()
        } else {
            "wss://stream.binance.com:9443/ws".to_string()
        }
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: String,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let url = format!("{}/api/v3/klines", self.base_url);

        let mut query_params = vec![("symbol", symbol.clone()), ("interval", interval.clone())];

        if let Some(limit_val) = limit {
            query_params.push(("limit", limit_val.to_string()));
        }

        if let Some(start) = start_time {
            query_params.push(("startTime", start.to_string()));
        }

        if let Some(end) = end_time {
            query_params.push(("endTime", end.to_string()));
        }

        let response = self.client.get(&url).query(&query_params).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "K-lines request failed: {}",
                error_text
            )));
        }

        let klines_data: Vec<Vec<serde_json::Value>> = response.json().await?;

        let klines = klines_data
            .into_iter()
            .map(|kline_array| {
                // Binance returns k-lines as arrays, we need to parse them manually
                let open_time = kline_array[0].as_i64().unwrap_or(0);
                let open_price = kline_array[1].as_str().unwrap_or("0").to_string();
                let high_price = kline_array[2].as_str().unwrap_or("0").to_string();
                let low_price = kline_array[3].as_str().unwrap_or("0").to_string();
                let close_price = kline_array[4].as_str().unwrap_or("0").to_string();
                let volume = kline_array[5].as_str().unwrap_or("0").to_string();
                let close_time = kline_array[6].as_i64().unwrap_or(0);
                let number_of_trades = kline_array[8].as_i64().unwrap_or(0);

                Kline {
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
                }
            })
            .collect();

        Ok(klines)
    }
}
