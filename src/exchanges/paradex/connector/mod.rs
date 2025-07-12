use crate::core::errors::ExchangeError;
use crate::core::traits::{AccountInfo, FundingRateSource, MarketDataSource, OrderPlacer};
use crate::core::types::{
    Balance, FundingRate, Kline, KlineInterval, Market, MarketDataType, OrderRequest,
    OrderResponse, Position, SubscriptionType, WebSocketConfig,
};
use crate::core::{config::ExchangeConfig, kernel::RestClient, kernel::WsSession};
use crate::exchanges::paradex::codec::ParadexCodec;
use async_trait::async_trait;
use tokio::sync::mpsc;

pub mod account;
pub mod market_data;
pub mod trading;

pub use account::Account;
pub use market_data::MarketData;
pub use trading::Trading;

/// Paradex connector that composes all sub-trait implementations
pub struct ParadexConnector<R: RestClient, W = ()> {
    pub market: MarketData<R, W>,
    pub trading: Trading<R>,
    pub account: Account<R>,
}

impl<R: RestClient + Clone + Send + Sync, W: WsSession<ParadexCodec> + Send + Sync>
    ParadexConnector<R, W>
{
    /// Create a new Paradex connector with WebSocket support
    pub fn new(rest: R, ws: W, _config: ExchangeConfig) -> Self {
        Self {
            market: MarketData::<R, W>::new_with_ws(&rest, ws),
            trading: Trading::new(&rest),
            account: Account::new(&rest),
        }
    }
}

impl<R: RestClient + Clone + Send + Sync> ParadexConnector<R, ()> {
    /// Create a new Paradex connector without WebSocket support
    pub fn new_without_ws(rest: R, _config: ExchangeConfig) -> Self {
        Self {
            market: MarketData::<R, ()>::new(&rest, None),
            trading: Trading::new(&rest),
            account: Account::new(&rest),
        }
    }
}

// Implement traits for the connector by delegating to sub-components

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: WsSession<ParadexCodec> + Send + Sync> MarketDataSource
    for ParadexConnector<R, W>
{
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        self.market.get_markets().await
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        self.market
            .subscribe_market_data(symbols, subscription_types, config)
            .await
    }

    fn get_websocket_url(&self) -> String {
        self.market.get_websocket_url()
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        self.market
            .get_klines(symbol, interval, limit, start_time, end_time)
            .await
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync> MarketDataSource for ParadexConnector<R, ()> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        self.market.get_markets().await
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
        self.market.get_websocket_url()
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        self.market
            .get_klines(symbol, interval, limit, start_time, end_time)
            .await
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> OrderPlacer for ParadexConnector<R, W> {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        self.trading.place_order(order).await
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        self.trading.cancel_order(symbol, order_id).await
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> AccountInfo for ParadexConnector<R, W> {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        self.account.get_account_balance().await
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        self.account.get_positions().await
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> FundingRateSource
    for ParadexConnector<R, W>
{
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        self.market.get_funding_rates(symbols).await
    }

    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        self.market.get_all_funding_rates().await
    }

    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        self.market
            .get_funding_rate_history(symbol, start_time, end_time, limit)
            .await
    }
}
