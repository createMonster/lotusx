use super::client::BinancePerpConnector;
use super::converters::{convert_order_side, convert_order_type, convert_time_in_force};
use super::types::{self as binance_perp_types, BinancePerpError};
use crate::core::errors::ExchangeError;
use crate::core::traits::OrderPlacer;
use crate::core::types::{conversion, OrderRequest, OrderResponse, OrderType};
use crate::exchanges::binance::auth; // Reuse auth from spot Binance
use async_trait::async_trait;
use tracing::{error, instrument};

#[async_trait]
impl OrderPlacer for BinancePerpConnector {
    #[instrument(
        skip(self),
        fields(
            symbol = %order.symbol,
            side = ?order.side,
            order_type = ?order.order_type,
            quantity = %order.quantity
        )
    )]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        let url = format!("{}/fapi/v1/order", self.base_url);
        let timestamp = auth::get_timestamp().map_err(|e| {
            BinancePerpError::auth_error(
                format!("Failed to generate timestamp: {}", e),
                Some(order.symbol.to_string()),
            )
        })?;

        // Build params vector with pre-allocated capacity to avoid reallocations
        let mut params = Vec::with_capacity(8);
        let side_str = convert_order_side(&order.side);
        let type_str = convert_order_type(&order.order_type);
        let timestamp_str = timestamp.to_string();

        params.extend_from_slice(&[
            ("symbol", order.symbol.to_string()),
            ("side", side_str),
            ("type", type_str),
            ("quantity", order.quantity.to_string()),
            ("timestamp", timestamp_str),
        ]);

        // Add conditional parameters without heap allocation in most cases
        let price_str;
        if matches!(order.order_type, OrderType::Limit) {
            if let Some(ref price) = order.price {
                price_str = price.to_string();
                params.push(("price", price_str));
            }
        }

        let tif_str;
        if matches!(order.order_type, OrderType::Limit) {
            if let Some(ref tif) = order.time_in_force {
                tif_str = convert_time_in_force(tif);
                params.push(("timeInForce", tif_str));
            } else {
                params.push(("timeInForce", "GTC".to_string()));
            }
        }

        let stop_price_str;
        if let Some(ref stop_price) = order.stop_price {
            stop_price_str = stop_price.to_string();
            params.push(("stopPrice", stop_price_str));
        }

        let signature = auth::sign_request(
            &params
                .iter()
                .map(|(k, v)| (*k, v.to_string()))
                .collect::<Vec<_>>(),
            self.config.secret_key(),
            "POST",
            "/fapi/v1/order",
        )
        .map_err(|e| {
            BinancePerpError::auth_error(
                format!("Failed to sign order request: {}", e),
                Some(order.symbol.to_string()),
            )
        })?;

        let signature_str = signature;
        params.push(("signature", signature_str));

        let response = self
            .client
            .post(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                error!(
                    symbol = %order.symbol,
                    url = %url,
                    error = %e,
                    "Failed to send order request"
                );
                BinancePerpError::network_error(format!("Order request failed: {}", e))
            })?;

        self.handle_order_response(response, &order).await
    }

    #[instrument(
        skip(self),
        fields(symbol = %symbol, order_id = %order_id)
    )]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let url = format!("{}/fapi/v1/order", self.base_url);
        let timestamp = auth::get_timestamp().map_err(|e| {
            BinancePerpError::auth_error(
                format!("Failed to generate timestamp: {}", e),
                Some(symbol.clone()),
            )
        })?;

        let timestamp_str = timestamp.to_string();
        let params = vec![
            ("symbol", symbol.clone()),
            ("orderId", order_id.clone()),
            ("timestamp", timestamp_str),
        ];

        let signature = auth::sign_request(
            &params
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect::<Vec<_>>(),
            self.config.secret_key(),
            "DELETE",
            "/fapi/v1/order",
        )
        .map_err(|e| {
            BinancePerpError::auth_error(
                format!("Failed to sign cancel request: {}", e),
                Some(symbol.clone()),
            )
        })?;

        let signature_str = signature;
        let mut form_params = params;
        form_params.push(("signature", signature_str));

        let response = self
            .client
            .delete(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .form(&form_params)
            .send()
            .await
            .map_err(|e| {
                error!(
                    symbol = %symbol,
                    order_id = %order_id,
                    url = %url,
                    error = %e,
                    "Failed to send cancel request"
                );
                BinancePerpError::network_error(format!("Cancel request failed: {}", e))
            })?;

        self.handle_cancel_response(response, &symbol, &order_id)
            .await
    }
}

impl BinancePerpConnector {
    #[cold]
    #[inline(never)]
    async fn handle_order_response(
        &self,
        response: reqwest::Response,
        order: &OrderRequest,
    ) -> Result<OrderResponse, ExchangeError> {
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| {
                BinancePerpError::network_error(format!("Failed to read error response: {}", e))
            })?;

            error!(
                symbol = %order.symbol,
                status = %status,
                error_text = %error_text,
                "Order placement failed"
            );

            return Err(BinancePerpError::order_error(
                status.as_u16() as i32,
                error_text,
                order.symbol.to_string(),
            )
            .into());
        }

        let binance_response: binance_perp_types::BinancePerpOrderResponse =
            response.json().await.map_err(|e| {
                BinancePerpError::parse_error(
                    format!("Failed to parse order response: {}", e),
                    Some(order.symbol.to_string()),
                )
            })?;

        Ok(OrderResponse {
            order_id: binance_response.order_id.to_string(),
            client_order_id: binance_response.client_order_id,
            symbol: conversion::string_to_symbol(&binance_response.symbol),
            side: order.side.clone(),
            order_type: order.order_type.clone(),
            quantity: conversion::string_to_quantity(&binance_response.orig_qty),
            price: Some(conversion::string_to_price(&binance_response.price)),
            status: binance_response.status,
            timestamp: binance_response.update_time,
        })
    }

    #[cold]
    #[inline(never)]
    async fn handle_cancel_response(
        &self,
        response: reqwest::Response,
        symbol: &str,
        order_id: &str,
    ) -> Result<(), ExchangeError> {
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| {
                BinancePerpError::network_error(format!("Failed to read error response: {}", e))
            })?;

            error!(
                symbol = %symbol,
                order_id = %order_id,
                status = %status,
                error_text = %error_text,
                "Order cancellation failed"
            );

            return Err(
                BinancePerpError::order_error(status.as_u16() as i32, error_text, symbol).into(),
            );
        }

        Ok(())
    }
}
