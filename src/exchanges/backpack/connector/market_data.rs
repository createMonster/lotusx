use crate::core::{
    errors::ExchangeError,
    kernel::{RestClient, WsSession},
    traits::MarketDataSource,
    types::{
        conversion, Kline, KlineInterval, Market, MarketDataType, Price, Quantity,
        SubscriptionType, Symbol, WebSocketConfig,
    },
};
use crate::exchanges::backpack::{codec::BackpackCodec, rest::BackpackRestClient};
use async_trait::async_trait;
use rust_decimal::Decimal;
use tokio::sync::mpsc;

/// Market data implementation for Backpack
pub struct MarketData<R: RestClient, W = ()> {
    rest: BackpackRestClient<R>,
    #[allow(dead_code)] // May be used for future WebSocket functionality
    ws: Option<W>,
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    fn ws_url(&self) -> String {
        "wss://ws.backpack.exchange".to_string()
    }
}

impl<R: RestClient + Clone, W: WsSession<BackpackCodec>> MarketData<R, W> {
    /// Create a new market data source with WebSocket support
    pub fn new(rest: &R, ws: Option<W>) -> Self {
        Self {
            rest: BackpackRestClient::new(rest.clone()),
            ws,
        }
    }
}

impl<R: RestClient + Clone> MarketData<R, ()> {
    /// Create a new market data source without WebSocket support
    pub fn new(rest: &R, _ws: Option<()>) -> Self {
        Self {
            rest: BackpackRestClient::new(rest.clone()),
            ws: None,
        }
    }
}

#[async_trait]
impl<R: RestClient + Clone, W: WsSession<BackpackCodec>> MarketDataSource for MarketData<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let markets = self.rest.get_markets().await?;

        Ok(markets
            .into_iter()
            .map(|m| Market {
                symbol: Symbol {
                    base: m.base_symbol,
                    quote: m.quote_symbol,
                },
                status: m.order_book_state,
                base_precision: 8,  // Default precision
                quote_precision: 8, // Default precision
                min_qty: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.quantity.as_ref())
                    .and_then(|q| q.min_quantity.as_ref())
                    .map(|s| conversion::string_to_quantity(s))
                    .or_else(|| Some(Quantity::new(Decimal::from(0)))),
                max_qty: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.quantity.as_ref())
                    .and_then(|q| q.max_quantity.as_ref())
                    .map(|s| conversion::string_to_quantity(s))
                    .or_else(|| Some(Quantity::new(Decimal::from(999_999_999)))),
                min_price: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.price.as_ref())
                    .and_then(|p| p.min_price.as_ref())
                    .map(|s| conversion::string_to_price(s))
                    .or_else(|| Some(Price::new(Decimal::from(0)))),
                max_price: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.price.as_ref())
                    .and_then(|p| p.max_price.as_ref())
                    .map(|s| conversion::string_to_price(s))
                    .or_else(|| Some(Price::new(Decimal::from(999_999_999)))),
            })
            .collect())
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Use the helper to create stream identifiers
        let streams = crate::exchanges::backpack::create_backpack_stream_identifiers(
            &symbols,
            &subscription_types,
        );

        // Create WebSocket URL
        let ws_url = self.ws_url();

        // Use kernel WebSocket implementation with BackpackCodec
        let codec = crate::exchanges::backpack::codec::BackpackCodec::new();
        let ws_session =
            crate::core::kernel::ws::TungsteniteWs::new(ws_url, "backpack".to_string(), codec);

        // Add reconnection wrapper for production reliability
        let mut reconnect_ws = crate::core::kernel::ws::ReconnectWs::new(ws_session)
            .with_auto_resubscribe(true)
            .with_max_reconnect_attempts(u32::MAX);

        // Connect and subscribe
        reconnect_ws.connect().await.map_err(|e| {
            ExchangeError::Other(format!(
                "Failed to connect to WebSocket for symbols: {:?}, error: {}",
                symbols, e
            ))
        })?;

        if !streams.is_empty() {
            let stream_refs: Vec<&str> = streams.iter().map(|s| s.as_str()).collect();
            reconnect_ws.subscribe(&stream_refs).await.map_err(|e| {
                ExchangeError::Other(format!(
                    "Failed to subscribe to streams: {:?}, error: {}",
                    streams, e
                ))
            })?;
        }

        // Create channel for messages
        let (tx, rx) = mpsc::channel(1000);

        // Spawn task to handle messages
        tokio::spawn(async move {
            while let Some(result) = reconnect_ws.next_message().await {
                match result {
                    Ok(backpack_message) => {
                        // Convert BackpackMessage to MarketDataType
                        if let Some(market_data) =
                            convert_backpack_message_to_market_data(backpack_message)
                        {
                            if tx.send(market_data).await.is_err() {
                                break; // Receiver dropped
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {:?}", e);
                        // Continue processing to handle reconnection
                    }
                }
            }
        });

        Ok(rx)
    }

    fn get_websocket_url(&self) -> String {
        self.ws_url()
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = interval.to_backpack_format();
        let klines = self
            .rest
            .get_klines(&symbol, &interval_str, start_time, end_time, limit)
            .await?;

        Ok(klines
            .into_iter()
            .map(|k| Kline {
                symbol: conversion::string_to_symbol(&symbol),
                open_time: k.start.parse::<i64>().unwrap_or(0),
                close_time: k.end.parse::<i64>().unwrap_or(0),
                interval: interval_str.clone(),
                open_price: conversion::string_to_price(&k.open),
                high_price: conversion::string_to_price(&k.high),
                low_price: conversion::string_to_price(&k.low),
                close_price: conversion::string_to_price(&k.close),
                volume: conversion::string_to_volume(&k.volume),
                number_of_trades: k.trades.parse::<i64>().unwrap_or(0),
                final_bar: true, // Backpack doesn't indicate if bar is final
            })
            .collect())
    }
}

#[async_trait]
impl<R: RestClient + Clone> MarketDataSource for MarketData<R, ()> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let markets = self.rest.get_markets().await?;

        Ok(markets
            .into_iter()
            .map(|m| Market {
                symbol: Symbol {
                    base: m.base_symbol,
                    quote: m.quote_symbol,
                },
                status: m.order_book_state,
                base_precision: 8,  // Default precision
                quote_precision: 8, // Default precision
                min_qty: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.quantity.as_ref())
                    .and_then(|q| q.min_quantity.as_ref())
                    .map(|s| conversion::string_to_quantity(s))
                    .or_else(|| Some(Quantity::new(Decimal::from(0)))),
                max_qty: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.quantity.as_ref())
                    .and_then(|q| q.max_quantity.as_ref())
                    .map(|s| conversion::string_to_quantity(s))
                    .or_else(|| Some(Quantity::new(Decimal::from(999_999_999)))),
                min_price: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.price.as_ref())
                    .and_then(|p| p.min_price.as_ref())
                    .map(|s| conversion::string_to_price(s))
                    .or_else(|| Some(Price::new(Decimal::from(0)))),
                max_price: m
                    .filters
                    .as_ref()
                    .and_then(|f| f.price.as_ref())
                    .and_then(|p| p.max_price.as_ref())
                    .map(|s| conversion::string_to_price(s))
                    .or_else(|| Some(Price::new(Decimal::from(999_999_999)))),
            })
            .collect())
    }

    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        Err(ExchangeError::WebSocketError(
            "WebSocket not available in REST-only mode".to_string(),
        ))
    }

    fn get_websocket_url(&self) -> String {
        self.ws_url()
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = interval.to_backpack_format();
        let klines = self
            .rest
            .get_klines(&symbol, &interval_str, start_time, end_time, limit)
            .await?;

        Ok(klines
            .into_iter()
            .map(|k| Kline {
                symbol: conversion::string_to_symbol(&symbol),
                open_time: k.start.parse::<i64>().unwrap_or(0),
                close_time: k.end.parse::<i64>().unwrap_or(0),
                interval: interval_str.clone(),
                open_price: conversion::string_to_price(&k.open),
                high_price: conversion::string_to_price(&k.high),
                low_price: conversion::string_to_price(&k.low),
                close_price: conversion::string_to_price(&k.close),
                volume: conversion::string_to_volume(&k.volume),
                number_of_trades: k.trades.parse::<i64>().unwrap_or(0),
                final_bar: true, // Backpack doesn't indicate if bar is final
            })
            .collect())
    }
}

/// Extension trait for `KlineInterval` to support Backpack format
pub trait BackpackKlineInterval {
    fn to_backpack_format(&self) -> String;
}

impl BackpackKlineInterval for KlineInterval {
    fn to_backpack_format(&self) -> String {
        match self {
            Self::Minutes1 => "1m".to_string(),
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
            Self::Seconds1 => "1s".to_string(), // Backpack may not support seconds
        }
    }
}

/// Convert `BackpackMessage` to `MarketDataType`
fn convert_backpack_message_to_market_data(
    message: crate::exchanges::backpack::codec::BackpackMessage,
) -> Option<MarketDataType> {
    use crate::core::types::conversion;

    match message {
        crate::exchanges::backpack::codec::BackpackMessage::Ticker(ticker) => {
            let symbol = conversion::string_to_symbol(&ticker.s);
            let price = conversion::string_to_price(&ticker.c);
            // Backpack doesn't provide price change directly, calculate from open and close
            let open_price = conversion::string_to_price(&ticker.o);
            let close_price = conversion::string_to_price(&ticker.c);
            let price_change =
                crate::core::types::Price::new(close_price.value() - open_price.value());
            let price_change_percent = if open_price.value().is_zero() {
                rust_decimal::Decimal::ZERO
            } else {
                (close_price.value() - open_price.value()) / open_price.value()
                    * rust_decimal::Decimal::from(100)
            };
            let high_price = conversion::string_to_price(&ticker.h);
            let low_price = conversion::string_to_price(&ticker.l);
            let volume = conversion::string_to_volume(&ticker.v);
            let quote_volume = conversion::string_to_volume(&ticker.V);

            Some(MarketDataType::Ticker(crate::core::types::Ticker {
                symbol,
                price,
                price_change,
                price_change_percent,
                high_price,
                low_price,
                volume,
                quote_volume,
                open_time: 0, // Backpack doesn't provide this in the ticker
                close_time: ticker.E,
                count: ticker.n,
            }))
        }
        crate::exchanges::backpack::codec::BackpackMessage::Trade(trade) => {
            let symbol = conversion::string_to_symbol(&trade.s);
            let price = conversion::string_to_price(&trade.p);
            let quantity = conversion::string_to_quantity(&trade.q);

            Some(MarketDataType::Trade(crate::core::types::Trade {
                symbol,
                id: trade.t,
                price,
                quantity,
                time: trade.T,
                is_buyer_maker: trade.m,
            }))
        }
        crate::exchanges::backpack::codec::BackpackMessage::OrderBook(orderbook) => {
            let symbol = conversion::string_to_symbol(&orderbook.s);

            let bids = orderbook
                .b
                .iter()
                .map(|bid| crate::core::types::OrderBookEntry {
                    price: conversion::string_to_price(&bid[0]),
                    quantity: conversion::string_to_quantity(&bid[1]),
                })
                .collect();
            let asks = orderbook
                .a
                .iter()
                .map(|ask| crate::core::types::OrderBookEntry {
                    price: conversion::string_to_price(&ask[0]),
                    quantity: conversion::string_to_quantity(&ask[1]),
                })
                .collect();

            Some(MarketDataType::OrderBook(crate::core::types::OrderBook {
                symbol,
                bids,
                asks,
                last_update_id: orderbook.u,
            }))
        }
        crate::exchanges::backpack::codec::BackpackMessage::Kline(kline) => {
            let symbol = conversion::string_to_symbol(&kline.s);
            let open_price = conversion::string_to_price(&kline.o);
            let high_price = conversion::string_to_price(&kline.h);
            let low_price = conversion::string_to_price(&kline.l);
            let close_price = conversion::string_to_price(&kline.c);
            let volume = conversion::string_to_volume(&kline.v);

            Some(MarketDataType::Kline(crate::core::types::Kline {
                symbol,
                open_time: kline.t,
                close_time: kline.T,
                interval: "1m".to_string(), // Backpack doesn't provide interval in kline message
                open_price,
                high_price,
                low_price,
                close_price,
                volume,
                number_of_trades: kline.n,
                final_bar: kline.X,
            }))
        }
        _ => None, // Ignore other message types for now
    }
}
