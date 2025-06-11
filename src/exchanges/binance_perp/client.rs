use super::types as binance_perp_types;
use crate::core::{
    config::ExchangeConfig,
    errors::ExchangeError,
    traits::{AccountInfo, ExchangeConnector, MarketDataSource, OrderPlacer},
    types::{
        Kline, Market, MarketDataType, OrderBook, OrderBookEntry, OrderRequest, OrderResponse,
        OrderSide, OrderType, SubscriptionType, Symbol, Ticker, TimeInForce, Trade,
        WebSocketConfig,
    },
    websocket::{build_binance_stream_url, WebSocketManager},
};
use crate::exchanges::binance::auth; // Reuse auth from spot Binance
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::mpsc;

pub struct BinancePerpConnector {
    client: Client,
    config: ExchangeConfig,
    base_url: String,
}

impl BinancePerpConnector {
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://testnet.binancefuture.com".to_string()
        } else {
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://fapi.binance.com".to_string())
        };

        Self {
            client: Client::new(),
            config,
            base_url,
        }
    }

    fn convert_order_side(&self, side: &OrderSide) -> String {
        match side {
            OrderSide::Buy => "BUY".to_string(),
            OrderSide::Sell => "SELL".to_string(),
        }
    }

    fn convert_order_type(&self, order_type: &OrderType) -> String {
        match order_type {
            OrderType::Market => "MARKET".to_string(),
            OrderType::Limit => "LIMIT".to_string(),
            OrderType::StopLoss => "STOP".to_string(),
            OrderType::StopLossLimit => "STOP_MARKET".to_string(),
            OrderType::TakeProfit => "TAKE_PROFIT".to_string(),
            OrderType::TakeProfitLimit => "TAKE_PROFIT_MARKET".to_string(),
        }
    }

    fn convert_time_in_force(&self, tif: &TimeInForce) -> String {
        match tif {
            TimeInForce::GTC => "GTC".to_string(),
            TimeInForce::IOC => "IOC".to_string(),
            TimeInForce::FOK => "FOK".to_string(),
        }
    }

    fn convert_binance_perp_market(
        &self,
        binance_market: binance_perp_types::BinancePerpMarket,
    ) -> Market {
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
                        binance_perp_types::BinancePerpWebSocketTicker,
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
                        binance_perp_types::BinancePerpWebSocketOrderBook,
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
                } else if stream.contains("@aggTrade") {
                    if let Ok(trade) = serde_json::from_value::<
                        binance_perp_types::BinancePerpWebSocketTrade,
                    >(data.clone())
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
                    if let Ok(kline_data) = serde_json::from_value::<
                        binance_perp_types::BinancePerpWebSocketKline,
                    >(data.clone())
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
impl MarketDataSource for BinancePerpConnector {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/fapi/v1/exchangeInfo", self.base_url);

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Failed to get markets: {}",
                error_text
            )));
        }

        let exchange_info: binance_perp_types::BinancePerpExchangeInfo = response.json().await?;

        let markets = exchange_info
            .symbols
            .into_iter()
            .map(|market| self.convert_binance_perp_market(market))
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
                        streams.push(format!("{}@aggTrade", lower_symbol));
                    }
                    SubscriptionType::Klines { interval } => {
                        streams.push(format!("{}@kline_{}", lower_symbol, interval));
                    }
                }
            }
        }

        // Build WebSocket URL using helper function
        let base_url = if self.config.testnet {
            "wss://stream.binancefuture.com"
        } else {
            "wss://fstream.binance.com"
        };

        let ws_url = build_binance_stream_url(base_url, &streams);
        let ws_manager = WebSocketManager::new(ws_url);

        ws_manager.start_stream(Self::parse_websocket_message).await
    }

    fn get_websocket_url(&self) -> String {
        if self.config.testnet {
            "wss://stream.binancefuture.com".to_string()
        } else {
            "wss://fstream.binance.com".to_string()
        }
    }
}

#[async_trait]
impl OrderPlacer for BinancePerpConnector {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let timestamp = auth::get_timestamp();

        let side_str = self.convert_order_side(&order.side);
        let type_str = self.convert_order_type(&order.order_type);
        let timestamp_str = timestamp.to_string();

        let mut params = vec![
            ("symbol", order.symbol.as_str()),
            ("side", side_str.as_str()),
            ("type", type_str.as_str()),
            ("quantity", order.quantity.as_str()),
            ("timestamp", timestamp_str.as_str()),
        ];

        let price_str;
        if let Some(ref price) = order.price {
            price_str = price.clone();
            params.push(("price", &price_str));
        }

        let tif_str;
        if let Some(ref tif) = order.time_in_force {
            tif_str = self.convert_time_in_force(tif);
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

        let url = format!("{}/fapi/v1/order", self.base_url);

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

        let binance_response: binance_perp_types::BinancePerpOrderResponse =
            response.json().await?;

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
                "STOP" => OrderType::StopLoss,
                "STOP_MARKET" => OrderType::StopLossLimit,
                "TAKE_PROFIT" => OrderType::TakeProfit,
                "TAKE_PROFIT_MARKET" => OrderType::TakeProfitLimit,
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
            timestamp: binance_response.timestamp,
        })
    }
}

#[async_trait]
impl AccountInfo for BinancePerpConnector {
    // Account-related methods will be added here in the future
}

// Blanket implementation for ExchangeConnector
impl ExchangeConnector for BinancePerpConnector {}
