use crate::core::kernel::RestClient;
use crate::core::traits::{AccountInfo, ExchangeConnector, MarketDataSource, OrderPlacer};
use crate::exchanges::hyperliquid::rest::HyperliquidRest;
use async_trait::async_trait;

pub mod account;
pub mod market_data;
pub mod trading;

pub use account::Account;
pub use market_data::MarketData;
pub use trading::Trading;

/// Hyperliquid connector that composes all sub-trait implementations
pub struct HyperliquidConnector<R: RestClient, W = ()> {
    pub market: MarketData<R, W>,
    pub trading: Trading<R>,
    pub account: Account<R>,
}

impl<R: RestClient + Clone> HyperliquidConnector<R, ()> {
    pub fn new(rest: HyperliquidRest<R>) -> Self {
        Self {
            market: MarketData::new(rest.clone()),
            trading: Trading::new(rest.clone()),
            account: Account::new(rest),
        }
    }
}

impl<R: RestClient + Clone, W> HyperliquidConnector<R, W> {
    pub fn new_with_ws(rest: HyperliquidRest<R>, ws: W) -> Self
    where
        W: crate::core::kernel::WsSession<crate::exchanges::hyperliquid::codec::HyperliquidCodec>
            + Send
            + Sync,
    {
        Self {
            market: MarketData::new_with_ws(rest.clone(), ws),
            trading: Trading::new(rest.clone()),
            account: Account::new(rest),
        }
    }
}

// Implement the composite trait for convenience
#[async_trait]
impl<R: RestClient + Clone + Send + Sync> ExchangeConnector for HyperliquidConnector<R, ()> {}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W> ExchangeConnector for HyperliquidConnector<R, W> where
    W: crate::core::kernel::WsSession<crate::exchanges::hyperliquid::codec::HyperliquidCodec>
        + Send
        + Sync
{
}

// Delegate MarketDataSource methods to the market component
#[async_trait]
impl<R: RestClient + Clone + Send + Sync> MarketDataSource for HyperliquidConnector<R, ()> {
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
impl<R: RestClient + Clone + Send + Sync, W> MarketDataSource for HyperliquidConnector<R, W>
where
    W: crate::core::kernel::WsSession<crate::exchanges::hyperliquid::codec::HyperliquidCodec>
        + Send
        + Sync,
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

// Delegate OrderPlacer methods to the trading component
#[async_trait]
impl<R: RestClient + Clone + Send + Sync> OrderPlacer for HyperliquidConnector<R, ()> {
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
impl<R: RestClient + Clone + Send + Sync, W> OrderPlacer for HyperliquidConnector<R, W>
where
    W: crate::core::kernel::WsSession<crate::exchanges::hyperliquid::codec::HyperliquidCodec>
        + Send
        + Sync,
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

// Delegate AccountInfo methods to the account component
#[async_trait]
impl<R: RestClient + Clone + Send + Sync> AccountInfo for HyperliquidConnector<R, ()> {
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

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W> AccountInfo for HyperliquidConnector<R, W>
where
    W: crate::core::kernel::WsSession<crate::exchanges::hyperliquid::codec::HyperliquidCodec>
        + Send
        + Sync,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::kernel::ReqwestRest;
    use crate::exchanges::hyperliquid::signer::HyperliquidSigner;

    #[test]
    fn test_connector_creation() {
        let rest_client = ReqwestRest::new(
            "https://api.hyperliquid.xyz".to_string(),
            "hyperliquid".to_string(),
            None,
        )
        .unwrap();
        let hyperliquid_rest = HyperliquidRest::new(rest_client, None, false);
        let connector = HyperliquidConnector::new(hyperliquid_rest);

        // Test that we can access components
        assert!(connector.market.get_websocket_url().contains("hyperliquid"));
        assert!(!connector.trading.can_sign());
        assert!(!connector.account.can_sign());
    }

    #[test]
    fn test_connector_with_signer() {
        let rest_client = ReqwestRest::new(
            "https://api.hyperliquid.xyz".to_string(),
            "hyperliquid".to_string(),
            None,
        )
        .unwrap();
        let signer = HyperliquidSigner::new();
        let hyperliquid_rest = HyperliquidRest::new(rest_client, Some(signer), false);
        let connector = HyperliquidConnector::new(hyperliquid_rest);

        assert!(!connector.trading.can_sign()); // Should be false since we created an empty signer
        assert!(!connector.account.can_sign());
    }
}
