use crate::core::errors::ExchangeError;
use crate::core::traits::{AccountInfo, MarketDataSource, OrderPlacer};
use crate::core::types::{
    Balance, Kline, KlineInterval, Market, MarketDataType, OrderRequest, OrderResponse, Position,
    SubscriptionType, WebSocketConfig,
};
use crate::core::{config::ExchangeConfig, kernel::RestClient, kernel::WsSession};
use crate::exchanges::backpack::codec::BackpackCodec;
use async_trait::async_trait;
use tokio::sync::mpsc;

pub mod account;
pub mod market_data;
pub mod trading;

pub use account::Account;
pub use market_data::MarketData;
pub use trading::Trading;

/// Backpack connector that composes all sub-trait implementations
pub struct BackpackConnector<R: RestClient, W = ()> {
    pub market: MarketData<R, W>,
    pub trading: Trading<R>,
    pub account: Account<R>,
}

impl<R: RestClient + Clone + Send + Sync, W: WsSession<BackpackCodec> + Send + Sync>
    BackpackConnector<R, W>
{
    /// Create a new Backpack connector with WebSocket support
    pub fn new(rest: R, ws: W, _config: ExchangeConfig) -> Self {
        Self {
            market: MarketData::<R, W>::new(&rest, Some(ws)),
            trading: Trading::new(&rest),
            account: Account::new(&rest),
        }
    }
}

impl<R: RestClient + Clone + Send + Sync> BackpackConnector<R, ()> {
    /// Create a new Backpack connector without WebSocket support
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
impl<R: RestClient + Clone + Send + Sync, W: WsSession<BackpackCodec> + Send + Sync>
    MarketDataSource for BackpackConnector<R, W>
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
impl<R: RestClient + Clone + Send + Sync> MarketDataSource for BackpackConnector<R, ()> {
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
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> OrderPlacer for BackpackConnector<R, W> {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        self.trading.place_order(order).await
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        self.trading.cancel_order(symbol, order_id).await
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> AccountInfo for BackpackConnector<R, W> {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        self.account.get_account_balance().await
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        self.account.get_positions().await
    }
}
