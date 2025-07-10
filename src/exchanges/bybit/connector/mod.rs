use crate::core::config::ExchangeConfig;
use crate::core::kernel::RestClient;
use crate::core::traits::{AccountInfo, MarketDataSource, OrderPlacer};
use async_trait::async_trait;

pub mod account;
pub mod market_data;
pub mod trading;

pub use account::Account;
pub use market_data::MarketData;
pub use trading::Trading;

/// Bybit connector that composes all sub-trait implementations
pub struct BybitConnector<R: RestClient, W = ()> {
    pub market: MarketData<R, W>,
    pub trading: Trading<R>,
    pub account: Account<R>,
}

impl<R: RestClient + Clone + Send + Sync> BybitConnector<R, ()> {
    pub fn new(config: ExchangeConfig) -> BybitConnector<crate::core::kernel::ReqwestRest> {
        // Create a default REST client for factory usage
        let rest_config = crate::core::kernel::RestClientConfig::new(
            "https://api.bybit.com".to_string(),
            "bybit".to_string(),
        );

        let rest_client = crate::core::kernel::RestClientBuilder::new(rest_config)
            .build()
            .expect("Failed to create REST client");

        BybitConnector::new_with_rest(rest_client, config)
    }

    pub fn new_with_rest(rest: R, _config: ExchangeConfig) -> Self {
        Self {
            market: MarketData::new(rest.clone()),
            trading: Trading::new(&rest),
            account: Account::new(&rest),
        }
    }

    pub fn new_without_ws(rest: R, config: ExchangeConfig) -> Self {
        Self::new_with_rest(rest, config)
    }
}

// Concrete factory method for exchange factory usage
impl BybitConnector<crate::core::kernel::ReqwestRest> {
    pub fn for_factory(config: ExchangeConfig) -> Self {
        let rest_config = crate::core::kernel::RestClientConfig::new(
            "https://api.bybit.com".to_string(),
            "bybit".to_string(),
        );

        let rest_client = crate::core::kernel::RestClientBuilder::new(rest_config)
            .build()
            .expect("Failed to create REST client");

        Self::new_with_rest(rest_client, config)
    }
}

// Implement traits for the connector by delegating to sub-components
#[async_trait]
impl<R: RestClient + Clone + Send + Sync + 'static, W: Send + Sync + 'static> MarketDataSource
    for BybitConnector<R, W>
{
    async fn get_markets(
        &self,
    ) -> Result<Vec<crate::core::types::Market>, crate::core::errors::ExchangeError> {
        self.market.get_markets().await
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<crate::core::types::SubscriptionType>,
        config: Option<crate::core::types::WebSocketConfig>,
    ) -> Result<
        tokio::sync::mpsc::Receiver<crate::core::types::MarketDataType>,
        crate::core::errors::ExchangeError,
    > {
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
        interval: crate::core::types::KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<crate::core::types::Kline>, crate::core::errors::ExchangeError> {
        self.market
            .get_klines(symbol, interval, limit, start_time, end_time)
            .await
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync + 'static, W: Send + Sync + 'static> OrderPlacer
    for BybitConnector<R, W>
{
    async fn place_order(
        &self,
        order: crate::core::types::OrderRequest,
    ) -> Result<crate::core::types::OrderResponse, crate::core::errors::ExchangeError> {
        self.trading.place_order(order).await
    }

    async fn cancel_order(
        &self,
        symbol: String,
        order_id: String,
    ) -> Result<(), crate::core::errors::ExchangeError> {
        self.trading.cancel_order(symbol, order_id).await
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync + 'static, W: Send + Sync + 'static> AccountInfo
    for BybitConnector<R, W>
{
    async fn get_account_balance(
        &self,
    ) -> Result<Vec<crate::core::types::Balance>, crate::core::errors::ExchangeError> {
        self.account.get_account_balance().await
    }

    async fn get_positions(
        &self,
    ) -> Result<Vec<crate::core::types::Position>, crate::core::errors::ExchangeError> {
        self.account.get_positions().await
    }
}
