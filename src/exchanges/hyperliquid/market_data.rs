use super::client::HyperliquidClient;
use super::types::{HyperliquidError, InfoRequest};
use crate::core::errors::ExchangeError;
use crate::core::traits::{FundingRateSource, MarketDataSource};
use crate::core::types::{
    FundingRate, Kline, KlineInterval, Market, MarketDataType, SubscriptionType, Symbol,
    WebSocketConfig,
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

// Funding Rate Implementation for Hyperliquid
#[async_trait]
impl FundingRateSource for HyperliquidClient {
    #[instrument(skip(self), fields(symbols = ?symbols))]
    async fn get_funding_rates(
        &self,
        symbols: Option<Vec<String>>,
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        match symbols {
            Some(symbol_list) if symbol_list.len() == 1 => {
                // Get funding rate for single symbol
                self.get_single_funding_rate(&symbol_list[0])
                    .await
                    .map(|rate| vec![rate])
            }
            Some(_) | None => {
                // Get all funding rates
                self.get_all_funding_rates().await
            }
        }
    }

    #[instrument(skip(self))]
    async fn get_all_funding_rates(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        self.get_all_funding_rates_internal().await
    }

    #[instrument(skip(self), fields(symbol = %symbol))]
    async fn get_funding_rate_history(
        &self,
        symbol: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
        _limit: Option<u32>, // Hyperliquid doesn't support limit in funding history
    ) -> Result<Vec<FundingRate>, ExchangeError> {
        let request = InfoRequest::FundingHistory {
            coin: symbol.clone(),
            start_time: start_time.and_then(|t| u64::try_from(t).ok()),
            end_time: end_time.and_then(|t| u64::try_from(t).ok()),
        };

        match self
            .post_info_request::<Vec<super::types::FundingHistoryEntry>>(&request)
            .await
        {
            Ok(funding_history) => {
                let mut result = Vec::with_capacity(funding_history.len());
                for entry in funding_history {
                    result.push(FundingRate {
                        symbol: entry.coin,
                        funding_rate: Some(entry.funding_rate),
                        previous_funding_rate: None,
                        next_funding_rate: None,
                        funding_time: Some(i64::try_from(entry.time).unwrap_or(0)),
                        next_funding_time: None,
                        mark_price: None,
                        index_price: None,
                        timestamp: chrono::Utc::now().timestamp_millis(),
                    });
                }
                Ok(result)
            }
            Err(e) => {
                warn!(symbol = %symbol, error = %e, "Failed to get funding rate history");
                Err(ExchangeError::Other(
                    HyperliquidError::funding_rate_error(
                        format!("Failed to get funding rate history: {}", e),
                        Some(symbol),
                    )
                    .to_string(),
                ))
            }
        }
    }
}

impl HyperliquidClient {
    async fn get_single_funding_rate(&self, symbol: &str) -> Result<FundingRate, ExchangeError> {
        // Get current funding rate and mark price from meta endpoint
        let request = InfoRequest::MetaAndAssetCtxs;

        match self
            .post_info_request::<super::types::MetaAndAssetCtxsResponse>(&request)
            .await
        {
            Ok(response) => {
                // Find the asset context for this symbol
                for (i, asset) in response.universe.iter().enumerate() {
                    if asset.name == symbol {
                        if let Some(ctx) = response.asset_contexts.get(i) {
                            return Ok(FundingRate {
                                symbol: symbol.to_string(),
                                funding_rate: Some(ctx.funding.clone()),
                                previous_funding_rate: None,
                                next_funding_rate: None,
                                funding_time: None,
                                next_funding_time: None,
                                mark_price: Some(ctx.mark_px.clone()),
                                index_price: Some(ctx.oracle_px.clone()),
                                timestamp: chrono::Utc::now().timestamp_millis(),
                            });
                        }
                    }
                }

                Err(ExchangeError::Other(
                    HyperliquidError::funding_rate_error(
                        "Symbol not found in universe".to_string(),
                        Some(symbol.to_string()),
                    )
                    .to_string(),
                ))
            }
            Err(e) => {
                warn!(symbol = %symbol, error = %e, "Failed to get asset contexts");
                Err(ExchangeError::Other(
                    HyperliquidError::funding_rate_error(
                        format!("Failed to get asset contexts: {}", e),
                        Some(symbol.to_string()),
                    )
                    .to_string(),
                ))
            }
        }
    }

    async fn get_all_funding_rates_internal(&self) -> Result<Vec<FundingRate>, ExchangeError> {
        // Get all current funding rates and mark prices from meta endpoint
        let request = InfoRequest::MetaAndAssetCtxs;

        match self
            .post_info_request::<super::types::MetaAndAssetCtxsResponse>(&request)
            .await
        {
            Ok(response) => {
                let mut result = Vec::with_capacity(response.universe.len());

                for (i, asset) in response.universe.iter().enumerate() {
                    if let Some(ctx) = response.asset_contexts.get(i) {
                        result.push(FundingRate {
                            symbol: asset.name.clone(),
                            funding_rate: Some(ctx.funding.clone()),
                            previous_funding_rate: None,
                            next_funding_rate: None,
                            funding_time: None,
                            next_funding_time: None,
                            mark_price: Some(ctx.mark_px.clone()),
                            index_price: Some(ctx.oracle_px.clone()),
                            timestamp: chrono::Utc::now().timestamp_millis(),
                        });
                    }
                }

                Ok(result)
            }
            Err(e) => {
                warn!(error = %e, "Failed to get all asset contexts");
                Err(ExchangeError::Other(
                    HyperliquidError::funding_rate_error(
                        format!("Failed to get asset contexts: {}", e),
                        None,
                    )
                    .to_string(),
                ))
            }
        }
    }
}
