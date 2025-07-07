use super::auth;
use super::client::BinanceConnector;
use super::converters::{convert_order_side, convert_order_type, convert_time_in_force};
use super::types as binance_types;
use crate::core::errors::{ExchangeError, ResultExt};
use crate::core::traits::OrderPlacer;
use crate::core::types::{conversion, OrderRequest, OrderResponse, OrderType};
use async_trait::async_trait;

#[async_trait]
impl OrderPlacer for BinanceConnector {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let url = format!("{}/api/v3/order", self.base_url);
        let timestamp = auth::get_timestamp()?;

        let mut params = vec![
            ("symbol", order.symbol.to_string()),
            ("side", convert_order_side(&order.side)),
            ("type", convert_order_type(&order.order_type)),
            ("quantity", order.quantity.to_string()),
            ("timestamp", timestamp.to_string()),
        ];

        // Add price for limit orders
        if matches!(order.order_type, OrderType::Limit) {
            if let Some(price) = &order.price {
                params.push(("price", price.to_string()));
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
            params.push(("stopPrice", stop_price.to_string()));
        }

        let signature =
            auth::sign_request(&params, self.config.secret_key(), "POST", "/api/v3/order")
                .with_exchange_context(|| {
                    format!(
                        "Failed to sign order request: symbol={}, url={}",
                        order.symbol.to_string(),
                        url
                    )
                })?;
        params.push(("signature", signature));

        let response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .form(&params)
            .send()
            .await
            .with_exchange_context(|| {
                format!(
                    "Failed to send order request: symbol={}, url={}",
                    order.symbol.to_string(),
                    url
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.with_exchange_context(|| {
                format!(
                    "Failed to read error response for order: symbol={}",
                    order.symbol.to_string()
                )
            })?;
            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Order placement failed: {}", error_text),
            });
        }

        let binance_response: binance_types::BinanceOrderResponse =
            response.json().await.with_exchange_context(|| {
                format!(
                    "Failed to parse order response: symbol={}",
                    order.symbol.to_string()
                )
            })?;

        Ok(OrderResponse {
            order_id: binance_response.order_id.to_string(),
            client_order_id: binance_response.client_order_id,
            symbol: conversion::string_to_symbol(&binance_response.symbol),
            side: order.side,
            order_type: order.order_type,
            quantity: conversion::string_to_quantity(&binance_response.quantity),
            price: Some(conversion::string_to_price(&binance_response.price)),
            status: binance_response.status,
            timestamp: binance_response.timestamp.into(),
        })
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let url = format!("{}/api/v3/order", self.base_url);
        let timestamp = auth::get_timestamp()?;

        let params = vec![
            ("symbol", symbol.clone()),
            ("orderId", order_id.clone()),
            ("timestamp", timestamp.to_string()),
        ];

        let signature =
            auth::sign_request(&params, self.config.secret_key(), "DELETE", "/api/v3/order")
                .with_exchange_context(|| {
                    format!(
                        "Failed to sign cancel request: symbol={}, order_id={}",
                        symbol, order_id
                    )
                })?;

        let mut form_params = params;
        form_params.push(("signature", signature));

        let response = self
            .client
            .delete(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .form(&form_params)
            .send()
            .await
            .with_exchange_context(|| {
                format!(
                    "Failed to send cancel request: symbol={}, order_id={}, url={}",
                    symbol, order_id, url
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.with_exchange_context(|| {
                format!(
                    "Failed to read cancel error response: symbol={}, order_id={}",
                    symbol, order_id
                )
            })?;
            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Order cancellation failed: {}", error_text),
            });
        }

        Ok(())
    }
}
