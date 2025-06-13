use super::client::BybitPerpConnector;
use super::converters::{convert_order_side, convert_order_type, convert_time_in_force};
use super::types as bybit_perp_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse, OrderType};
use crate::exchanges::bybit::auth; // Reuse auth from spot Bybit
use async_trait::async_trait;

#[async_trait]
impl OrderPlacer for BybitPerpConnector {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let url = format!("{}/v2/private/order", self.base_url);
        let timestamp = auth::get_timestamp();

        let mut params = vec![
            ("symbol".to_string(), order.symbol.clone()),
            ("side".to_string(), convert_order_side(&order.side)),
            (
                "order_type".to_string(),
                convert_order_type(&order.order_type),
            ),
            ("qty".to_string(), order.quantity.clone()),
            ("timestamp".to_string(), timestamp.to_string()),
        ];

        // Add price for limit orders
        if matches!(order.order_type, OrderType::Limit) {
            if let Some(price) = &order.price {
                params.push(("price".to_string(), price.clone()));
            }
        }

        // Add time in force for limit orders
        if matches!(order.order_type, OrderType::Limit) {
            if let Some(tif) = &order.time_in_force {
                params.push(("time_in_force".to_string(), convert_time_in_force(tif)));
            } else {
                params.push(("time_in_force".to_string(), "GoodTillCancel".to_string()));
            }
        }

        // Add stop price for stop orders
        if let Some(stop_price) = &order.stop_price {
            params.push(("stop_px".to_string(), stop_price.clone()));
        }

        // Add reduce_only for futures
        params.push(("reduce_only".to_string(), "false".to_string()));

        // Add close_on_trigger for futures
        params.push(("close_on_trigger".to_string(), "false".to_string()));

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            self.config.api_key(),
            "POST",
            "/v2/private/order",
        )?;
        params.push(("sign".to_string(), signature));

        let response = self
            .client
            .post(&url)
            .header("X-BAPI-API-KEY", self.config.api_key())
            .header("X-BAPI-TIMESTAMP", timestamp.to_string())
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Order placement failed: {}",
                error_text
            )));
        }

        let bybit_response: bybit_perp_types::BybitPerpOrderResponse = response.json().await?;

        let order_id = bybit_response.order_id.clone();
        Ok(OrderResponse {
            order_id: order_id.clone(),
            client_order_id: order_id, // Bybit uses same ID
            symbol: bybit_response.symbol,
            side: order.side,
            order_type: order.order_type,
            quantity: bybit_response.qty,
            price: Some(bybit_response.price),
            status: bybit_response.status,
            timestamp: bybit_response.timestamp,
        })
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let url = format!("{}/v2/private/order", self.base_url);
        let timestamp = auth::get_timestamp();

        let params = vec![
            ("symbol".to_string(), symbol),
            ("order_id".to_string(), order_id),
            ("timestamp".to_string(), timestamp.to_string()),
        ];

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            self.config.api_key(),
            "DELETE",
            "/v2/private/order",
        )?;

        let mut form_params = params;
        form_params.push(("sign".to_string(), signature));

        let response = self
            .client
            .delete(&url)
            .header("X-BAPI-API-KEY", self.config.api_key())
            .header("X-BAPI-TIMESTAMP", timestamp.to_string())
            .form(&form_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Order cancellation failed: {}",
                error_text
            )));
        }

        Ok(())
    }
}
