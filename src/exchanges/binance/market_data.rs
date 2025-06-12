use super::client::BinanceConnector;
use super::converters::{convert_binance_market, parse_websocket_message};
use super::types as binance_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::MarketDataSource;
use crate::core::types::{Market, MarketDataType, SubscriptionType, WebSocketConfig};
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
}
