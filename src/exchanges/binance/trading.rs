use super::connector::BinanceConnector;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClient, WsSession};
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse};
use crate::exchanges::binance::codec::BinanceCodec;
use async_trait::async_trait;
use tracing::instrument;

/// OrderPlacer trait implementation for Binance
#[async_trait]
impl<R: RestClient, W: WsSession<BinanceCodec>> OrderPlacer for BinanceConnector<R, W> {
    #[instrument(skip(self), fields(exchange = "binance"))]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        // Convert OrderRequest to Binance API format
        let mut body = serde_json::json!({
            "symbol": order.symbol.as_str(),
            "side": crate::exchanges::binance::converters::convert_order_side(&order.side),
            "type": crate::exchanges::binance::converters::convert_order_type(&order.order_type),
            "quantity": order.quantity.value().to_string(),
        });

        // Add price for limit orders
        if let Some(price) = &order.price {
            body["price"] = serde_json::json!(price.value().to_string());
        }

        // Add time in force for limit orders
        if let Some(tif) = &order.time_in_force {
            body["timeInForce"] = serde_json::json!(
                crate::exchanges::binance::converters::convert_time_in_force(tif)
            );
        }

        // Add stop price for stop orders
        if let Some(stop_price) = &order.stop_price {
            body["stopPrice"] = serde_json::json!(stop_price.value().to_string());
        }

        let binance_response = self.place_order(&body).await?;

        Ok(OrderResponse {
            order_id: binance_response.order_id.to_string(),
            client_order_id: binance_response.client_order_id,
            symbol: crate::core::types::conversion::string_to_symbol(&binance_response.symbol),
            side: order.side,
            order_type: order.order_type,
            quantity: crate::core::types::conversion::string_to_quantity(
                &binance_response.quantity,
            ),
            price: Some(crate::core::types::conversion::string_to_price(
                &binance_response.price,
            )),
            status: binance_response.status,
            timestamp: binance_response.timestamp.into(),
        })
    }

    #[instrument(skip(self), fields(exchange = "binance", symbol = %symbol, order_id = %order_id))]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let order_id_num = order_id
            .parse::<u64>()
            .map_err(|e| ExchangeError::Other(format!("Invalid order ID format: {}", e)))?;

        self.cancel_order(&symbol, Some(order_id_num), None).await?;
        Ok(())
    }
}
