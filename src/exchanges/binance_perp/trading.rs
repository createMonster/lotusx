use super::client::BinancePerpConnector;
use super::converters::{convert_order_side, convert_order_type, convert_time_in_force};
use super::types as binance_perp_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse, OrderType};
use crate::exchanges::binance::auth; // Reuse auth from spot Binance
use async_trait::async_trait;

#[async_trait]
impl OrderPlacer for BinancePerpConnector {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let url = format!("{}/fapi/v1/order", self.base_url);
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;

        let mut params = vec![
            ("symbol", order.symbol.clone()),
            ("side", convert_order_side(&order.side)),
            ("type", convert_order_type(&order.order_type)),
            ("quantity", order.quantity.clone()),
            ("timestamp", timestamp.to_string()),
        ];

        // Add price for limit orders
        if matches!(order.order_type, OrderType::Limit) {
            if let Some(price) = &order.price {
                params.push(("price", price.clone()));
            }
        }

        // Add time in force for limit orders
        if matches!(order.order_type, OrderType::Limit) {
            if let Some(tif) = &order.time_in_force {
                params.push(("timeInForce", convert_time_in_force(tif)));
            } else {
                params.push(("timeInForce", "GTC".to_string()));
            }
        }

        // Add stop price for stop orders
        if let Some(stop_price) = &order.stop_price {
            params.push(("stopPrice", stop_price.clone()));
        }

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            "POST",
            "/fapi/v1/order",
        )?;
        params.push(("signature", signature));

        let response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
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

        let binance_response: binance_perp_types::BinancePerpOrderResponse = response.json().await?;

        Ok(OrderResponse {
            order_id: binance_response.order_id.to_string(),
            client_order_id: binance_response.client_order_id,
            symbol: binance_response.symbol,
            side: order.side,
            order_type: order.order_type,
            quantity: binance_response.orig_qty,
            price: Some(binance_response.price),
            status: binance_response.status,
            timestamp: binance_response.update_time.into(),
        })
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let url = format!("{}/fapi/v1/order", self.base_url);
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;

        let params = vec![
            ("symbol", symbol),
            ("orderId", order_id),
            ("timestamp", timestamp.to_string()),
        ];

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            "DELETE",
            "/fapi/v1/order",
        )?;

        let mut form_params = params;
        form_params.push(("signature", signature));

        let response = self
            .client
            .delete(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
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