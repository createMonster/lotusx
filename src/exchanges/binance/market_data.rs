use super::codec::BinanceCodec;
use super::connector::BinanceConnector;
use super::types as binance_types;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClient, WsSession};
use tracing::instrument;

// The MarketDataSource trait is now implemented directly in connector.rs
// This file is kept for backwards compatibility but could be removed in the future

/// Extended market data functionality for Binance
impl<R: RestClient, W: WsSession<BinanceCodec>> BinanceConnector<R, W> {
    /// Get all tickers from Binance
    #[instrument(skip(self), fields(exchange = "binance"))]
    pub async fn get_all_tickers(
        &self,
    ) -> Result<Vec<binance_types::BinanceWebSocketTicker>, ExchangeError> {
        self.rest()
            .get_json("/api/v3/ticker/24hr", &[], false)
            .await
    }

    /// Get a specific ticker by symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn get_ticker_by_symbol(
        &self,
        symbol: &str,
    ) -> Result<binance_types::BinanceWebSocketTicker, ExchangeError> {
        let params = [("symbol", symbol)];
        self.rest()
            .get_json("/api/v3/ticker/24hr", &params, false)
            .await
    }

    /// Get order book depth for a symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn get_depth(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<binance_types::BinanceWebSocketOrderBook, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest().get_json("/api/v3/depth", &params, false).await
    }

    /// Get recent trades for a symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn get_recent_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> Result<Vec<binance_types::BinanceWebSocketTrade>, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest().get_json("/api/v3/trades", &params, false).await
    }

    /// Get historical trades for a symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn get_historical_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
        from_id: Option<u64>,
    ) -> Result<Vec<binance_types::BinanceWebSocketTrade>, ExchangeError> {
        let limit_str = limit.map(|l| l.to_string());
        let from_id_str = from_id.map(|id| id.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        if let Some(ref from_id) = from_id_str {
            params.push(("fromId", from_id.as_str()));
        }

        self.rest()
            .get_json("/api/v3/historicalTrades", &params, false)
            .await
    }

    /// Get aggregate trades for a symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn get_aggregate_trades(
        &self,
        symbol: &str,
        from_id: Option<u64>,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<Vec<serde_json::Value>, ExchangeError> {
        let from_id_str = from_id.map(|id| id.to_string());
        let start_time_str = start_time.map(|t| t.to_string());
        let end_time_str = end_time.map(|t| t.to_string());
        let limit_str = limit.map(|l| l.to_string());
        let mut params = vec![("symbol", symbol)];

        if let Some(ref from_id) = from_id_str {
            params.push(("fromId", from_id.as_str()));
        }

        if let Some(ref start_time) = start_time_str {
            params.push(("startTime", start_time.as_str()));
        }

        if let Some(ref end_time) = end_time_str {
            params.push(("endTime", end_time.as_str()));
        }

        if let Some(ref limit) = limit_str {
            params.push(("limit", limit.as_str()));
        }

        self.rest()
            .get_json("/api/v3/aggTrades", &params, false)
            .await
    }

    /// Get 24hr ticker price change statistics
    #[instrument(skip(self), fields(exchange = "binance"))]
    pub async fn get_24hr_ticker_stats(
        &self,
        symbol: Option<&str>,
    ) -> Result<serde_json::Value, ExchangeError> {
        let mut params = vec![];

        if let Some(symbol) = symbol {
            params.push(("symbol", symbol));
        }

        self.rest()
            .get_json("/api/v3/ticker/24hr", &params, false)
            .await
    }

    /// Get current average price for a symbol
    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol))]
    pub async fn get_average_price(
        &self,
        symbol: &str,
    ) -> Result<serde_json::Value, ExchangeError> {
        let params = [("symbol", symbol)];
        self.rest()
            .get_json("/api/v3/avgPrice", &params, false)
            .await
    }
}
