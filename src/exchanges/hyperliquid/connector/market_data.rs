use crate::core::{
    errors::ExchangeError,
    kernel::{rest::RestClient, ws::WsSession},
    traits::MarketDataSource,
    types::{Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig},
};
use crate::exchanges::hyperliquid::{codec::HyperliquidCodec, conversions, rest::HyperliquidRest};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::instrument;

pub struct MarketData<R: RestClient, W = ()> {
    rest: HyperliquidRest<R>,
    #[allow(dead_code)]
    ws: Option<W>,
}

impl<R: RestClient + Clone> MarketData<R, ()> {
    pub fn new(rest: HyperliquidRest<R>) -> Self {
        Self { rest, ws: None }
    }
}

impl<R: RestClient + Clone, W: WsSession<HyperliquidCodec> + Send + Sync> MarketData<R, W> {
    pub fn new_with_ws(rest: HyperliquidRest<R>, ws: W) -> Self {
        Self { rest, ws: Some(ws) }
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync> MarketDataSource for MarketData<R, ()> {
    /// Get all available markets/trading pairs
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let assets = self.rest.get_markets().await?;
        Ok(assets
            .into_iter()
            .map(conversions::convert_asset_to_market)
            .collect())
    }

    /// Subscribe to market data via WebSocket
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // For now, return an error as we don't have WebSocket support in this implementation
        Err(ExchangeError::Other(
            "WebSocket subscriptions require WebSocket session".to_string(),
        ))
    }

    /// Get WebSocket endpoint URL for market data
    fn get_websocket_url(&self) -> String {
        self.rest.get_websocket_url()
    }

    /// Get historical k-lines/candlestick data
    #[instrument(skip(self), fields(exchange = "hyperliquid", symbol = %symbol, interval = ?interval))]
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = conversions::convert_kline_interval_to_hyperliquid(interval);
        let candles = self
            .rest
            .get_candlestick_snapshot(&symbol, &interval_str, start_time, end_time)
            .await?;

        // Apply limit if specified
        let mut klines: Vec<Kline> = candles
            .into_iter()
            .map(|c| conversions::convert_candle_to_kline(&c, &symbol, interval))
            .collect();

        if let Some(limit) = limit {
            klines.truncate(limit as usize);
        }

        Ok(klines)
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync, W: WsSession<HyperliquidCodec> + Send + Sync>
    MarketDataSource for MarketData<R, W>
{
    /// Get all available markets/trading pairs
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let assets = self.rest.get_markets().await?;
        Ok(assets
            .into_iter()
            .map(conversions::convert_asset_to_market)
            .collect())
    }

    /// Subscribe to market data via WebSocket
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // WebSocket implementation would require a different approach
        // due to trait design limitations with mutable references
        Err(ExchangeError::Other(
            "WebSocket subscriptions not yet implemented".to_string(),
        ))
    }

    /// Get WebSocket endpoint URL for market data
    fn get_websocket_url(&self) -> String {
        self.rest.get_websocket_url()
    }

    /// Get historical k-lines/candlestick data
    #[instrument(skip(self), fields(exchange = "hyperliquid", symbol = %symbol, interval = ?interval))]
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = conversions::convert_kline_interval_to_hyperliquid(interval);
        let candles = self
            .rest
            .get_candlestick_snapshot(&symbol, &interval_str, start_time, end_time)
            .await?;

        // Apply limit if specified
        let mut klines: Vec<Kline> = candles
            .into_iter()
            .map(|c| conversions::convert_candle_to_kline(&c, &symbol, interval))
            .collect();

        if let Some(limit) = limit {
            klines.truncate(limit as usize);
        }

        Ok(klines)
    }
}

// Note: WebSocket implementation would need a different approach
// due to trait design limitations with mutable references
// For now, we focus on the REST-only implementation

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::kernel::rest::ReqwestRest;
    use crate::exchanges::hyperliquid::rest::HyperliquidRest;

    #[test]
    fn test_market_data_creation() {
        let rest_client = ReqwestRest::new(
            "https://api.hyperliquid.xyz".to_string(),
            "hyperliquid".to_string(),
            None,
        )
        .unwrap();
        let hyperliquid_rest = HyperliquidRest::new(rest_client, None, false);
        let market_data = MarketData::new(hyperliquid_rest);

        // Test basic functionality
        assert!(market_data.get_websocket_url().contains("hyperliquid"));
    }
}
