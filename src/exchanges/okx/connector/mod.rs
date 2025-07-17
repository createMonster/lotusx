use crate::core::errors::ExchangeError;
use crate::core::traits::{AccountInfo, MarketDataSource, OrderPlacer};
use crate::core::types::{
    Balance, Kline, KlineInterval, Market, MarketDataType, OrderRequest, OrderResponse, Position,
    SubscriptionType, WebSocketConfig,
};
use crate::core::{config::ExchangeConfig, kernel::RestClient, kernel::WsSession};
use crate::exchanges::okx::codec::OkxCodec;
use async_trait::async_trait;
use tokio::sync::mpsc;

pub mod account;
pub mod market_data;
pub mod trading;

pub use account::Account;
pub use market_data::MarketData;
pub use trading::Trading;

/// OKX connector that composes all sub-trait implementations
pub struct OkxConnector<R: RestClient, W = ()> {
    pub market: MarketData<R, W>,
    pub trading: Trading<R>,
    pub account: Account<R>,
}

impl<R: RestClient + Clone + Send + Sync, W: WsSession<OkxCodec> + Send + Sync> OkxConnector<R, W> {
    /// Create a new OKX connector with WebSocket support
    pub fn new_with_ws(rest: R, ws: W, config: ExchangeConfig) -> Self {
        Self {
            market: MarketData::<R, W>::new(&rest, Some(ws), config.testnet),
            trading: Trading::new(&rest),
            account: Account::new(&rest),
        }
    }
}

impl<R: RestClient + Clone + Send + Sync> OkxConnector<R, ()> {
    /// Create a new OKX connector without WebSocket support
    pub fn new_without_ws(rest: R, config: ExchangeConfig) -> Self {
        Self {
            market: MarketData::<R, ()>::new(&rest, None, config.testnet),
            trading: Trading::new(&rest),
            account: Account::new(&rest),
        }
    }
}

/// Implement AccountInfo trait for the OKX connector
#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> AccountInfo for OkxConnector<R, W> {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        self.account.get_account_balance().await
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        self.account.get_positions().await
    }
}

/// Implement MarketDataSource trait for the OKX connector
#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> MarketDataSource for OkxConnector<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        self.market.get_markets().await
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        self.market.subscribe_market_data(symbols, subscription_types, config).await
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
        self.market.get_klines(symbol, interval, limit, start_time, end_time).await
    }
}

/// Implement OrderPlacer trait for the OKX connector
#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> OrderPlacer for OkxConnector<R, W> {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        self.trading.place_order(order).await
    }

    async fn cancel_order(
        &self,
        symbol: String,
        order_id: String,
    ) -> Result<(), ExchangeError> {
        self.trading.cancel_order(symbol, order_id).await
    }
}
