use crate::core::errors::ExchangeError;
use crate::core::kernel::rest::RestClient;
use crate::core::traits::{FundingRateSource, MarketDataSource};
use crate::core::types::{
    FundingRate, Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig,
};
use crate::exchanges::paradex::codec::ParadexWsEvent;
use crate::exchanges::paradex::conversions::{
    convert_paradex_funding_rate, convert_paradex_kline, convert_paradex_market,
};
use crate::exchanges::paradex::rest::ParadexRestClient;
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{error, instrument};

/// Market data connector for Paradex
pub struct MarketData<R: RestClient, W = ()> {
    rest: ParadexRestClient<R>,
    _ws: Option<W>,
}

impl<R: RestClient + Clone> MarketData<R, ()> {
    pub fn new(rest: &R, _ws: Option<()>) -> Self {
        Self {
            rest: ParadexRestClient::new(rest.clone()),
            _ws: None,
        }
    }
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    pub fn new_with_ws(rest: &R, ws: W) -> Self {
        Self {
            rest: ParadexRestClient::new(rest.clone()),
            _ws: Some(ws),
        }
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> MarketDataSource for MarketData<R, W> {
    #[instrument(skip(self), fields(exchange = "paradex"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let paradex_markets = self.rest.get_markets().await?;
        Ok(paradex_markets
            .into_iter()
            .map(convert_paradex_market)
            .collect())
    }

    #[instrument(
        skip(self, _config),
        fields(
            exchange = "paradex",
            symbols_count = _symbols.len(),
            subscription_types = ?_subscription_types
        )
    )]
    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Check if WebSocket is available
        if self._ws.is_none() {
            return Err(ExchangeError::WebSocketError(
                "WebSocket not available in REST-only mode".to_string(),
            ));
        }

        // For now, return an error since WebSocket implementation needs the kernel WsSession
        // This will be implemented when the WebSocket session is properly integrated
        Err(ExchangeError::WebSocketError(
            "WebSocket integration in progress".to_string(),
        ))
    }

    fn get_websocket_url(&self) -> String {
        "wss://ws.paradex.trade/v1".to_string()
    }

    #[instrument(skip(self), fields(exchange = "paradex", symbol = %symbol))]
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let response = self
            .rest
            .get_klines(&symbol, interval, limit, start_time, end_time)
            .await?;

        // Parse the response and convert to Kline objects
        if let Some(data) = response.as_array() {
            let klines = data
                .iter()
                .filter_map(|item| convert_paradex_kline(item, &symbol))
                .collect();
            Ok(klines)
        } else {
            error!(
                symbol = %symbol,
                interval = ?interval,
                response = ?response,
                "Unexpected klines response format"
            );
            Err(ExchangeError::Other(
                "Unexpected klines response format".to_string(),
            ))
        }
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> FundingRateSource for MarketData<R, W> {
    #[instrument(skip(self), fields(exchange = "paradex"))]
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let paradex_rates = self.rest.get_funding_rates(symbols).await?;
        Ok(paradex_rates
            .into_iter()
            .map(convert_paradex_funding_rate)
            .collect())
    }

    #[instrument(skip(self), fields(exchange = "paradex"))]
    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        self.get_funding_rates(None).await
    }

    #[instrument(skip(self), fields(exchange = "paradex", symbol = %symbol))]
    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let history = self
            .rest
            .get_funding_rate_history(&symbol, start_time, end_time, limit)
            .await?;

        Ok(history
            .into_iter()
            .map(|h| FundingRate {
                symbol: crate::core::types::conversion::string_to_symbol(&h.symbol),
                funding_rate: Some(crate::core::types::conversion::string_to_decimal(
                    &h.funding_rate,
                )),
                previous_funding_rate: None,
                next_funding_rate: None,
                funding_time: Some(h.funding_time),
                next_funding_time: None,
                mark_price: None,
                index_price: None,
                timestamp: chrono::Utc::now().timestamp_millis(),
            })
            .collect())
    }
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    /// Helper function to convert WebSocket events to MarketDataType
    fn convert_ws_event(event: ParadexWsEvent) -> Option<MarketDataType> {
        match event {
            ParadexWsEvent::Ticker(ticker) => Some(MarketDataType::Ticker(ticker)),
            ParadexWsEvent::OrderBook(orderbook) => Some(MarketDataType::OrderBook(orderbook)),
            ParadexWsEvent::Trade(trade) => Some(MarketDataType::Trade(trade)),
            ParadexWsEvent::Kline(kline) => Some(MarketDataType::Kline(kline)),
            ParadexWsEvent::SubscriptionConfirmation(_) => None,
            ParadexWsEvent::Error(_) => None,
            ParadexWsEvent::Heartbeat => None,
        }
    }
}

/// Helper function to create subscription channels for Paradex WebSocket
fn create_subscription_channel(symbol: &str, subscription_type: &SubscriptionType) -> String {
    match subscription_type {
        SubscriptionType::Ticker => format!("ticker@{}", symbol),
        SubscriptionType::OrderBook { depth } => {
            if let Some(depth) = depth {
                format!("depth{}@{}", depth, symbol)
            } else {
                format!("depth@{}", symbol)
            }
        }
        SubscriptionType::Trades => format!("trade@{}", symbol),
        SubscriptionType::Klines { interval } => {
            format!("kline_{}@{}", interval.to_binance_format(), symbol)
        }
    }
}
