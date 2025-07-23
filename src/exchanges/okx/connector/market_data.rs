use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::MarketDataSource;
use crate::core::types::{
    Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig,
};

use crate::exchanges::okx::{conversions, rest::OkxRest};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// OKX market data implementation
#[derive(Debug)]
pub struct MarketData<R: RestClient, W = ()> {
    rest: OkxRest<R>,
    #[allow(dead_code)]
    ws: Option<W>,
    #[allow(dead_code)]
    testnet: bool,
}

impl<R: RestClient + Clone, W> MarketData<R, W> {
    pub fn new(rest: &R, ws: Option<W>, testnet: bool) -> Self {
        Self {
            rest: OkxRest::new(rest.clone()),
            ws,
            testnet,
        }
    }
}

#[async_trait]
impl<R: RestClient + Send + Sync, W: Send + Sync> MarketDataSource for MarketData<R, W> {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let okx_markets = self.rest.get_instruments("SPOT").await?;

        let mut markets = Vec::new();
        for okx_market in okx_markets {
            // Only include live markets
            if okx_market.state == "live" {
                match conversions::convert_okx_market(okx_market) {
                    Ok(market) => markets.push(market),
                    Err(e) => {
                        eprintln!("Failed to convert OKX market: {}", e);
                    }
                }
            }
        }

        Ok(markets)
    }

    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        _start_time: Option<i64>,
        _end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        // Convert KlineInterval to OKX bar format
        let bar = match interval {
            KlineInterval::Minutes1 => "1m",
            KlineInterval::Minutes3 => "3m",
            KlineInterval::Minutes5 => "5m",
            KlineInterval::Minutes15 => "15m",
            KlineInterval::Minutes30 => "30m",
            KlineInterval::Hours1 => "1H",
            KlineInterval::Hours2 => "2H",
            KlineInterval::Hours4 => "4H",
            KlineInterval::Hours6 => "6H",
            KlineInterval::Hours8 => "8H",
            KlineInterval::Hours12 => "12H",
            KlineInterval::Days1 => "1D",
            KlineInterval::Days3 => "3D",
            KlineInterval::Weeks1 => "1W",
            KlineInterval::Months1 => "1M",
        };

        let okx_klines = self
            .rest
            .get_candlesticks(&symbol, Some(bar), limit)
            .await?;

        let mut klines = Vec::new();
        for okx_kline in okx_klines {
            match conversions::convert_okx_kline(okx_kline, &symbol) {
                Ok(kline) => klines.push(kline),
                Err(e) => {
                    eprintln!("Failed to convert OKX kline: {}", e);
                }
            }
        }

        Ok(klines)
    }

    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // For now, return an error if WebSocket is not available
        // TODO: Implement WebSocket subscription logic when WsSession is available
        Err(ExchangeError::NotSupported(
            "WebSocket subscriptions not yet implemented for OKX".to_string(),
        ))
    }

    fn get_websocket_url(&self) -> String {
        "wss://ws.okx.com:8443/ws/v5/public".to_string()
    }
}
