use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClient, WsSession};
use crate::core::traits::MarketDataSource;
use crate::core::types::{
    Kline, KlineInterval, Market, MarketDataType, OrderBook, SubscriptionType, Ticker, Trade,
};
use crate::exchanges::okx::{
    codec::{OkxCodec, OkxMessage},
    conversions,
    rest::OkxRest,
    types::OkxMarket,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

/// OKX market data implementation
pub struct MarketData<R: RestClient, W = ()> {
    rest: OkxRest<R>,
    ws: Option<W>,
    testnet: bool,
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    pub fn new(rest: &R, ws: Option<W>, testnet: bool) -> Self {
        Self {
            rest: OkxRest::new(rest.clone()),
            ws,
            testnet,
        }
    }
}

#[async_trait]
impl<R: RestClient + Send + Sync, W: Send + Sync> MarketDataSource for MarketData<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let okx_markets = self.rest.get_instruments("SPOT").await?;

        let mut markets = Vec::new();
        for okx_market in okx_markets {
            // Only include live markets
            if okx_market.state == "live" {
                match conversions::convert_okx_market(okx_market) {
                    Ok(market) => markets.push(market),
                    Err(e) => {
                        log::warn!("Failed to convert OKX market: {}", e);
                    }
                }
            }
        }

        Ok(markets)
    }

    async fn get_ticker(&self, symbol: &str) -> Result<Ticker, ExchangeError> {
        let okx_ticker = self.rest.get_ticker(symbol).await?;
        conversions::convert_okx_ticker(okx_ticker).map_err(|e| ExchangeError::ParseError(e))
    }

    async fn get_order_book(&self, symbol: &str) -> Result<OrderBook, ExchangeError> {
        let okx_order_book = self.rest.get_order_book(symbol, Some(20)).await?;
        conversions::convert_okx_order_book(okx_order_book, symbol)
            .map_err(|e| ExchangeError::ParseError(e))
    }

    async fn get_recent_trades(&self, symbol: &str) -> Result<Vec<Trade>, ExchangeError> {
        let okx_trades = self.rest.get_trades(symbol, Some(100)).await?;

        let mut trades = Vec::new();
        for okx_trade in okx_trades {
            match conversions::convert_okx_trade(okx_trade) {
                Ok(trade) => trades.push(trade),
                Err(e) => {
                    log::warn!("Failed to convert OKX trade: {}", e);
                }
            }
        }

        Ok(trades)
    }

    async fn get_klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        // Convert KlineInterval to OKX bar format
        let bar = match interval {
            KlineInterval::Minute1 => "1m",
            KlineInterval::Minute5 => "5m",
            KlineInterval::Minute15 => "15m",
            KlineInterval::Minute30 => "30m",
            KlineInterval::Hour1 => "1H",
            KlineInterval::Hour4 => "4H",
            KlineInterval::Hour6 => "6H",
            KlineInterval::Hour12 => "12H",
            KlineInterval::Day1 => "1D",
            KlineInterval::Week1 => "1W",
            KlineInterval::Month1 => "1M",
        };

        let okx_klines = self.rest.get_candlesticks(symbol, Some(bar), limit).await?;

        let mut klines = Vec::new();
        for okx_kline in okx_klines {
            match conversions::convert_okx_kline(okx_kline, symbol) {
                Ok(kline) => klines.push(kline),
                Err(e) => {
                    log::warn!("Failed to convert OKX kline: {}", e);
                }
            }
        }

        Ok(klines)
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        data_types: Vec<MarketDataType>,
    ) -> Result<mpsc::Receiver<Result<MarketDataType, ExchangeError>>, ExchangeError> {
        // For now, return an error if WebSocket is not available
        // TODO: Implement WebSocket subscription logic when WsSession is available
        Err(ExchangeError::NotSupported(
            "WebSocket subscriptions not yet implemented for OKX".to_string(),
        ))
    }

    async fn unsubscribe_market_data(
        &self,
        symbols: Vec<String>,
        data_types: Vec<MarketDataType>,
    ) -> Result<(), ExchangeError> {
        // For now, return an error if WebSocket is not available
        // TODO: Implement WebSocket unsubscription logic when WsSession is available
        Err(ExchangeError::NotSupported(
            "WebSocket subscriptions not yet implemented for OKX".to_string(),
        ))
    }
}

// WebSocket implementation for when WsSession is available
impl<R: RestClient + Send + Sync, W: WsSession<OkxCodec> + Send + Sync> MarketData<R, W> {
    /// Subscribe to market data via WebSocket
    pub async fn subscribe_ws_market_data(
        &self,
        symbols: Vec<String>,
        data_types: Vec<MarketDataType>,
    ) -> Result<mpsc::Receiver<Result<MarketDataType, ExchangeError>>, ExchangeError> {
        if let Some(ref ws) = self.ws {
            let subscription_types: Vec<SubscriptionType> = data_types
                .iter()
                .map(|dt| match dt {
                    MarketDataType::Ticker(_) => SubscriptionType::Ticker,
                    MarketDataType::OrderBook(_) => SubscriptionType::OrderBook,
                    MarketDataType::Trade(_) => SubscriptionType::Trade,
                    MarketDataType::Kline(_, _) => SubscriptionType::Kline,
                })
                .collect();

            // Create receiver for market data
            let (tx, rx) = mpsc::channel(1000);

            // Subscribe to WebSocket streams
            let mut ws_session = ws.clone();
            let symbols_clone = symbols.clone();
            let subscription_types_clone = subscription_types.clone();

            tokio::spawn(async move {
                // Subscribe to channels
                if let Err(e) = ws_session
                    .subscribe(symbols_clone, subscription_types_clone)
                    .await
                {
                    let _ = tx.send(Err(e)).await;
                    return;
                }

                // Process incoming messages
                while let Ok(message) = ws_session.next_message().await {
                    match message {
                        OkxMessage::Data {
                            channel,
                            inst_id,
                            data,
                        } => {
                            let result = match channel.as_str() {
                                "tickers" => {
                                    if let Some(ref symbol) = inst_id {
                                        conversions::convert_okx_ws_ticker(&data, symbol)
                                            .map(|ticker| MarketDataType::Ticker(ticker))
                                            .map_err(|e| ExchangeError::ParseError(e))
                                    } else {
                                        Err(ExchangeError::ParseError(
                                            "Missing instrument ID".to_string(),
                                        ))
                                    }
                                }
                                "books" => {
                                    if let Some(ref symbol) = inst_id {
                                        conversions::convert_okx_ws_order_book(&data, symbol)
                                            .map(|order_book| MarketDataType::OrderBook(order_book))
                                            .map_err(|e| ExchangeError::ParseError(e))
                                    } else {
                                        Err(ExchangeError::ParseError(
                                            "Missing instrument ID".to_string(),
                                        ))
                                    }
                                }
                                "trades" => {
                                    if let Some(ref symbol) = inst_id {
                                        conversions::convert_okx_ws_trade(&data, symbol)
                                            .map(|trades| {
                                                if let Some(trade) = trades.into_iter().next() {
                                                    MarketDataType::Trade(trade)
                                                } else {
                                                    return Err(ExchangeError::ParseError(
                                                        "No trade data".to_string(),
                                                    ));
                                                }
                                            })
                                            .map_err(|e| ExchangeError::ParseError(e))
                                    } else {
                                        Err(ExchangeError::ParseError(
                                            "Missing instrument ID".to_string(),
                                        ))
                                    }
                                }
                                _ => Err(ExchangeError::ParseError(format!(
                                    "Unknown channel: {}",
                                    channel
                                ))),
                            };

                            if tx.send(result).await.is_err() {
                                break;
                            }
                        }
                        OkxMessage::Error { code, message } => {
                            let error = ExchangeError::ApiError(format!(
                                "OKX WebSocket error {}: {}",
                                code, message
                            ));
                            if tx.send(Err(error)).await.is_err() {
                                break;
                            }
                        }
                        _ => {} // Ignore other message types
                    }
                }
            });

            Ok(rx)
        } else {
            Err(ExchangeError::NotSupported(
                "WebSocket not available".to_string(),
            ))
        }
    }
}
