use crate::core::{
    config::ExchangeConfig,
    errors::ExchangeError,
    kernel::{RestClient, WsSession},
    traits::{ExchangeConnector, MarketDataSource},
    types::{Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig},
};
use crate::exchanges::binance::codec::{BinanceCodec, BinanceMessage};
use crate::exchanges::binance::converters::convert_binance_market;
use crate::exchanges::binance::types::{
    BinanceAccountInfo, BinanceExchangeInfo, BinanceOrderResponse, BinanceWebSocketOrderBook,
    BinanceWebSocketTicker, BinanceWebSocketTrade,
};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::instrument;

/// Binance connector using kernel architecture for optimal performance
pub struct BinanceConnector<R: RestClient, W: WsSession<BinanceCodec>> {
    rest: R,
    ws: Option<W>,
    base_url: String,
    config: ExchangeConfig,
}

impl<R: RestClient, W: WsSession<BinanceCodec>> BinanceConnector<R, W> {
    /// Create a new Binance connector with dependency injection
    pub fn new(rest: R, ws: Option<W>, config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://testnet.binance.vision".to_string()
        } else {
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.binance.com".to_string())
        };

        Self {
            rest,
            ws,
            base_url,
            config,
        }
    }

    /// Get the base URL for API requests
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Check if authentication is available
    pub fn can_authenticate(&self) -> bool {
        !self.config.api_key().is_empty() && !self.config.secret_key().is_empty()
    }

    /// Get a mutable reference to the WebSocket session
    pub fn ws_mut(&mut self) -> Option<&mut W> {
        self.ws.as_mut()
    }

    /// Get the current configuration
    pub fn config(&self) -> &ExchangeConfig {
        &self.config
    }

    /// Get the REST client
    pub fn rest(&self) -> &R {
        &self.rest
    }

    /// Get the WebSocket URL
    pub fn get_websocket_url(&self) -> String {
        if self.config.testnet {
            "wss://testnet.binance.vision/ws".to_string()
        } else {
            "wss://stream.binance.com:443/ws".to_string()
        }
    }
}

impl<R: RestClient, W: WsSession<BinanceCodec>> ExchangeConnector for BinanceConnector<R, W> {}

/// WebSocket functionality for Binance
impl<R: RestClient, W: WsSession<BinanceCodec>> BinanceConnector<R, W> {
    /// Subscribe to WebSocket streams
    pub async fn subscribe_websocket(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError> {
        if let Some(ws) = &mut self.ws {
            ws.connect().await?;
            ws.subscribe(streams).await?;
        } else {
            return Err(ExchangeError::ConfigurationError(
                "WebSocket session not configured".to_string(),
            ));
        }
        Ok(())
    }

    /// Unsubscribe from WebSocket streams
    pub async fn unsubscribe_websocket(
        &mut self,
        streams: &[impl AsRef<str> + Send + Sync],
    ) -> Result<(), ExchangeError> {
        if let Some(ws) = &mut self.ws {
            ws.unsubscribe(streams).await?;
        } else {
            return Err(ExchangeError::ConfigurationError(
                "WebSocket session not configured".to_string(),
            ));
        }
        Ok(())
    }

    /// Get the next WebSocket message
    pub async fn next_websocket_message(
        &mut self,
    ) -> Option<Result<BinanceMessage, ExchangeError>> {
        if let Some(ws) = &mut self.ws {
            ws.next_message().await
        } else {
            None
        }
    }

    /// Close the WebSocket connection
    pub async fn close_websocket(&mut self) -> Result<(), ExchangeError> {
        if let Some(ws) = &mut self.ws {
            ws.close().await?;
        }
        Ok(())
    }

    /// Check if WebSocket is connected
    pub fn is_websocket_connected(&self) -> bool {
        self.ws.as_ref().is_some_and(|ws| ws.is_connected())
    }
}

/// REST API functionality for Binance
impl<R: RestClient, W: WsSession<BinanceCodec>> BinanceConnector<R, W> {
    /// Get exchange info from REST API
    #[instrument(skip(self), fields(exchange = "binance"))]
    pub async fn get_exchange_info(&self) -> Result<BinanceExchangeInfo, ExchangeError> {
        self.rest.get_json("/api/v3/exchangeInfo", &[], false).await
    }

    /// Get ticker for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn get_ticker(&self, symbol: &str) -> Result<BinanceWebSocketTicker, ExchangeError> {
        let params = [("symbol", symbol)];
        self.rest
            .get_json("/api/v3/ticker/24hr", &params, false)
            .await
    }

    /// Get order book for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn get_order_book(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<BinanceWebSocketOrderBook, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json("/api/v3/depth", &params, false).await
    }

    /// Get recent trades for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn get_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<Vec<BinanceWebSocketTrade>, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json("/api/v3/trades", &params, false).await
    }

    /// Get klines for a specific symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol, interval = %interval))]
    pub async fn get_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<Vec<serde_json::Value>>, ExchangeError> {
        let start_str = start_time.map(|t| t.to_string());
        let end_str = end_time.map(|t| t.to_string());
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol), ("interval", interval)];

        if let Some(ref start) = start_str {
            params.push(("startTime", start.as_str()));
        }
        if let Some(ref end) = end_str {
            params.push(("endTime", end.as_str()));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest.get_json("/api/v3/klines", &params, false).await
    }
}

/// Authenticated endpoints for Binance
impl<R: RestClient, W: WsSession<BinanceCodec>> BinanceConnector<R, W> {
    /// Get account information
    #[instrument(skip(self), fields(exchange = "binance"))]
    pub async fn get_account_info(&self) -> Result<BinanceAccountInfo, ExchangeError> {
        if !self.can_authenticate() {
            return Err(ExchangeError::AuthError(
                "Missing API credentials for account access".to_string(),
            ));
        }

        self.rest.get_json("/api/v3/account", &[], true).await
    }

    /// Place a new order
    #[instrument(skip(self), fields(exchange = "binance"))]
    pub async fn place_order(
        &self,
        body: &serde_json::Value,
    ) -> Result<BinanceOrderResponse, ExchangeError> {
        if !self.can_authenticate() {
            return Err(ExchangeError::AuthError(
                "Missing API credentials for trading".to_string(),
            ));
        }

        self.rest.post_json("/api/v3/order", body, true).await
    }

    /// Cancel an order
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
    ) -> Result<BinanceOrderResponse, ExchangeError> {
        if !self.can_authenticate() {
            return Err(ExchangeError::AuthError(
                "Missing API credentials for trading".to_string(),
            ));
        }

        let order_id_str = order_id.map(|id| id.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref order_id) = order_id_str {
            params.push(("orderId", order_id.as_str()));
        }
        if let Some(orig_client_order_id) = orig_client_order_id {
            params.push(("origClientOrderId", orig_client_order_id));
        }

        self.rest.delete_json("/api/v3/order", &params, true).await
    }
}

/// Helper functions for working with Binance WebSocket messages
impl<R: RestClient, W: WsSession<BinanceCodec>> BinanceConnector<R, W> {
    /// Convert a `BinanceMessage` to core types
    pub fn convert_message_to_market_data(
        message: &BinanceMessage,
    ) -> Option<crate::core::types::MarketDataType> {
        match message {
            BinanceMessage::Ticker(ticker) => Some(crate::core::types::MarketDataType::Ticker(
                crate::core::types::Ticker {
                    symbol: crate::core::types::conversion::string_to_symbol(&ticker.symbol),
                    price: crate::core::types::conversion::string_to_price(&ticker.price),
                    price_change: crate::core::types::conversion::string_to_price(
                        &ticker.price_change,
                    ),
                    price_change_percent: crate::core::types::conversion::string_to_decimal(
                        &ticker.price_change_percent,
                    ),
                    high_price: crate::core::types::conversion::string_to_price(&ticker.high_price),
                    low_price: crate::core::types::conversion::string_to_price(&ticker.low_price),
                    volume: crate::core::types::conversion::string_to_volume(&ticker.volume),
                    quote_volume: crate::core::types::conversion::string_to_volume(
                        &ticker.quote_volume,
                    ),
                    open_time: ticker.open_time,
                    close_time: ticker.close_time,
                    count: ticker.count,
                },
            )),
            BinanceMessage::OrderBook(order_book) => {
                let bids = order_book
                    .bids
                    .iter()
                    .map(|b| crate::core::types::OrderBookEntry {
                        price: crate::core::types::conversion::string_to_price(&b[0]),
                        quantity: crate::core::types::conversion::string_to_quantity(&b[1]),
                    })
                    .collect();

                let asks = order_book
                    .asks
                    .iter()
                    .map(|a| crate::core::types::OrderBookEntry {
                        price: crate::core::types::conversion::string_to_price(&a[0]),
                        quantity: crate::core::types::conversion::string_to_quantity(&a[1]),
                    })
                    .collect();

                Some(crate::core::types::MarketDataType::OrderBook(
                    crate::core::types::OrderBook {
                        symbol: crate::core::types::conversion::string_to_symbol(
                            &order_book.symbol,
                        ),
                        bids,
                        asks,
                        last_update_id: order_book.final_update_id,
                    },
                ))
            }
            BinanceMessage::Trade(trade) => Some(crate::core::types::MarketDataType::Trade(
                crate::core::types::Trade {
                    symbol: crate::core::types::conversion::string_to_symbol(&trade.symbol),
                    id: trade.id,
                    price: crate::core::types::conversion::string_to_price(&trade.price),
                    quantity: crate::core::types::conversion::string_to_quantity(&trade.quantity),
                    time: trade.time,
                    is_buyer_maker: trade.is_buyer_maker,
                },
            )),
            BinanceMessage::Kline(kline) => Some(crate::core::types::MarketDataType::Kline(
                crate::core::types::Kline {
                    symbol: crate::core::types::conversion::string_to_symbol(&kline.symbol),
                    open_time: kline.kline.open_time,
                    close_time: kline.kline.close_time,
                    interval: kline.kline.interval.clone(),
                    open_price: crate::core::types::conversion::string_to_price(
                        &kline.kline.open_price,
                    ),
                    high_price: crate::core::types::conversion::string_to_price(
                        &kline.kline.high_price,
                    ),
                    low_price: crate::core::types::conversion::string_to_price(
                        &kline.kline.low_price,
                    ),
                    close_price: crate::core::types::conversion::string_to_price(
                        &kline.kline.close_price,
                    ),
                    volume: crate::core::types::conversion::string_to_volume(&kline.kline.volume),
                    number_of_trades: kline.kline.number_of_trades,
                    final_bar: kline.kline.final_bar,
                },
            )),
            BinanceMessage::Unknown => None,
        }
    }
}

/// MarketDataSource trait implementation
#[async_trait]
impl<R: RestClient, W: WsSession<BinanceCodec>> MarketDataSource for BinanceConnector<R, W> {
    #[instrument(skip(self), fields(exchange = "binance"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let exchange_info: BinanceExchangeInfo = self.get_exchange_info().await?;

        let markets = exchange_info
            .symbols
            .into_iter()
            .map(convert_binance_market)
            .collect::<Result<Vec<_>, _>>()
            .map_err(ExchangeError::Other)?;

        Ok(markets)
    }

    #[instrument(skip(self), fields(exchange = "binance", symbols = ?symbols))]
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Use the codec helper to create stream identifiers
        let streams = crate::exchanges::binance::codec::create_binance_stream_identifiers(
            &symbols,
            &subscription_types,
        );

        // Create WebSocket URL
        let ws_url = self.get_websocket_url();
        let full_url = crate::core::websocket::build_binance_stream_url(&ws_url, &streams);

        // Use WebSocket manager to start the stream
        let ws_manager = crate::core::websocket::WebSocketManager::new(full_url);
        ws_manager
            .start_stream(crate::exchanges::binance::converters::parse_websocket_message)
            .await
            .map_err(|e| {
                ExchangeError::Other(format!(
                    "Failed to start WebSocket stream for symbols: {:?}, error: {}",
                    symbols, e
                ))
            })
    }

    fn get_websocket_url(&self) -> String {
        self.get_websocket_url()
    }

    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = interval.to_binance_format();
        let klines_data = self
            .get_klines(&symbol, &interval_str, start_time, end_time, limit)
            .await?;

        let symbol_obj = crate::core::types::conversion::string_to_symbol(&symbol);

        let klines = klines_data
            .into_iter()
            .map(|kline_array| {
                // Binance returns k-lines as arrays, we need to parse them safely
                let open_time = kline_array.first().and_then(|v| v.as_i64()).unwrap_or(0);
                let open_price_str = kline_array.get(1).and_then(|v| v.as_str()).unwrap_or("0");
                let high_price_str = kline_array.get(2).and_then(|v| v.as_str()).unwrap_or("0");
                let low_price_str = kline_array.get(3).and_then(|v| v.as_str()).unwrap_or("0");
                let close_price_str = kline_array.get(4).and_then(|v| v.as_str()).unwrap_or("0");
                let volume_str = kline_array.get(5).and_then(|v| v.as_str()).unwrap_or("0");
                let close_time = kline_array.get(6).and_then(|v| v.as_i64()).unwrap_or(0);
                let number_of_trades = kline_array.get(8).and_then(|v| v.as_i64()).unwrap_or(0);

                // Parse all price/volume fields to proper types
                let open_price = crate::core::types::conversion::string_to_price(open_price_str);
                let high_price = crate::core::types::conversion::string_to_price(high_price_str);
                let low_price = crate::core::types::conversion::string_to_price(low_price_str);
                let close_price = crate::core::types::conversion::string_to_price(close_price_str);
                let volume = crate::core::types::conversion::string_to_volume(volume_str);

                Kline {
                    symbol: symbol_obj.clone(),
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

// AccountInfo and OrderPlacer trait implementations moved to separate files:
// - account.rs: AccountInfo trait implementation
// - trading.rs: OrderPlacer trait implementation
