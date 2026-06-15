use crate::core::errors::ExchangeError;
use crate::core::kernel::{ws::WsSession, RestClient};
use crate::core::traits::MarketDataSource;
use crate::core::types::{
    conversion, Kline, KlineInterval, Market, MarketDataType, OrderBook, OrderBookEntry,
    SubscriptionType, Ticker, Trade, WebSocketConfig,
};
use crate::exchanges::bybit::codec::BybitWsEvent;
use crate::exchanges::bybit::conversions::{
    convert_bybit_kline, convert_bybit_market, kline_interval_to_bybit_string,
};
use crate::exchanges::bybit::types::{BybitApiResponse, BybitKlineResult, BybitMarketsResult};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Market data operations for Bybit
pub struct MarketData<R: RestClient, W = ()> {
    pub rest: R,
    pub _ws: std::marker::PhantomData<W>,
    pub testnet: bool,
}

impl<R: RestClient, W> MarketData<R, W> {
    pub fn new(rest: R) -> Self {
        Self {
            rest,
            _ws: std::marker::PhantomData,
            testnet: false, // Default to mainnet
        }
    }

    pub fn with_testnet(rest: R, testnet: bool) -> Self {
        Self {
            rest,
            _ws: std::marker::PhantomData,
            testnet,
        }
    }
}

#[async_trait]
impl<R: RestClient + 'static, W: Send + Sync + 'static> MarketDataSource for MarketData<R, W> {
    /// Get all available markets/trading pairs
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let response: BybitApiResponse<BybitMarketsResult> = self
            .rest
            .get_json(
                "/v5/market/instruments-info",
                &[("category", "spot")],
                false,
            )
            .await?;

        if response.ret_code != 0 {
            return Err(ExchangeError::ApiError {
                code: response.ret_code,
                message: response.ret_msg,
            });
        }

        let bybit_markets = response.result.list;
        let mut markets = Vec::new();

        for bybit_market in bybit_markets {
            if let Ok(market) = convert_bybit_market(&bybit_market) {
                markets.push(market);
            }
        }

        Ok(markets)
    }

    /// Subscribe to market data via WebSocket
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        let streams =
            crate::exchanges::bybit::create_bybit_stream_identifiers(&symbols, &subscription_types);
        let ws_url = self.get_websocket_url();
        let codec = crate::exchanges::bybit::codec::BybitCodec;
        let ws_session =
            crate::core::kernel::ws::TungsteniteWs::new(ws_url, "bybit".to_string(), codec);

        let mut reconnect_ws = crate::core::kernel::ws::ReconnectWs::new(ws_session)
            .with_auto_resubscribe(true)
            .with_max_reconnect_attempts(u32::MAX);

        reconnect_ws.connect().await.map_err(|e| {
            ExchangeError::Other(format!(
                "Failed to connect to WebSocket for symbols: {:?}, error: {}",
                symbols, e
            ))
        })?;

        if !streams.is_empty() {
            let stream_refs: Vec<&str> = streams.iter().map(String::as_str).collect();
            reconnect_ws.subscribe(&stream_refs).await.map_err(|e| {
                ExchangeError::Other(format!(
                    "Failed to subscribe to streams: {:?}, error: {}",
                    streams, e
                ))
            })?;
        }

        let (tx, rx) = mpsc::channel(1000);

        tokio::spawn(async move {
            while let Some(result) = reconnect_ws.next_message().await {
                match result {
                    Ok(bybit_event) => {
                        if let Some(market_data) = convert_bybit_event_to_market_data(bybit_event) {
                            if tx.send(market_data).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {:?}", e);
                    }
                }
            }
        });

        Ok(rx)
    }

    /// Get WebSocket endpoint URL for market data
    fn get_websocket_url(&self) -> String {
        if self.testnet {
            "wss://stream-testnet.bybit.com/v5/public/spot".to_string()
        } else {
            "wss://stream.bybit.com/v5/public/spot".to_string()
        }
    }

    /// Get historical k-lines/candlestick data
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = kline_interval_to_bybit_string(interval);
        let limit_str = limit.unwrap_or(200).to_string();

        let mut params = vec![
            ("category", "spot"),
            ("symbol", &symbol),
            ("interval", interval_str),
            ("limit", &limit_str),
        ];

        let start_time_str;
        let end_time_str;

        if let Some(start) = start_time {
            start_time_str = start.to_string();
            params.push(("start", &start_time_str));
        }

        if let Some(end) = end_time {
            end_time_str = end.to_string();
            params.push(("end", &end_time_str));
        }

        let response: BybitApiResponse<BybitKlineResult> = self
            .rest
            .get_json("/v5/market/kline", &params, false)
            .await?;

        if response.ret_code != 0 {
            return Err(ExchangeError::ApiError {
                code: response.ret_code,
                message: response.ret_msg,
            });
        }

        let bybit_klines = response.result.list;
        let mut klines = Vec::new();

        for bybit_kline in bybit_klines {
            if bybit_kline.len() >= 6 {
                let kline_data = crate::exchanges::bybit::types::BybitKlineData {
                    start_time: bybit_kline[0].parse::<i64>().unwrap_or_default(),
                    end_time: bybit_kline[0].parse::<i64>().unwrap_or_default() + 60000, // Approximate end time
                    interval: interval_str.to_string(),
                    open_price: bybit_kline[1].clone(),
                    high_price: bybit_kline[2].clone(),
                    low_price: bybit_kline[3].clone(),
                    close_price: bybit_kline[4].clone(),
                    volume: bybit_kline[5].clone(),
                    turnover: if bybit_kline.len() > 6 {
                        bybit_kline[6].clone()
                    } else {
                        "0".to_string()
                    },
                };

                if let Ok(kline) = convert_bybit_kline(&kline_data, &symbol, interval_str) {
                    klines.push(kline);
                }
            }
        }

        Ok(klines)
    }
}

fn convert_bybit_event_to_market_data(event: BybitWsEvent) -> Option<MarketDataType> {
    match event {
        BybitWsEvent::Ticker { data } => Some(MarketDataType::Ticker(Ticker {
            symbol: conversion::string_to_symbol(&data.symbol),
            price: conversion::string_to_price(&data.price),
            price_change: conversion::string_to_price("0"),
            price_change_percent: conversion::string_to_decimal(&data.price_24h_pcnt),
            high_price: conversion::string_to_price(&data.high_price_24h),
            low_price: conversion::string_to_price(&data.low_price_24h),
            volume: conversion::string_to_volume(&data.volume_24h),
            quote_volume: conversion::string_to_volume(&data.turnover_24h),
            open_time: data.timestamp.parse().unwrap_or(0),
            close_time: data.timestamp.parse().unwrap_or(0),
            count: 0,
        })),
        BybitWsEvent::OrderBook { data } => Some(MarketDataType::OrderBook(OrderBook {
            symbol: conversion::string_to_symbol(&data.symbol),
            bids: data
                .bids
                .into_iter()
                .map(|[price, quantity]| OrderBookEntry {
                    price: conversion::string_to_price(&price),
                    quantity: conversion::string_to_quantity(&quantity),
                })
                .collect(),
            asks: data
                .asks
                .into_iter()
                .map(|[price, quantity]| OrderBookEntry {
                    price: conversion::string_to_price(&price),
                    quantity: conversion::string_to_quantity(&quantity),
                })
                .collect(),
            last_update_id: data.update_id,
        })),
        BybitWsEvent::Trade { data } => Some(MarketDataType::Trade(Trade {
            symbol: conversion::string_to_symbol(&data.symbol),
            id: data.trade_id.parse().unwrap_or(0),
            price: conversion::string_to_price(&data.price),
            quantity: conversion::string_to_quantity(&data.size),
            time: data.timestamp,
            is_buyer_maker: data.side == "Sell",
        })),
        BybitWsEvent::Kline { data } => {
            convert_bybit_kline(&data.kline, &data.symbol, data.kline.interval.as_str())
                .ok()
                .map(MarketDataType::Kline)
        }
        BybitWsEvent::Pong { .. } | BybitWsEvent::Unknown => None,
    }
}
