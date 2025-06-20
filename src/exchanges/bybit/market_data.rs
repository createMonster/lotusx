use super::client::BybitConnector;
use super::converters::{convert_bybit_market, parse_websocket_message};
use super::types::{self as bybit_types, BybitError, BybitResultExt};
use crate::core::errors::ExchangeError;
use crate::core::traits::MarketDataSource;
use crate::core::types::{
    Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig,
};
use crate::core::websocket::WebSocketManager;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{instrument, warn};

/// Helper to check API response status and convert to proper error
#[cold]
#[inline(never)]
fn handle_api_response_error(ret_code: i32, ret_msg: String) -> BybitError {
    BybitError::api_error(ret_code, ret_msg)
}

#[async_trait]
impl MarketDataSource for BybitConnector {
    #[instrument(skip(self), fields(exchange = "bybit"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/v5/market/instruments-info?category=spot", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_symbol_context("*")?;

        let api_response: bybit_types::BybitApiResponse<bybit_types::BybitExchangeInfo> =
            response.json().await.with_symbol_context("*")?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::Other(
                handle_api_response_error(api_response.ret_code, api_response.ret_msg).to_string(),
            ));
        }

        let markets = api_response
            .result
            .list
            .into_iter()
            .map(convert_bybit_market)
            .collect();

        Ok(markets)
    }

    #[instrument(skip(self, _config), fields(exchange = "bybit", symbols_count = symbols.len()))]
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Build streams for Bybit WebSocket format
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
        let full_url = format!("{}?subscribe={}", ws_url, streams.join(","));

        let ws_manager = WebSocketManager::new(full_url);
        ws_manager.start_stream(parse_websocket_message).await
    }

    fn get_websocket_url(&self) -> String {
        if self.config.testnet {
            "wss://stream-testnet.bybit.com/v5/public/spot".to_string()
        } else {
            "wss://stream.bybit.com/v5/public/spot".to_string()
        }
    }

    #[instrument(skip(self), fields(exchange = "bybit", symbol = %symbol, interval = %interval))]
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
            "{}/v5/market/kline?category=spot&symbol={}&interval={}",
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
            .with_symbol_context(&symbol)?;

        if !response.status().is_success() {
            let error_text = response.text().await.with_symbol_context(&symbol)?;
            return Err(ExchangeError::Other(format!(
                "K-lines request failed for {}: {}",
                symbol, error_text
            )));
        }

        let klines_data: Vec<Vec<serde_json::Value>> =
            response.json().await.with_symbol_context(&symbol)?;

        let klines = klines_data
            .into_iter()
            .map(|kline_vec| {
                // Avoid unwrap() per HFT guidelines - handle parsing errors gracefully
                let start_time: i64 = kline_vec
                    .first()
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| {
                        warn!(symbol = %symbol, "Failed to parse kline start_time");
                        0
                    });
                let end_time = start_time + 60000; // Assuming 1 minute interval, adjust as needed

                Kline {
                    symbol: symbol.clone(),
                    open_time: start_time,
                    close_time: end_time,
                    interval: interval_str.clone(),
                    open_price: kline_vec
                        .get(1)
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    high_price: kline_vec
                        .get(2)
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    low_price: kline_vec
                        .get(3)
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    close_price: kline_vec
                        .get(4)
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    volume: kline_vec
                        .get(5)
                        .and_then(|v| v.as_str())
                        .unwrap_or("0")
                        .to_string(),
                    number_of_trades: 0,
                    final_bar: true,
                }
            })
            .collect();

        Ok(klines)
    }
}
