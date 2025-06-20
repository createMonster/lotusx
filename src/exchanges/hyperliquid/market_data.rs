use super::client::HyperliquidClient;
use super::types::{HyperliquidError, InfoRequest};
use crate::core::errors::ExchangeError;
use crate::core::traits::MarketDataSource;
use crate::core::types::{
    Kline, KlineInterval, Market, MarketDataType, SubscriptionType, Symbol, WebSocketConfig,
};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::{instrument, warn};

/// Helper to handle unavailable operations
#[cold]
#[inline(never)]
fn handle_unavailable_operation(operation: &str) -> HyperliquidError {
    warn!(operation = %operation, "Operation not supported by Hyperliquid");
    HyperliquidError::api_error(format!("Hyperliquid does not provide {} API", operation))
}

#[async_trait]
impl MarketDataSource for HyperliquidClient {
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let request = InfoRequest::Meta;
        let response: super::types::Universe = self.post_info_request(&request).await?;
        let markets = response
            .universe
            .into_iter()
            .map(|asset| {
                Market {
                    symbol: Symbol {
                        base: asset.name.clone(),
                        quote: "USD".to_string(), // Hyperliquid uses USD as quote currency
                        symbol: asset.name.clone(),
                    },
                    status: "TRADING".to_string(),
                    base_precision: 8, // Default precision
                    quote_precision: 2,
                    min_qty: Some(asset.sz_decimals.to_string()),
                    max_qty: None,
                    min_price: None,
                    max_price: None,
                }
            })
            .collect();
        Ok(markets)
    }

    #[instrument(skip(self, config), fields(exchange = "hyperliquid", symbols_count = symbols.len()))]
    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Delegate to the websocket module
        super::websocket::subscribe_market_data_impl(self, symbols, subscription_types, config)
            .await
    }

    fn get_websocket_url(&self) -> String {
        self.get_websocket_url()
    }

    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn get_klines(
        &self,
        _symbol: String,
        _interval: KlineInterval,
        _limit: Option<u32>,
        _start_time: Option<i64>,
        _end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        // Hyperliquid does not provide a k-lines/candlestick API for perpetuals as of the official documentation:
        // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals
        Err(ExchangeError::Other(
            handle_unavailable_operation("k-lines/candlestick").to_string(),
        ))
    }
}
