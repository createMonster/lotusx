use crate::core::{
    errors::{ExchangeError, ResultExt},
    traits::{FundingRateSource, MarketDataSource},
    types::{
        FundingRate, Kline, KlineInterval, Market, MarketDataType, SubscriptionType,
        WebSocketConfig,
    },
};
use crate::exchanges::backpack::{
    client::BackpackConnector,
    types::{
        BackpackDepthResponse, BackpackFundingRate, BackpackKlineResponse, BackpackMarkPrice,
        BackpackMarketResponse, BackpackTickerResponse, BackpackTradeResponse,
        BackpackWebSocketMessage, BackpackWebSocketSubscription,
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

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_exchange_context(|| format!("Failed to send request to {}", url))?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get markets: {}", response.status()),
            });
        }

        // Backpack API returns markets directly as an array, not wrapped
        let markets: Vec<BackpackMarketResponse> = response
            .json()
            .await
            .with_exchange_context(|| "Failed to parse markets response".to_string())?;

        Ok(markets
            .into_iter()
            .map(|m| Market {
                symbol: crate::core::types::Symbol {
                    base: m.base_symbol,
                    quote: m.quote_symbol,
                    symbol: m.symbol,
                },
                status: m.order_book_state,
                base_precision: 8,  // Default precision
                quote_precision: 8, // Default precision
                min_qty: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.quantity.as_ref())
                    .and_then(|q| q.min_quantity.clone())
                    .or_else(|| Some("0".to_string())),
                max_qty: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.quantity.as_ref())
                    .and_then(|q| q.max_quantity.clone())
                    .or_else(|| Some("999999999".to_string())),
                min_price: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.price.as_ref())
                    .and_then(|p| p.min_price.clone())
                    .or_else(|| Some("0".to_string())),
                max_price: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.price.as_ref())
                    .and_then(|p| p.max_price.clone())
                    .or_else(|| Some("999999999".to_string())),
            })
            .collect())
    }

    #[allow(clippy::too_many_lines)]
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        let ws_url = "wss://ws.backpack.exchange";

        let (ws_stream, _) = connect_async(ws_url).await.map_err(|e| {
            ExchangeError::NetworkError(format!("WebSocket connection failed to {}: {}", ws_url, e))
        })?;

        let (mut write, read) = ws_stream.split();

        // Create subscription requests according to Backpack API format
        let mut subscription_params = Vec::new();

        for symbol in &symbols {
            for sub_type in &subscription_types {
                match sub_type {
                    SubscriptionType::Ticker => {
                        subscription_params.push(format!("ticker.{}", symbol));
                    }
                    SubscriptionType::OrderBook { depth: _ } => {
                        subscription_params.push(format!("depth.{}", symbol));
                    }
                    SubscriptionType::Trades => {
                        subscription_params.push(format!("trade.{}", symbol));
                    }
                    SubscriptionType::Klines { interval } => {
                        subscription_params.push(format!(
                            "kline.{}.{}",
                            interval.to_backpack_format(),
                            symbol
                        ));
                    }
                }
            }
        }

        // Send subscription request
        let subscription = BackpackWebSocketSubscription {
            method: "SUBSCRIBE".to_string(),
            params: subscription_params.clone(),
            id: 1,
        };

        let subscription_msg =
            serde_json::to_string(&subscription).with_exchange_context(|| {
                format!(
                    "Failed to serialize subscription: params={:?}",
                    subscription_params
                )
            })?;

        write
            .send(Message::Text(subscription_msg))
            .await
            .map_err(|e| {
                ExchangeError::NetworkError(format!(
                    "Failed to send subscription to {}: {}",
                    ws_url, e
                ))
            })?;

        // Create channel for market data
        let (tx, rx) = mpsc::channel(1000);

        // Spawn task to handle WebSocket messages
        tokio::spawn(async move {
            let mut read = read;

            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(ws_message) =
                            serde_json::from_str::<BackpackWebSocketMessage>(&text)
                        {
                            let market_data = match ws_message {
                                BackpackWebSocketMessage::Ticker(ticker) => {
                                    Some(MarketDataType::Ticker(crate::core::types::Ticker {
                                        symbol: ticker.s,
                                        price: ticker.c,
                                        price_change: "0".to_string(),
                                        price_change_percent: "0".to_string(),
                                        high_price: ticker.h,
                                        low_price: ticker.l,
                                        volume: ticker.v,
                                        quote_volume: ticker.V,
                                        open_time: 0,
                                        close_time: ticker.E,
                                        count: ticker.n,
                                    }))
                                }
                                BackpackWebSocketMessage::OrderBook(orderbook) => {
                                    Some(MarketDataType::OrderBook(crate::core::types::OrderBook {
                                        symbol: orderbook.s,
                                        bids: orderbook
                                            .b
                                            .iter()
                                            .map(|b| crate::core::types::OrderBookEntry {
                                                price: b[0].clone(),
                                                quantity: b[1].clone(),
                                            })
                                            .collect(),
                                        asks: orderbook
                                            .a
                                            .iter()
                                            .map(|a| crate::core::types::OrderBookEntry {
                                                price: a[0].clone(),
                                                quantity: a[1].clone(),
                                            })
                                            .collect(),
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
                                        interval: "1m".to_string(),
                                        open_price: kline.o,
                                        high_price: kline.h,
                                        low_price: kline.l,
                                        close_price: kline.c,
                                        volume: kline.v,
                                        number_of_trades: kline.n,
                                        final_bar: kline.X,
                                    }))
                                }
                                _ => None,
                            };

                            if let Some(data) = market_data {
                                if tx.send(data).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(_) => {
                        // Don't log, just break and let the task end gracefully
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(rx)
    }

    fn get_websocket_url(&self) -> String {
        "wss://ws.backpack.exchange".to_string()
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        _limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = interval.to_backpack_format();
        let mut params = vec![
            ("symbol".to_string(), symbol.clone()),
            ("interval".to_string(), interval_str.clone()),
        ];

        if start_time.is_none() {
            // Default to last 24 hours if no start time provided
            #[allow(clippy::cast_possible_wrap)]
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| ExchangeError::Other(format!("System time error: {}", e)))?
                .as_secs()
                .min(i64::MAX as u64) as i64;
            params.push(("startTime".to_string(), (now - 86400).to_string()));
        }

        if let Some(end_time) = end_time {
            params.push(("endTime".to_string(), (end_time / 1000).to_string()));
        }

        let query_string = Self::create_query_string(&params);
        let url = format!("{}/api/v1/klines?{}", self.base_url, query_string);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_exchange_context(|| {
                format!(
                    "Failed to send klines request: url={}, symbol={}",
                    url, symbol
                )
            })?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get klines: {}", response.status()),
            });
        }

        // Backpack API returns klines directly as an array
        let klines_data: Vec<BackpackKlineResponse> =
            response.json().await.with_exchange_context(|| {
                format!("Failed to parse klines response for symbol {}", symbol)
            })?;

        let klines = klines_data
            .into_iter()
            .map(|kline| Kline {
                symbol: symbol.clone(),
                open_time: kline.start.parse().unwrap_or(0),
                close_time: kline.end.parse().unwrap_or(0),
                interval: interval_str.clone(),
                open_price: kline.open,
                high_price: kline.high,
                low_price: kline.low,
                close_price: kline.close,
                volume: kline.volume,
                number_of_trades: kline.trades.parse().unwrap_or(0),
                final_bar: true,
            })
            .collect();

        Ok(klines)
    }
}

impl BackpackConnector {
    /// Get ticker information for a symbol
    pub async fn get_ticker(
        &self,
        symbol: &str,
    ) -> Result<crate::core::types::Ticker, ExchangeError> {
        let params = vec![("symbol".to_string(), symbol.to_string())];
        let query_string = Self::create_query_string(&params);
        let url = format!("{}/api/v1/ticker?{}", self.base_url, query_string);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_exchange_context(|| {
                format!(
                    "Failed to send ticker request: url={}, symbol={}",
                    url, symbol
                )
            })?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get ticker: {}", response.status()),
            });
        }

        // Backpack API returns ticker directly, not wrapped
        let ticker: BackpackTickerResponse = response.json().await.with_exchange_context(|| {
            format!("Failed to parse ticker response for symbol {}", symbol)
        })?;

        Ok(crate::core::types::Ticker {
            symbol: ticker.symbol,
            price: ticker.last_price,
            price_change: ticker.price_change,
            price_change_percent: ticker.price_change_percent,
            high_price: ticker.high,
            low_price: ticker.low,
            volume: ticker.volume,
            quote_volume: ticker.quote_volume,
            open_time: 0,  // Not provided by Backpack API
            close_time: 0, // Not provided by Backpack API
            count: ticker.trades.parse().unwrap_or(0),
        })
    }

    /// Get order book for a symbol
    pub async fn get_order_book(
        &self,
        symbol: &str,
        _limit: Option<u32>,
    ) -> Result<crate::core::types::OrderBook, ExchangeError> {
        let params = vec![("symbol".to_string(), symbol.to_string())];
        let query_string = Self::create_query_string(&params);
        let url = format!("{}/api/v1/depth?{}", self.base_url, query_string);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_exchange_context(|| {
                format!(
                    "Failed to send order book request: url={}, symbol={}",
                    url, symbol
                )
            })?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get order book: {}", response.status()),
            });
        }

        // Backpack API returns depth directly, not wrapped
        let depth: BackpackDepthResponse = response.json().await.with_exchange_context(|| {
            format!("Failed to parse order book response for symbol {}", symbol)
        })?;

        Ok(crate::core::types::OrderBook {
            symbol: symbol.to_string(),
            bids: depth
                .bids
                .iter()
                .map(|b| crate::core::types::OrderBookEntry {
                    price: b[0].clone(),
                    quantity: b[1].clone(),
                })
                .collect(),
            asks: depth
                .asks
                .iter()
                .map(|a| crate::core::types::OrderBookEntry {
                    price: a[0].clone(),
                    quantity: a[1].clone(),
                })
                .collect(),
            last_update_id: depth.last_update_id.parse().unwrap_or(0),
        })
    }

    /// Get recent trades for a symbol
    pub async fn get_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<Vec<crate::core::types::Trade>, ExchangeError> {
        let mut params = vec![("symbol".to_string(), symbol.to_string())];

        if let Some(limit) = limit {
            params.push(("limit".to_string(), limit.to_string()));
        }

        let query_string = Self::create_query_string(&params);
        let url = format!("{}/api/v1/trades?{}", self.base_url, query_string);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_exchange_context(|| {
                format!(
                    "Failed to send trades request: url={}, symbol={}",
                    url, symbol
                )
            })?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get trades: {}", response.status()),
            });
        }

        // Backpack API returns trades directly as an array
        let trades: Vec<BackpackTradeResponse> =
            response.json().await.with_exchange_context(|| {
                format!("Failed to parse trades response for symbol {}", symbol)
            })?;

        Ok(trades
            .into_iter()
            .map(|trade| crate::core::types::Trade {
                symbol: symbol.to_string(),
                id: trade.id,
                price: trade.price,
                quantity: trade.quantity,
                time: trade.timestamp,
                is_buyer_maker: trade.is_buyer_maker,
            })
            .collect())
    }
}

// Funding Rate Implementation for Backpack
#[async_trait]
impl FundingRateSource for BackpackConnector {
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        match symbols {
            Some(symbol_list) if symbol_list.len() == 1 => {
                // Get funding rate for single symbol
                self.get_single_funding_rate(&symbol_list[0])
                    .await
                    .map(|rate| vec![rate])
            }
            Some(symbol_list) => {
                // Get funding rates for multiple symbols
                let mut results = Vec::new();
                for symbol in symbol_list {
                    if let Ok(rate) = self.get_single_funding_rate(&symbol).await {
                        results.push(rate);
                    }
                    // Skip symbols that don't have funding rates
                }
                Ok(results)
            }
            None => {
                // Get all funding rates
                self.get_all_funding_rates().await
            }
        }
    }

    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let mut params = vec![("symbol".to_string(), symbol.clone())];

        if let Some(limit) = limit {
            params.push(("limit".to_string(), limit.to_string()));
        }

        if let Some(start) = start_time {
            params.push(("startTime".to_string(), start.to_string()));
        }

        if let Some(end) = end_time {
            params.push(("endTime".to_string(), end.to_string()));
        }

        let query_string = Self::create_query_string(&params);
        let url = format!("{}/api/v1/fundingRate?{}", self.base_url, query_string);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_exchange_context(|| {
                format!(
                    "Failed to send funding rate history request: url={}, symbol={}",
                    url, symbol
                )
            })?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get funding rate history: {}", response.status()),
            });
        }

        let funding_rates: Vec<BackpackFundingRate> =
            response.json().await.with_exchange_context(|| {
                format!(
                    "Failed to parse funding rate history response for symbol {}",
                    symbol
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
                next_funding_time: Some(rate.next_funding_time),
                mark_price: None,
                index_price: None,
                timestamp: chrono::Utc::now().timestamp_millis(),
            });
        }

        Ok(result)
    }

    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        // Backpack doesn't have a single endpoint for all funding rates
        // We need to get all markets first and then get funding rates for perpetual markets
        let markets = self.get_markets().await?;

        let mut funding_rates = Vec::new();

        // Filter for perpetual markets and get their funding rates
        for market in markets {
            let symbol = &market.symbol.symbol;

            // Try to get funding rate for this symbol
            // Only perpetual futures will have funding rates
            if let Ok(funding_rate) = self.get_single_funding_rate(symbol).await {
                funding_rates.push(funding_rate);
            }
            // Continue with other symbols even if one fails
        }

        Ok(funding_rates)
    }
}

impl BackpackConnector {
    async fn get_single_funding_rate(&self, symbol: &str) -> Result<FundingRate, ExchangeError> {
        // First get the mark price data which includes funding rate info
        let params = vec![("symbol".to_string(), symbol.to_string())];
        let query_string = Self::create_query_string(&params);
        let url = format!("{}/api/v1/markPrice?{}", self.base_url, query_string);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .with_exchange_context(|| {
                format!(
                    "Failed to send mark price request: url={}, symbol={}",
                    url, symbol
                )
            })?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get mark price: {}", response.status()),
            });
        }

        let mark_price: BackpackMarkPrice = response.json().await.with_exchange_context(|| {
            format!("Failed to parse mark price response for symbol {}", symbol)
        })?;

        Ok(FundingRate {
            symbol: mark_price.symbol,
            funding_rate: Some(mark_price.estimated_funding_rate),
            previous_funding_rate: None,
            next_funding_rate: None,
            funding_time: None,
            next_funding_time: Some(mark_price.next_funding_time),
            mark_price: Some(mark_price.mark_price),
            index_price: Some(mark_price.index_price),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }
}
