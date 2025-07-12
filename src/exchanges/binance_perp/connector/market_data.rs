use crate::core::{
    errors::ExchangeError,
    kernel::{RestClient, WsSession},
    traits::{FundingRateSource, MarketDataSource},
    types::{
        FundingRate, Kline, KlineInterval, Market, MarketDataType, SubscriptionType,
        WebSocketConfig,
    },
};
use crate::exchanges::binance_perp::{
    codec::BinancePerpCodec,
    conversions::{
        convert_binance_perp_market, convert_binance_perp_rest_kline, parse_websocket_message,
    },
    rest::BinancePerpRestClient,
};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::instrument;

/// Market data implementation for Binance Perpetual
pub struct MarketData<R: RestClient, W = ()> {
    rest: BinancePerpRestClient<R>,
    #[allow(dead_code)] // May be used for future WebSocket functionality
    ws: Option<W>,
    testnet: bool,
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    fn ws_url(&self) -> String {
        if self.testnet {
            "wss://stream.binancefuture.com/ws".to_string()
        } else {
            "wss://fstream.binance.com/ws".to_string()
        }
    }

    /// Convert Binance Perpetual funding rate to core type
    fn convert_funding_rate(
        &self,
        binance_rate: &crate::exchanges::binance_perp::types::BinancePerpFundingRate,
    ) -> FundingRate {
        FundingRate {
            symbol: crate::core::types::conversion::string_to_symbol(&binance_rate.symbol),
            funding_rate: Some(crate::core::types::conversion::string_to_decimal(
                &binance_rate.funding_rate,
            )),
            previous_funding_rate: None,
            next_funding_rate: None,
            funding_time: Some(binance_rate.funding_time),
            next_funding_time: None,
            mark_price: None,
            index_price: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        }
    }

    /// Convert Binance Perpetual funding rate with premium index data to core type
    fn convert_funding_rate_with_premium(
        &self,
        binance_rate: &crate::exchanges::binance_perp::types::BinancePerpFundingRate,
        premium_index: &crate::exchanges::binance_perp::types::BinancePerpPremiumIndex,
    ) -> FundingRate {
        FundingRate {
            symbol: crate::core::types::conversion::string_to_symbol(&binance_rate.symbol),
            funding_rate: Some(crate::core::types::conversion::string_to_decimal(
                &binance_rate.funding_rate,
            )),
            previous_funding_rate: None,
            next_funding_rate: None,
            funding_time: Some(binance_rate.funding_time),
            next_funding_time: Some(premium_index.next_funding_time),
            mark_price: Some(crate::core::types::conversion::string_to_price(
                &premium_index.mark_price,
            )),
            index_price: Some(crate::core::types::conversion::string_to_price(
                &premium_index.index_price,
            )),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        }
    }
}

impl<R: RestClient + Clone, W: WsSession<BinancePerpCodec>> MarketData<R, W> {
    /// Create a new market data source with WebSocket support
    pub fn new(rest: &R, ws: Option<W>, testnet: bool) -> Self {
        Self {
            rest: BinancePerpRestClient::new(rest.clone()),
            ws,
            testnet,
        }
    }
}

impl<R: RestClient + Clone> MarketData<R, ()> {
    /// Create a new market data source without WebSocket support
    pub fn new(rest: &R, _ws: Option<()>, testnet: bool) -> Self {
        Self {
            rest: BinancePerpRestClient::new(rest.clone()),
            ws: None,
            testnet,
        }
    }
}

#[async_trait]
impl<R: RestClient + Clone, W: WsSession<BinancePerpCodec>> MarketDataSource for MarketData<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let exchange_info = self.rest.get_exchange_info().await?;
        let markets = exchange_info
            .symbols
            .into_iter()
            .map(convert_binance_perp_market)
            .collect();
        Ok(markets)
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Use the codec helper to create stream identifiers
        let streams = crate::exchanges::binance_perp::codec::create_binance_perp_stream_identifiers(
            &symbols,
            &subscription_types,
        );

        // Create WebSocket URL
        let ws_url = self.ws_url();
        let full_url = crate::core::websocket::build_binance_stream_url(&ws_url, &streams);

        // Use WebSocket manager to start the stream
        let ws_manager = crate::core::websocket::WebSocketManager::new(full_url);
        ws_manager
            .start_stream(parse_websocket_message)
            .await
            .map_err(|e| {
                ExchangeError::Other(format!(
                    "Failed to start WebSocket stream for symbols: {:?}, error: {}",
                    symbols, e
                ))
            })
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
        let klines = self
            .rest
            .get_klines(&symbol, interval, limit, start_time, end_time)
            .await?;

        let converted_klines = klines
            .into_iter()
            .map(|k| {
                let mut kline = convert_binance_perp_rest_kline(&k);
                kline.symbol = crate::core::types::conversion::string_to_symbol(&symbol);
                kline.interval = interval.to_string();
                kline
            })
            .collect();

        Ok(converted_klines)
    }
}

#[async_trait]
impl<R: RestClient + Clone> MarketDataSource for MarketData<R, ()> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let exchange_info = self.rest.get_exchange_info().await?;
        let markets = exchange_info
            .symbols
            .into_iter()
            .map(convert_binance_perp_market)
            .collect();
        Ok(markets)
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
        let klines = self
            .rest
            .get_klines(&symbol, interval, limit, start_time, end_time)
            .await?;

        let converted_klines = klines
            .into_iter()
            .map(|k| {
                let mut kline = convert_binance_perp_rest_kline(&k);
                kline.symbol = crate::core::types::conversion::string_to_symbol(&symbol);
                kline.interval = interval.to_string();
                kline
            })
            .collect();

        Ok(converted_klines)
    }
}

#[async_trait]
impl<R: RestClient + Clone, W: Send + Sync> FundingRateSource for MarketData<R, W> {
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        if let Some(symbols) = symbols {
            let mut all_rates = Vec::new();
            for symbol in symbols {
                // Get both funding rate and premium index for complete data
                let (funding_rate, premium_index) = tokio::try_join!(
                    self.rest.get_funding_rate(&symbol),
                    self.rest.get_premium_index(&symbol)
                )?;
                all_rates
                    .push(self.convert_funding_rate_with_premium(&funding_rate, &premium_index));
            }
            Ok(all_rates)
        } else {
            let rates = self.rest.get_all_funding_rates().await?;
            // For all funding rates, we can't efficiently get premium index for each
            // So we'll just use the basic conversion for now
            Ok(rates
                .iter()
                .map(|rate| self.convert_funding_rate(rate))
                .collect())
        }
    }

    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        let rates = self.rest.get_all_funding_rates().await?;
        // For performance reasons with getting all funding rates, we'll use basic conversion
        // Individual funding rate requests will use the premium index for complete data
        Ok(rates
            .iter()
            .map(|rate| self.convert_funding_rate(rate))
            .collect())
    }

    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol))]
    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let rates = self
            .rest
            .get_funding_rate_history(&symbol, start_time, end_time, limit)
            .await?;

        Ok(rates
            .iter()
            .map(|rate| self.convert_funding_rate(rate))
            .collect())
    }
}
