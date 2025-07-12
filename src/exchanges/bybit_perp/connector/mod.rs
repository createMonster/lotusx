use crate::core::config::ExchangeConfig;
use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::{AccountInfo, FundingRateSource, MarketDataSource, OrderPlacer};
use async_trait::async_trait;

pub mod account;
pub mod market_data;
pub mod trading;

pub use account::Account;
pub use market_data::MarketData;
pub use trading::Trading;

/// Bybit Perpetual connector that composes all sub-trait implementations
pub struct BybitPerpConnector<R: RestClient, W = ()> {
    pub market: MarketData<R, W>,
    pub trading: Trading<R>,
    pub account: Account<R>,
}

impl<R: RestClient + Clone + Send + Sync> BybitPerpConnector<R, ()> {
    pub fn new_without_ws(rest: R, config: ExchangeConfig) -> Self {
        Self {
            market: MarketData::with_testnet(&rest, None, config.testnet),
            trading: Trading::new(&rest),
            account: Account::new(&rest),
        }
    }
}

impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> BybitPerpConnector<R, W> {
    pub fn new(rest: R, ws: W, config: ExchangeConfig) -> Self {
        Self {
            market: MarketData::with_testnet(&rest, Some(ws), config.testnet),
            trading: Trading::new(&rest),
            account: Account::new(&rest),
        }
    }
}

// Implement traits for the connector by delegating to sub-components
#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> MarketDataSource
    for BybitPerpConnector<R, W>
{
    async fn get_markets(&self) -> Result<Vec<crate::core::types::Market>, ExchangeError> {
        self.market.get_markets().await
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: crate::core::types::KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<crate::core::types::Kline>, ExchangeError> {
        self.market
            .get_klines(symbol, interval, limit, start_time, end_time)
            .await
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<crate::core::types::SubscriptionType>,
        config: Option<crate::core::types::WebSocketConfig>,
    ) -> Result<tokio::sync::mpsc::Receiver<crate::core::types::MarketDataType>, ExchangeError>
    {
        self.market
            .subscribe_market_data(symbols, subscription_types, config)
            .await
    }

    fn get_websocket_url(&self) -> String {
        self.market.get_websocket_url()
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> FundingRateSource
    for BybitPerpConnector<R, W>
{
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<crate::core::types::FundingRate>, ExchangeError> {
        self.market.get_funding_rates(symbols).await
    }

    async fn get_all_funding_rates(
        &self,
    ) -> Result<Vec<crate::core::types::FundingRate>, ExchangeError> {
        self.market.get_all_funding_rates().await
    }

    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<crate::core::types::FundingRate>, ExchangeError> {
        self.market
            .get_funding_rate_history(symbol, start_time, end_time, limit)
            .await
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> OrderPlacer for BybitPerpConnector<R, W> {
    async fn place_order(
        &self,
        order: crate::core::types::OrderRequest,
    ) -> Result<crate::core::types::OrderResponse, ExchangeError> {
        self.trading.place_order(order).await
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        self.trading.cancel_order(symbol, order_id).await
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: Send + Sync> AccountInfo for BybitPerpConnector<R, W> {
    async fn get_account_balance(&self) -> Result<Vec<crate::core::types::Balance>, ExchangeError> {
        self.account.get_account_balance().await
    }

    async fn get_positions(&self) -> Result<Vec<crate::core::types::Position>, ExchangeError> {
        self.account.get_positions().await
    }
}
