use crate::core::{
    errors::ExchangeError,
    traits::MarketDataSource,
    types::{Kline, Market, MarketDataType, SubscriptionType, WebSocketConfig},
};
use crate::exchanges::backpack::{
    client::BackpackConnector,
    types::{
        BackpackApiResponse, BackpackMarket, BackpackOrderBook,
        BackpackTicker, BackpackTrade, BackpackWebSocketMessage,
        BackpackWebSocketSubscription,
    },
};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[async_trait]
impl MarketDataSource for BackpackConnector {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/api/v1/markets", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(ExchangeError::HttpError)?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get markets: {}", response.status()),
            });
        }

        let api_response: BackpackApiResponse<Vec<BackpackMarket>> = response
            .json()
            .await
            .map_err(|e| ExchangeError::Other(format!("Failed to parse markets response: {}", e)))?;

        if !api_response.success {
            return Err(ExchangeError::ApiError {
                code: api_response.error.as_ref().map(|e| e.code).unwrap_or(-1),
                message: api_response.error.map(|e| e.msg).unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        let markets = api_response.data.ok_or_else(|| {
            ExchangeError::ApiError {
                code: -1,
                message: "No market data received".to_string(),
            }
        })?;

        Ok(markets.into_iter().map(|m| Market {
            symbol: crate::core::types::Symbol {
                base: m.base_asset,
                quote: m.quote_asset,
                symbol: m.symbol,
            },
            status: m.status,
            base_precision: m.base_precision,
            quote_precision: m.quote_precision,
            min_qty: Some(m.min_qty),
            max_qty: Some(m.max_qty),
            min_price: Some(m.min_price),
            max_price: Some(m.max_price),
        }).collect())
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        let ws_url = "wss://ws.backpack.exchange/stream";
        
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("WebSocket connection failed: {}", e)))?;

        let (mut write, read) = ws_stream.split();

        // Create subscription requests
        let mut subscription_params = Vec::new();
        
        for symbol in &symbols {
            for sub_type in &subscription_types {
                match sub_type {
                    SubscriptionType::Ticker => {
                        subscription_params.push(format!("{}@ticker", symbol.to_lowercase()));
                    }
                    SubscriptionType::OrderBook { depth } => {
                        let depth_str = depth.map(|d| format!("@{}", d)).unwrap_or_else(|| "@20".to_string());
                        subscription_params.push(format!("{}@depth{}", symbol.to_lowercase(), depth_str));
                    }
                    SubscriptionType::Trades => {
                        subscription_params.push(format!("{}@trade", symbol.to_lowercase()));
                    }
                    SubscriptionType::Klines { interval } => {
                        subscription_params.push(format!("{}@kline_{}", symbol.to_lowercase(), interval));
                    }
                }
            }
        }

        // Send subscription request
        let subscription = BackpackWebSocketSubscription {
            method: "SUBSCRIBE".to_string(),
            params: subscription_params,
            id: 1,
        };

        let subscription_msg = serde_json::to_string(&subscription)
            .map_err(|e| ExchangeError::Other(format!("Failed to serialize subscription: {}", e)))?;

        write.send(Message::Text(subscription_msg))
            .await
            .map_err(|e| ExchangeError::NetworkError(format!("Failed to send subscription: {}", e)))?;

        // Create channel for market data
        let (tx, rx) = mpsc::channel(1000);

        // Spawn task to handle WebSocket messages
        tokio::spawn(async move {
            let mut read = read;
            
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(ws_message) = serde_json::from_str::<BackpackWebSocketMessage>(&text) {
                            let market_data = match ws_message {
                                BackpackWebSocketMessage::Ticker(ticker) => {
                                    Some(MarketDataType::Ticker(crate::core::types::Ticker {
                                        symbol: ticker.s,
                                        price: ticker.c,
                                        price_change: "0".to_string(), // Not available in WebSocket
                                        price_change_percent: "0".to_string(), // Not available in WebSocket
                                        high_price: ticker.h,
                                        low_price: ticker.l,
                                        volume: ticker.v,
                                        quote_volume: ticker.V,
                                        open_time: 0, // Not available in WebSocket
                                        close_time: ticker.E,
                                        count: ticker.n,
                                    }))
                                }
                                BackpackWebSocketMessage::OrderBook(orderbook) => {
                                    Some(MarketDataType::OrderBook(crate::core::types::OrderBook {
                                        symbol: orderbook.s,
                                        bids: orderbook.b.iter().map(|b| crate::core::types::OrderBookEntry {
                                            price: b[0].clone(),
                                            quantity: b[1].clone(),
                                        }).collect(),
                                        asks: orderbook.a.iter().map(|a| crate::core::types::OrderBookEntry {
                                            price: a[0].clone(),
                                            quantity: a[1].clone(),
                                        }).collect(),
                                        last_update_id: orderbook.u,
                                    }))
                                }
                                BackpackWebSocketMessage::Trade(trade) => {
                                    Some(MarketDataType::Trade(crate::core::types::Trade {
                                        symbol: trade.s,
                                        id: trade.t,
                                        price: trade.p,
                                        quantity: trade.q,
                                        time: trade.T,
                                        is_buyer_maker: trade.m,
                                    }))
                                }
                                BackpackWebSocketMessage::Kline(kline) => {
                                    Some(MarketDataType::Kline(crate::core::types::Kline {
                                        symbol: kline.s,
                                        open_time: kline.t,
                                        close_time: kline.T,
                                        interval: "1m".to_string(), // Default, should be extracted from subscription
                                        open_price: kline.o,
                                        high_price: kline.h,
                                        low_price: kline.l,
                                        close_price: kline.c,
                                        volume: kline.v,
                                        number_of_trades: kline.n,
                                        final_bar: kline.X,
                                    }))
                                }
                                _ => None, // Ignore other message types for now
                            };

                            if let Some(data) = market_data {
                                if tx.send(data).await.is_err() {
                                    break; // Receiver closed
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(rx)
    }

    fn get_websocket_url(&self) -> String {
        "wss://ws.backpack.exchange/stream".to_string()
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: String,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let mut params = vec![
            ("symbol".to_string(), symbol.clone()),
            ("interval".to_string(), interval.clone()),
        ];

        if let Some(limit) = limit {
            params.push(("limit".to_string(), limit.to_string()));
        }

        if let Some(start_time) = start_time {
            params.push(("startTime".to_string(), start_time.to_string()));
        }

        if let Some(end_time) = end_time {
            params.push(("endTime".to_string(), end_time.to_string()));
        }

        let query_string = Self::create_query_string(&params);
        let url = format!("{}/api/v1/klines?{}", self.base_url, query_string);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(ExchangeError::HttpError)?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get klines: {}", response.status()),
            });
        }

        let klines_data: Vec<Vec<serde_json::Value>> = response
            .json()
            .await
            .map_err(|e| ExchangeError::Other(format!("Failed to parse klines response: {}", e)))?;

        let klines = klines_data.into_iter().map(|kline| {
            Kline {
                symbol: symbol.clone(),
                open_time: kline[0].as_i64().unwrap_or(0),
                close_time: kline[6].as_i64().unwrap_or(0),
                interval: interval.clone(),
                open_price: kline[1].as_str().unwrap_or("0").to_string(),
                high_price: kline[2].as_str().unwrap_or("0").to_string(),
                low_price: kline[3].as_str().unwrap_or("0").to_string(),
                close_price: kline[4].as_str().unwrap_or("0").to_string(),
                volume: kline[5].as_str().unwrap_or("0").to_string(),
                number_of_trades: kline[8].as_i64().unwrap_or(0),
                final_bar: true, // Always true for historical data
            }
        }).collect();

        Ok(klines)
    }
}

impl BackpackConnector {
    /// Get ticker information for a symbol
    pub async fn get_ticker(&self, symbol: &str) -> Result<BackpackTicker, ExchangeError> {
        let url = format!("{}/api/v1/ticker/24hr", self.base_url);
        
        let params = vec![("symbol".to_string(), symbol.to_string())];
        let query_string = Self::create_query_string(&params);
        let full_url = format!("{}?{}", url, query_string);

        let response = self.client
            .get(&full_url)
            .send()
            .await
            .map_err(ExchangeError::HttpError)?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get ticker: {}", response.status()),
            });
        }

        let api_response: BackpackApiResponse<BackpackTicker> = response
            .json()
            .await
            .map_err(|e| ExchangeError::Other(format!("Failed to parse ticker response: {}", e)))?;

        if !api_response.success {
            return Err(ExchangeError::ApiError {
                code: api_response.error.as_ref().map(|e| e.code).unwrap_or(-1),
                message: api_response.error.map(|e| e.msg).unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        api_response.data.ok_or_else(|| {
            ExchangeError::ApiError {
                code: -1,
                message: "No ticker data received".to_string(),
            }
        })
    }

    /// Get order book for a symbol
    pub async fn get_order_book(&self, symbol: &str, limit: Option<u32>) -> Result<BackpackOrderBook, ExchangeError> {
        let mut params = vec![("symbol".to_string(), symbol.to_string())];
        
        if let Some(limit) = limit {
            params.push(("limit".to_string(), limit.to_string()));
        }

        let query_string = Self::create_query_string(&params);
        let url = format!("{}/api/v1/depth?{}", self.base_url, query_string);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(ExchangeError::HttpError)?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get order book: {}", response.status()),
            });
        }

        let api_response: BackpackApiResponse<BackpackOrderBook> = response
            .json()
            .await
            .map_err(|e| ExchangeError::Other(format!("Failed to parse order book response: {}", e)))?;

        if !api_response.success {
            return Err(ExchangeError::ApiError {
                code: api_response.error.as_ref().map(|e| e.code).unwrap_or(-1),
                message: api_response.error.map(|e| e.msg).unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        api_response.data.ok_or_else(|| {
            ExchangeError::ApiError {
                code: -1,
                message: "No order book data received".to_string(),
            }
        })
    }

    /// Get recent trades for a symbol
    pub async fn get_trades(&self, symbol: &str, limit: Option<u32>) -> Result<Vec<BackpackTrade>, ExchangeError> {
        let mut params = vec![("symbol".to_string(), symbol.to_string())];
        
        if let Some(limit) = limit {
            params.push(("limit".to_string(), limit.to_string()));
        }

        let query_string = Self::create_query_string(&params);
        let url = format!("{}/api/v1/trades?{}", self.base_url, query_string);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(ExchangeError::HttpError)?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get trades: {}", response.status()),
            });
        }

        let api_response: BackpackApiResponse<Vec<BackpackTrade>> = response
            .json()
            .await
            .map_err(|e| ExchangeError::Other(format!("Failed to parse trades response: {}", e)))?;

        if !api_response.success {
            return Err(ExchangeError::ApiError {
                code: api_response.error.as_ref().map(|e| e.code).unwrap_or(-1),
                message: api_response.error.map(|e| e.msg).unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        api_response.data.ok_or_else(|| {
            ExchangeError::ApiError {
                code: -1,
                message: "No trades data received".to_string(),
            }
        })
    }
} 