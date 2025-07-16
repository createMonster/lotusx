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
    async fn get_account_info(&self) -> Result<Vec<Balance>, ExchangeError> {
        self.account.get_account_info().await
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

    async fn get_ticker(&self, symbol: &str) -> Result<crate::core::types::Ticker, ExchangeError> {
        self.market.get_ticker(symbol).await
    }

    async fn get_order_book(
        &self,
        symbol: &str,
    ) -> Result<crate::core::types::OrderBook, ExchangeError> {
        self.market.get_order_book(symbol).await
    }

    async fn get_recent_trades(
        &self,
        symbol: &str,
    ) -> Result<Vec<crate::core::types::Trade>, ExchangeError> {
        self.market.get_recent_trades(symbol).await
    }

    async fn get_klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        limit: Option<u32>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        self.market.get_klines(symbol, interval, limit).await
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        data_types: Vec<MarketDataType>,
    ) -> Result<mpsc::Receiver<Result<MarketDataType, ExchangeError>>, ExchangeError> {
        self.market.subscribe_market_data(symbols, data_types).await
    }

    async fn unsubscribe_market_data(
        &self,
        symbols: Vec<String>,
        data_types: Vec<MarketDataType>,
    ) -> Result<(), ExchangeError> {
        self.market
            .unsubscribe_market_data(symbols, data_types)
            .await
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
        symbol: &str,
        order_id: &str,
    ) -> Result<OrderResponse, ExchangeError> {
        self.trading.cancel_order(symbol, order_id).await
    }

    async fn get_order_status(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> Result<OrderResponse, ExchangeError> {
        self.trading.get_order_status(symbol, order_id).await
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
    ) -> Result<Vec<OrderResponse>, ExchangeError> {
        self.trading.get_open_orders(symbol).await
    }
}
