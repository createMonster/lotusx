use super::{auth, types as binance_types};
use crate::core::{
    config::ExchangeConfig,
    errors::ExchangeError,
    traits::{AccountInfo, ExchangeConnector, MarketDataSource, OrderPlacer},
    types::{
        Balance, Kline, Market, MarketDataType, OrderBook, OrderBookEntry, OrderRequest,
        OrderResponse, OrderSide, OrderType, Position, SubscriptionType, Symbol, Ticker,
        TimeInForce, Trade, WebSocketConfig,
    },
    websocket::{build_binance_stream_url, WebSocketManager},
};
use async_trait::async_trait;
use reqwest::Client;
use secrecy::ExposeSecret;
use serde_json::Value;
use tokio::sync::mpsc;

pub struct BinanceConnector {
    client: Client,
    config: ExchangeConfig,
    base_url: String,
}

impl BinanceConnector {
    #[must_use]
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://testnet.binance.vision".to_string()
        } else {
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.binance.com".to_string())
        };

        Self {
            client: Client::new(),
            config,
            base_url,
        }
    }

    fn convert_order_side(side: &OrderSide) -> String {
        match side {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        }
    }

    fn convert_order_type(order_type: &OrderType) -> String {
        match order_type {
            OrderType::Market => "MARKET".to_string(),
            OrderType::Limit => "LIMIT".to_string(),
            OrderType::StopLoss => "STOP_LOSS".to_string(),
            OrderType::StopLossLimit => "STOP_LOSS_LIMIT".to_string(),
            OrderType::TakeProfit => "TAKE_PROFIT".to_string(),
            OrderType::TakeProfitLimit => "TAKE_PROFIT_LIMIT".to_string(),
        }
    }

    fn convert_time_in_force(tif: &TimeInForce) -> String {
        match tif {
            TimeInForce::GTC => "GTC".to_string(),
            TimeInForce::IOC => "IOC".to_string(),
            TimeInForce::FOK => "FOK".to_string(),
        }
    }

    fn convert_binance_market(binance_market: binance_types::BinanceMarket) -> Market {
        let mut min_qty = None;
        let mut max_qty = None;
        let mut min_price = None;
        let mut max_price = None;

        for filter in &binance_market.filters {
            match filter.filter_type.as_str() {
                "LOT_SIZE" => {
                    min_qty = filter.min_qty.clone();
                    max_qty = filter.max_qty.clone();
                }
                "PRICE_FILTER" => {
                    min_price = filter.min_price.clone();
                    max_price = filter.max_price.clone();
                }
                _ => {}
            }
        }

        Market {
            symbol: Symbol {
                base: binance_market.base_asset,
                quote: binance_market.quote_asset,
                symbol: binance_market.symbol,
            },
            status: binance_market.status,
            base_precision: binance_market.base_asset_precision,
            quote_precision: binance_market.quote_precision,
            min_qty,
            max_qty,
            min_price,
            max_price,
        }
    }

    fn parse_websocket_message(value: Value) -> Option<MarketDataType> {
        if let Some(stream) = value.get("stream").and_then(|s| s.as_str()) {
            if let Some(data) = value.get("data") {
                if stream.contains("@ticker") {
                    if let Ok(ticker) = serde_json::from_value::<
                        binance_types::BinanceWebSocketTicker,
                    >(data.clone())
                    {
                        return Some(MarketDataType::Ticker(Ticker {
                            symbol: ticker.symbol,
                            price: ticker.price,
                            price_change: ticker.price_change,
                            price_change_percent: ticker.price_change_percent,
                            high_price: ticker.high_price,
                            low_price: ticker.low_price,
                            volume: ticker.volume,
                            quote_volume: ticker.quote_volume,
                            open_time: ticker.open_time,
                            close_time: ticker.close_time,
                            count: ticker.count,
                        }));
                    }
                } else if stream.contains("@depth") {
                    if let Ok(depth) = serde_json::from_value::<
                        binance_types::BinanceWebSocketOrderBook,
                    >(data.clone())
                    {
                        let bids = depth
                            .bids
                            .into_iter()
                            .map(|b| OrderBookEntry {
                                price: b[0].clone(),
                                quantity: b[1].clone(),
                            })
                            .collect();
                        let asks = depth
                            .asks
                            .into_iter()
                            .map(|a| OrderBookEntry {
                                price: a[0].clone(),
                                quantity: a[1].clone(),
                            })
                            .collect();

                        return Some(MarketDataType::OrderBook(OrderBook {
                            symbol: depth.symbol,
                            bids,
                            asks,
                            last_update_id: depth.final_update_id,
                        }));
                    }
                } else if stream.contains("@trade") {
                    if let Ok(trade) =
                        serde_json::from_value::<binance_types::BinanceWebSocketTrade>(data.clone())
                    {
                        return Some(MarketDataType::Trade(Trade {
                            symbol: trade.symbol,
                            id: trade.id,
                            price: trade.price,
                            quantity: trade.quantity,
                            time: trade.time,
                            is_buyer_maker: trade.is_buyer_maker,
                        }));
                    }
                } else if stream.contains("@kline") {
                    if let Ok(kline_data) =
                        serde_json::from_value::<binance_types::BinanceWebSocketKline>(data.clone())
                    {
                        return Some(MarketDataType::Kline(Kline {
                            symbol: kline_data.symbol,
                            open_time: kline_data.kline.open_time,
                            close_time: kline_data.kline.close_time,
                            interval: kline_data.kline.interval,
                            open_price: kline_data.kline.open_price,
                            high_price: kline_data.kline.high_price,
                            low_price: kline_data.kline.low_price,
                            close_price: kline_data.kline.close_price,
                            volume: kline_data.kline.volume,
                            number_of_trades: kline_data.kline.number_of_trades,
                            final_bar: kline_data.kline.final_bar,
                        }));
                    }
                }
            }
        }
        None
    }
}

#[async_trait]
impl MarketDataSource for BinanceConnector {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/api/v3/exchangeInfo", self.base_url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Failed to get markets: {error_text}"
            )));
        }

        let exchange_info: binance_types::BinanceExchangeInfo = response.json().await?;

        let markets = exchange_info
            .symbols
            .into_iter()
            .map(Self::convert_binance_market)
            .collect();

        Ok(markets)
    }

    async fn subscribe_market_data(
        &self,
        symbols: Vec<String>,
        subscription_types: Vec<SubscriptionType>,
        _config: Option<WebSocketConfig>,
    ) -> Result<mpsc::Receiver<MarketDataType>, ExchangeError> {
        // Build streams for combined stream format
        let mut streams = Vec::new();

        for symbol in &symbols {
            let lower_symbol = symbol.to_lowercase();
            for sub_type in &subscription_types {
                match sub_type {
                    SubscriptionType::Ticker => {
                        streams.push(format!("{}@ticker", lower_symbol));
                    }
                    SubscriptionType::OrderBook { depth } => {
                        if let Some(d) = depth {
                            streams.push(format!("{}@depth{}@100ms", lower_symbol, d));
                        } else {
                            streams.push(format!("{}@depth@100ms", lower_symbol));
                        }
                    }
                    SubscriptionType::Trades => {
                        streams.push(format!("{}@trade", lower_symbol));
                    }
                    SubscriptionType::Klines { interval } => {
                        streams.push(format!("{}@kline_{}", lower_symbol, interval));
                    }
                }
            }
        }

        // Build WebSocket URL using helper function
        let base_url = if self.config.testnet {
            "wss://testnet.binance.vision"
        } else {
            "wss://stream.binance.com:443"
        };

        let ws_url = build_binance_stream_url(base_url, &streams);
        let ws_manager = WebSocketManager::new(ws_url);

        ws_manager.start_stream(Self::parse_websocket_message).await
    }

    fn get_websocket_url(&self) -> String {
        if self.config.testnet {
            "wss://testnet.binance.vision".to_string()
        } else {
            "wss://stream.binance.com:443".to_string()
        }
    }
}

#[async_trait]
impl OrderPlacer for BinanceConnector {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let timestamp = auth::get_timestamp();

        // Create longer-lived bindings for converted values
        let side_str = Self::convert_order_side(&order.side);
        let type_str = Self::convert_order_type(&order.order_type);
        let timestamp_str = timestamp.to_string();

        let mut params = vec![
            ("symbol", order.symbol.as_str()),
            ("side", side_str.as_str()),
            ("type", type_str.as_str()),
            ("quantity", order.quantity.as_str()),
            ("timestamp", timestamp_str.as_str()),
        ];

        // Add optional parameters
        let price_str;
        if let Some(ref price) = order.price {
            price_str = price.clone();
            params.push(("price", &price_str));
        }

        let tif_str;
        if let Some(ref tif) = order.time_in_force {
            tif_str = Self::convert_time_in_force(tif);
            params.push(("timeInForce", &tif_str));
        }

        let stop_price_str;
        if let Some(ref stop_price) = order.stop_price {
            stop_price_str = stop_price.clone();
            params.push(("stopPrice", &stop_price_str));
        }

        let query_string = auth::build_query_string(&params);
        let signature = auth::generate_signature(self.config.secret_key(), &query_string);

        params.push(("signature", &signature));

        let url = format!("{}/api/v3/order", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            let error_json: Result<Value, _> = serde_json::from_str(&error_text);

            if let Ok(json) = error_json {
                if let (Some(code), Some(msg)) = (json["code"].as_i64(), json["msg"].as_str()) {
                    return Err(ExchangeError::ApiError {
                        code: code as i32,
                        message: msg.to_string(),
                    });
                }
            }

            return Err(ExchangeError::NetworkError(format!(
                "Failed to place order: {}",
                error_text
            )));
        }

        let binance_response: binance_types::BinanceOrderResponse = response.json().await?;

        Ok(OrderResponse {
            order_id: binance_response.order_id.to_string(),
            client_order_id: binance_response.client_order_id,
            symbol: binance_response.symbol,
            side: match binance_response.side.as_str() {
                "BUY" => OrderSide::Buy,
                "SELL" => OrderSide::Sell,
                _ => {
                    return Err(ExchangeError::InvalidParameters(
                        "Invalid order side".to_string(),
                    ))
                }
            },
            order_type: match binance_response.order_type.as_str() {
                "MARKET" => OrderType::Market,
                "LIMIT" => OrderType::Limit,
                "STOP_LOSS" => OrderType::StopLoss,
                "STOP_LOSS_LIMIT" => OrderType::StopLossLimit,
                "TAKE_PROFIT" => OrderType::TakeProfit,
                "TAKE_PROFIT_LIMIT" => OrderType::TakeProfitLimit,
                _ => {
                    return Err(ExchangeError::InvalidParameters(
                        "Invalid order type".to_string(),
                    ))
                }
            },
            quantity: binance_response.quantity,
            price: if binance_response.price.is_empty() {
                None
            } else {
                Some(binance_response.price)
            },
            status: binance_response.status,
            timestamp: binance_response.timestamp as i64,
        })
    }
}

#[async_trait]
impl AccountInfo for BinanceConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let timestamp = auth::get_timestamp();
        let query_string = auth::build_query_string(&[("timestamp", &timestamp.to_string())]);
        let signature =
            auth::generate_signature(self.config.secret_key.expose_secret(), &query_string);

        let url = format!(
            "{}/api/v3/account?{}&signature={}",
            self.base_url, query_string, signature
        );

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", self.config.api_key.expose_secret())
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Failed to get account balance: {}",
                error_text
            )));
        }

        let account_info: binance_types::BinanceAccountInfo = response.json().await?;

        let balances = account_info
            .balances
            .into_iter()
            .map(|b| Balance {
                asset: b.asset,
                free: b.free,
                locked: b.locked,
            })
            .collect();

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        // Binance SPOT API does not have a concept of positions in the same way as futures.
        // This method would be more applicable to a futures connector.
        // For a spot connector, it's appropriate to return an empty list.
        Ok(vec![])
    }
}

// This provides ExchangeConnector automatically
impl ExchangeConnector for BinanceConnector {}
