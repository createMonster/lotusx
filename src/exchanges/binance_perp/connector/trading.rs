use crate::core::{
    errors::ExchangeError,
    kernel::RestClient,
    traits::OrderPlacer,
    types::{OrderRequest, OrderResponse, OrderSide, OrderType, TimeInForce},
};
use crate::exchanges::binance_perp::rest::BinancePerpRestClient;
use async_trait::async_trait;
use serde_json::json;
use tracing::instrument;

/// Trading implementation for Binance Perpetual
pub struct Trading<R: RestClient> {
    rest: BinancePerpRestClient<R>,
}

impl<R: RestClient> Trading<R> {
    /// Create a new trading engine
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: BinancePerpRestClient::new(rest.clone()),
        }
    }
}

fn order_side_to_string(side: &OrderSide) -> String {
    match side {
        OrderSide::Buy => "BUY".to_string(),
        OrderSide::Sell => "SELL".to_string(),
    }
}

fn order_type_to_string(order_type: &OrderType) -> String {
    match order_type {
        OrderType::Market => "MARKET".to_string(),
        OrderType::Limit => "LIMIT".to_string(),
        OrderType::StopLoss => "STOP_LOSS".to_string(),
        OrderType::StopLossLimit => "STOP_LOSS_LIMIT".to_string(),
        OrderType::TakeProfit => "TAKE_PROFIT".to_string(),
        OrderType::TakeProfitLimit => "TAKE_PROFIT_LIMIT".to_string(),
    }
}

fn time_in_force_to_string(tif: &TimeInForce) -> String {
    match tif {
        TimeInForce::GTC => "GTC".to_string(),
        TimeInForce::IOC => "IOC".to_string(),
        TimeInForce::FOK => "FOK".to_string(),
    }
}

fn string_to_order_side(s: &str) -> OrderSide {
    match s {
        "BUY" => OrderSide::Buy,
        "SELL" => OrderSide::Sell,
        _ => {
            tracing::warn!("Unknown order side: {}, defaulting to Buy", s);
            OrderSide::Buy
        }
    }
}

fn string_to_order_type(s: &str) -> OrderType {
    match s {
        "MARKET" => OrderType::Market,
        "LIMIT" => OrderType::Limit,
        "STOP_LOSS" => OrderType::StopLoss,
        "STOP_LOSS_LIMIT" => OrderType::StopLossLimit,
        "TAKE_PROFIT" => OrderType::TakeProfit,
        "TAKE_PROFIT_LIMIT" => OrderType::TakeProfitLimit,
        _ => {
            tracing::warn!("Unknown order type: {}, defaulting to Market", s);
            OrderType::Market
        }
    }
}

#[async_trait]
impl<R: RestClient> OrderPlacer for Trading<R> {
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        // Convert core OrderRequest to JSON for Binance API
        let mut order_json = json!({
            "symbol": order.symbol.as_str(),
            "side": order_side_to_string(&order.side),
            "type": order_type_to_string(&order.order_type),
            "quantity": order.quantity.to_string(),
        });

        // Add optional fields
        if let Some(price) = order.price {
            order_json["price"] = json!(price.to_string());
        }

        if let Some(tif) = order.time_in_force {
            order_json["timeInForce"] = json!(time_in_force_to_string(&tif));
        } else {
            order_json["timeInForce"] = json!("GTC");
        }

        if let Some(stop_price) = order.stop_price {
            order_json["stopPrice"] = json!(stop_price.to_string());
        }

        let response = self.rest.place_order(&order_json).await?;

        // Convert Binance response to core OrderResponse
        Ok(OrderResponse {
            order_id: response.order_id.to_string(),
            client_order_id: response.client_order_id,
            symbol: crate::core::types::conversion::string_to_symbol(&response.symbol),
            side: string_to_order_side(&response.side),
            order_type: string_to_order_type(&response.order_type),
            quantity: crate::core::types::conversion::string_to_quantity(&response.orig_qty),
            price: Some(crate::core::types::conversion::string_to_price(
                &response.price,
            )),
            status: response.status,
            timestamp: response.update_time,
        })
    }

    #[instrument(skip(self), fields(exchange = "binance_perp", symbol = %symbol, order_id = %order_id))]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let order_id_u64: u64 = order_id
            .parse()
            .map_err(|_| ExchangeError::Other(format!("Invalid order ID format: {}", order_id)))?;
        self.rest
            .cancel_order(&symbol, Some(order_id_u64), None)
            .await?;
        Ok(())
    }
}
