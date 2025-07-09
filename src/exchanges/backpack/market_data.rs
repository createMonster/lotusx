use crate::core::{
    errors::ExchangeError,
    kernel::{RestClient, WsSession},
    traits::{FundingRateSource, MarketDataSource},
    types::{
        conversion, FundingRate, Kline, KlineInterval, Market, MarketDataType, Price, Quantity,
        SubscriptionType, Symbol, WebSocketConfig,
    },
};
use crate::exchanges::backpack::{
    codec::{BackpackCodec, BackpackMessage},
    connector::BackpackConnector,
    types::{BackpackFundingRate, BackpackKlineResponse, BackpackMarketResponse},
};
use async_trait::async_trait;

use rust_decimal::Decimal;
use tokio::sync::mpsc;

#[async_trait]
impl<R: RestClient, W: WsSession<BackpackCodec>> MarketDataSource for BackpackConnector<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let response: serde_json::Value = self.rest().get("/api/v1/markets", &[], false).await?;
        let markets: Vec<BackpackMarketResponse> =
            serde_json::from_value(response).map_err(|e| {
                ExchangeError::DeserializationError(format!("Failed to parse markets: {}", e))
            })?;

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
        // Create subscription stream identifiers
        let _streams = crate::exchanges::backpack::create_backpack_stream_identifiers(
            &symbols,
            &subscription_types,
        );

        // Create a channel for sending market data
        let (_tx, _rx) = mpsc::channel::<MarketDataType>(1000);

        // Clone the connector for moving into the async task
        // Since we need to modify the WebSocket session, we'll need to handle this differently
        // For now, return an error if WebSocket isn't configured
        if !self.is_websocket_connected() {
            return Err(ExchangeError::ConfigurationError(
                "WebSocket session not configured or connected".to_string(),
            ));
        }

        // Note: The WebSocket session is borrowed, so we can't move it into the async task
        // This is a design issue that needs to be addressed in the connector architecture
        // For now, we'll return an error and suggest using the direct WebSocket methods
        return Err(ExchangeError::ConfigurationError(
            "Use subscribe_websocket() and next_websocket_message() methods instead".to_string(),
        ));
    }

    fn get_websocket_url(&self) -> String {
        "wss://ws.backpack.exchange".to_string()
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
        let mut params = vec![
            ("symbol", symbol.as_str()),
            ("interval", interval_str.as_str()),
        ];

        let start_time_str = start_time.map(|t| t.to_string());
        let end_time_str = end_time.map(|t| t.to_string());
        let limit_str = limit.map(|l| l.to_string());

        if let Some(ref start) = start_time_str {
            params.push(("startTime", start.as_str()));
        }
        if let Some(ref end) = end_time_str {
            params.push(("endTime", end.as_str()));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        let response: serde_json::Value = self.rest().get("/api/v1/klines", &params, false).await?;
        let klines: Vec<BackpackKlineResponse> = serde_json::from_value(response).map_err(|e| {
            ExchangeError::DeserializationError(format!("Failed to parse klines: {}", e))
        })?;

        Ok(klines
            .into_iter()
            .map(|k| Kline {
                symbol: conversion::string_to_symbol(&symbol),
                open_time: k.start.parse::<i64>().unwrap_or(0),
                close_time: k.end.parse::<i64>().unwrap_or(0),
                interval: interval.to_backpack_format().to_string(),
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
impl<R: RestClient, W: WsSession<BackpackCodec>> FundingRateSource for BackpackConnector<R, W> {
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        if let Some(symbols) = symbols {
            let mut funding_rates = Vec::new();
            for symbol in symbols {
                let rate = self.get_single_funding_rate(&symbol).await?;
                funding_rates.push(rate);
            }
            Ok(funding_rates)
        } else {
            self.get_all_funding_rates().await
        }
    }

    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let start_time_str = start_time.map(|t| t.to_string());
        let end_time_str = end_time.map(|t| t.to_string());
        let limit_str = limit.map(|l| l.to_string());

        let mut params = vec![("symbol", symbol.as_str())];

        if let Some(ref start) = start_time_str {
            params.push(("startTime", start.as_str()));
        }
        if let Some(ref end) = end_time_str {
            params.push(("endTime", end.as_str()));
        }
        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        let response: serde_json::Value = self
            .rest()
            .get("/api/v1/funding/rates/history", &params, false)
            .await?;
        let funding_rates: Vec<BackpackFundingRate> =
            serde_json::from_value(response).map_err(|e| {
                ExchangeError::DeserializationError(format!(
                    "Failed to parse funding rate history: {}",
                    e
                ))
            })?;

        Ok(funding_rates
            .into_iter()
            .map(|f| FundingRate {
                symbol: conversion::string_to_symbol(&f.symbol),
                funding_rate: Some(conversion::string_to_decimal(&f.funding_rate)),
                previous_funding_rate: None,
                next_funding_rate: None,
                funding_time: Some(f.funding_time),
                next_funding_time: Some(f.next_funding_time),
                mark_price: None,
                index_price: None,
                timestamp: chrono::Utc::now().timestamp_millis(),
            })
            .collect())
    }

    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        let response: serde_json::Value =
            self.rest().get("/api/v1/funding/rates", &[], false).await?;
        let funding_rates: Vec<BackpackFundingRate> =
            serde_json::from_value(response).map_err(|e| {
                ExchangeError::DeserializationError(format!("Failed to parse funding rates: {}", e))
            })?;

        Ok(funding_rates
            .into_iter()
            .map(|f| FundingRate {
                symbol: conversion::string_to_symbol(&f.symbol),
                funding_rate: Some(conversion::string_to_decimal(&f.funding_rate)),
                previous_funding_rate: None,
                next_funding_rate: None,
                funding_time: Some(f.funding_time),
                next_funding_time: Some(f.next_funding_time),
                mark_price: None,
                index_price: None,
                timestamp: chrono::Utc::now().timestamp_millis(),
            })
            .collect())
    }
}

impl<R: RestClient, W: WsSession<BackpackCodec>> BackpackConnector<R, W> {
    async fn get_single_funding_rate(&self, symbol: &str) -> Result<FundingRate, ExchangeError> {
        let params = [("symbol", symbol)];
        let response: serde_json::Value = self
            .rest()
            .get("/api/v1/funding/rates", &params, false)
            .await?;
        let funding_rates: Vec<BackpackFundingRate> =
            serde_json::from_value(response).map_err(|e| {
                ExchangeError::DeserializationError(format!("Failed to parse funding rate: {}", e))
            })?;
        let funding_rate = funding_rates
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::Other("No funding rate found for symbol".to_string()))?;

        Ok(FundingRate {
            symbol: conversion::string_to_symbol(&funding_rate.symbol),
            funding_rate: Some(conversion::string_to_decimal(&funding_rate.funding_rate)),
            previous_funding_rate: None,
            next_funding_rate: None,
            funding_time: Some(funding_rate.funding_time),
            next_funding_time: Some(funding_rate.next_funding_time),
            mark_price: None,
            index_price: None,
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }
}

/// Helper functions for working with Backpack WebSocket messages
impl<R: RestClient, W: WsSession<BackpackCodec>> BackpackConnector<R, W> {
    /// Convert a BackpackMessage to MarketDataType
    pub fn convert_message_to_market_data(
        message: &BackpackMessage,
        _symbol: &str,
    ) -> Option<MarketDataType> {
        match message {
            BackpackMessage::Ticker(ticker) => {
                Some(MarketDataType::Ticker(crate::core::types::Ticker {
                    symbol: conversion::string_to_symbol(&ticker.s),
                    price: conversion::string_to_price(&ticker.c),
                    price_change: Price::new(Decimal::from(0)),
                    price_change_percent: Decimal::from(0),
                    high_price: conversion::string_to_price(&ticker.h),
                    low_price: conversion::string_to_price(&ticker.l),
                    volume: conversion::string_to_volume(&ticker.v),
                    quote_volume: conversion::string_to_volume(&ticker.V),
                    open_time: 0,
                    close_time: ticker.E,
                    count: ticker.n,
                }))
            }
            BackpackMessage::OrderBook(orderbook) => {
                Some(MarketDataType::OrderBook(crate::core::types::OrderBook {
                    symbol: conversion::string_to_symbol(&orderbook.s),
                    bids: orderbook
                        .b
                        .iter()
                        .map(|b| crate::core::types::OrderBookEntry {
                            price: conversion::string_to_price(&b[0]),
                            quantity: conversion::string_to_quantity(&b[1]),
                        })
                        .collect(),
                    asks: orderbook
                        .a
                        .iter()
                        .map(|a| crate::core::types::OrderBookEntry {
                            price: conversion::string_to_price(&a[0]),
                            quantity: conversion::string_to_quantity(&a[1]),
                        })
                        .collect(),
                    last_update_id: orderbook.u,
                }))
            }
            BackpackMessage::Trade(trade) => {
                Some(MarketDataType::Trade(crate::core::types::Trade {
                    symbol: conversion::string_to_symbol(&trade.s),
                    id: trade.t,
                    price: conversion::string_to_price(&trade.p),
                    quantity: conversion::string_to_quantity(&trade.q),
                    time: trade.T,
                    is_buyer_maker: trade.m,
                }))
            }
            BackpackMessage::Kline(kline) => {
                Some(MarketDataType::Kline(crate::core::types::Kline {
                    symbol: conversion::string_to_symbol(&kline.s),
                    open_time: kline.t,
                    close_time: kline.T,
                    interval: "1m".to_string(), // Default interval since kline doesn't include it
                    open_price: conversion::string_to_price(&kline.o),
                    high_price: conversion::string_to_price(&kline.h),
                    low_price: conversion::string_to_price(&kline.l),
                    close_price: conversion::string_to_price(&kline.c),
                    volume: conversion::string_to_volume(&kline.v),
                    number_of_trades: kline.n,
                    final_bar: kline.X,
                }))
            }
            _ => None,
        }
    }
}
