use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::MarketDataSource;
use crate::core::types::{
    Kline, KlineInterval, Market, MarketDataType, SubscriptionType, WebSocketConfig,
};
use crate::exchanges::bybit::conversions::{
    convert_bybit_kline, convert_bybit_market, kline_interval_to_bybit_string,
};
use crate::exchanges::bybit::types::{BybitApiResponse, BybitKlineResult, BybitMarketsResult};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Market data operations for Bybit
pub struct MarketData<R: RestClient, W = ()> {
    pub rest: R,
    pub _ws: std::marker::PhantomData<W>,
}

impl<R: RestClient, W> MarketData<R, W> {
    pub fn new(rest: R) -> Self {
        Self {
            rest,
            _ws: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<R: RestClient + 'static, W: Send + Sync + 'static> MarketDataSource for MarketData<R, W> {
    /// Get all available markets/trading pairs
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let response: BybitApiResponse<BybitMarketsResult> = self
            .rest
            .get_json(
                "/v5/market/instruments-info",
                &[("category", "spot")],
                false,
            )
            .await?;

        if response.ret_code != 0 {
            return Err(ExchangeError::ApiError {
                code: response.ret_code,
                message: response.ret_msg,
            });
        }

        let bybit_markets = response.result.list;
        let mut markets = Vec::new();

        for bybit_market in bybit_markets {
            if let Ok(market) = convert_bybit_market(&bybit_market) {
                markets.push(market);
            }
        }

        Ok(markets)
    }

    /// Subscribe to market data via WebSocket
    async fn subscribe_market_data(
        &self,
        _symbols: Vec<String>,
        _subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // WebSocket implementation not yet ready
        Err(ExchangeError::Other(
            "WebSocket market data subscription not implemented yet".to_string(),
        ))
    }

    /// Get WebSocket endpoint URL for market data
    fn get_websocket_url(&self) -> String {
        "wss://stream.bybit.com/v5/public/spot".to_string()
    }

    /// Get historical k-lines/candlestick data
    async fn get_klines(
        &self,
        symbol: String,
        interval: KlineInterval,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<Kline>, ExchangeError> {
        let interval_str = kline_interval_to_bybit_string(interval);
        let limit_str = limit.unwrap_or(200).to_string();

        let mut params = vec![
            ("category", "spot"),
            ("symbol", &symbol),
            ("interval", interval_str),
            ("limit", &limit_str),
        ];

        let start_time_str;
        let end_time_str;

        if let Some(start) = start_time {
            start_time_str = start.to_string();
            params.push(("start", &start_time_str));
        }

        if let Some(end) = end_time {
            end_time_str = end.to_string();
            params.push(("end", &end_time_str));
        }

        let response: BybitApiResponse<BybitKlineResult> = self
            .rest
            .get_json("/v5/market/kline", &params, false)
            .await?;

        if response.ret_code != 0 {
            return Err(ExchangeError::ApiError {
                code: response.ret_code,
                message: response.ret_msg,
            });
        }

        let bybit_klines = response.result.list;
        let mut klines = Vec::new();

        for bybit_kline in bybit_klines {
            if bybit_kline.len() >= 6 {
                let kline_data = crate::exchanges::bybit::types::BybitKlineData {
                    start_time: bybit_kline[0].parse::<i64>().unwrap_or_default(),
                    end_time: bybit_kline[0].parse::<i64>().unwrap_or_default() + 60000, // Approximate end time
                    interval: interval_str.to_string(),
                    open_price: bybit_kline[1].clone(),
                    high_price: bybit_kline[2].clone(),
                    low_price: bybit_kline[3].clone(),
                    close_price: bybit_kline[4].clone(),
                    volume: bybit_kline[5].clone(),
                    turnover: if bybit_kline.len() > 6 {
                        bybit_kline[6].clone()
                    } else {
                        "0".to_string()
                    },
                };

                if let Ok(kline) = convert_bybit_kline(&kline_data, &symbol, interval_str) {
                    klines.push(kline);
                }
            }
        }

        Ok(klines)
    }
}
