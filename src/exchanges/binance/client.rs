use crate::core::{
    config::ExchangeConfig,
    errors::ExchangeError,
    traits::ExchangeConnector,
    types::*,
};
use super::{auth, types as binance_types};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

pub struct BinanceConnector {
    client: Client,
    config: ExchangeConfig,
    base_url: String,
}

impl BinanceConnector {
    pub fn new(config: ExchangeConfig) -> Self {
        let base_url = if config.testnet {
            "https://testnet.binance.vision".to_string()
        } else {
            config.base_url.clone().unwrap_or_else(|| "https://api.binance.com".to_string())
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
            OrderType::StopLoss => "STOP_LOSS".to_string(),
            OrderType::StopLossLimit => "STOP_LOSS_LIMIT".to_string(),
            OrderType::TakeProfit => "TAKE_PROFIT".to_string(),
            OrderType::TakeProfitLimit => "TAKE_PROFIT_LIMIT".to_string(),
        }
    }

    fn convert_time_in_force(&self, tif: &TimeInForce) -> String {
        match tif {
            TimeInForce::GTC => "GTC".to_string(),
            TimeInForce::IOC => "IOC".to_string(),
            TimeInForce::FOK => "FOK".to_string(),
        }
    }

    fn convert_binance_market(&self, binance_market: binance_types::BinanceMarket) -> Market {
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
}

#[async_trait]
impl ExchangeConnector for BinanceConnector {
    async fn get_markets(&self) -> Result<Vec<Market>, ExchangeError> {
        let url = format!("{}/api/v3/exchangeInfo", self.base_url);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Failed to get markets: {}",
                error_text
            )));
        }

        let exchange_info: binance_types::BinanceExchangeInfo = response.json().await?;
        
        let markets = exchange_info
            .symbols
            .into_iter()
            .map(|market| self.convert_binance_market(market))
            .collect();

        Ok(markets)
    }

    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let timestamp = auth::get_timestamp();
        
        // Create longer-lived bindings for converted values
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

        // Add optional parameters
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
        let signature = auth::generate_signature(&self.config.secret_key, &query_string);
        
        params.push(("signature", &signature));

        let url = format!("{}/api/v3/order", self.base_url);
        
        let response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", &self.config.api_key)
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
                _ => return Err(ExchangeError::InvalidParameters("Invalid order side".to_string())),
            },
            order_type: match binance_response.order_type.as_str() {
                "MARKET" => OrderType::Market,
                "LIMIT" => OrderType::Limit,
                "STOP_LOSS" => OrderType::StopLoss,
                "STOP_LOSS_LIMIT" => OrderType::StopLossLimit,
                "TAKE_PROFIT" => OrderType::TakeProfit,
                "TAKE_PROFIT_LIMIT" => OrderType::TakeProfitLimit,
                _ => return Err(ExchangeError::InvalidParameters("Invalid order type".to_string())),
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