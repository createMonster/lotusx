use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::OrderPlacer;
use crate::core::types::{conversion, OrderRequest, OrderResponse, OrderType};
use crate::exchanges::bybit_perp::conversions::{
    convert_order_side, convert_order_type, convert_time_in_force,
};
use crate::exchanges::bybit_perp::rest::BybitPerpRestClient;
use crate::exchanges::bybit_perp::types::{
    BybitPerpError, BybitPerpOrderRequest, BybitPerpResultExt,
};
use async_trait::async_trait;
use tracing::{error, instrument};

/// Trading implementation for Bybit Perpetual
pub struct Trading<R: RestClient> {
    rest: BybitPerpRestClient<R>,
}

impl<R: RestClient> Trading<R> {
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: BybitPerpRestClient::new(rest.clone()),
        }
    }
}

/// Helper to handle API response errors for orders
#[cold]
#[inline(never)]
fn handle_order_api_error(ret_code: i32, ret_msg: String, contract: &str) -> BybitPerpError {
    error!(contract = %contract, code = ret_code, message = %ret_msg, "Order API error");
    BybitPerpError::api_error(ret_code, ret_msg)
}

/// Helper to handle order parsing errors
#[cold]
#[inline(never)]
#[allow(dead_code)]
fn handle_order_parse_error(
    err: serde_json::Error,
    response_text: &str,
    contract: &str,
) -> BybitPerpError {
    error!(contract = %contract, response = %response_text, "Failed to parse order response");
    BybitPerpError::JsonError(err)
}

#[async_trait]
impl<R: RestClient> OrderPlacer for Trading<R> {
    #[instrument(skip(self), fields(exchange = "bybit_perp", contract = %order.symbol, side = ?order.side, order_type = ?order.order_type))]
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        // Build the request body for V5 API
        let mut request_body = BybitPerpOrderRequest {
            category: "linear".to_string(), // Use linear for perpetual futures
            symbol: order.symbol.to_string(),
            side: convert_order_side(&order.side),
            order_type: convert_order_type(&order.order_type),
            qty: order.quantity.to_string(),
            price: None,
            time_in_force: None,
            stop_price: None,
        };

        // Add price for limit orders
        if matches!(order.order_type, OrderType::Limit) {
            request_body.price = order.price.as_ref().map(|p| p.to_string());
            request_body.time_in_force = Some(
                order
                    .time_in_force
                    .as_ref()
                    .map_or_else(|| "GTC".to_string(), convert_time_in_force),
            );
        }

        // Add stop price for stop orders
        if let Some(stop_price) = &order.stop_price {
            request_body.stop_price = Some(stop_price.to_string());
        }

        let api_response = self
            .rest
            .place_order(&request_body)
            .await
            .with_position_context(
                &order.symbol.to_string(),
                &format!("{:?}", order.side),
                &order.quantity.to_string(),
            )?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::Other(
                handle_order_api_error(
                    api_response.ret_code,
                    api_response.ret_msg,
                    &order.symbol.to_string(),
                )
                .to_string(),
            ));
        }

        let bybit_response = api_response.result;
        let order_id = bybit_response.order_id.clone();
        Ok(OrderResponse {
            order_id,
            client_order_id: bybit_response.client_order_id,
            symbol: conversion::string_to_symbol(&bybit_response.symbol),
            side: order.side,
            order_type: order.order_type,
            quantity: conversion::string_to_quantity(&bybit_response.qty),
            price: Some(conversion::string_to_price(&bybit_response.price)),
            status: bybit_response.status,
            timestamp: bybit_response.timestamp,
        })
    }

    #[instrument(skip(self), fields(exchange = "bybit_perp", contract = %symbol, order_id = %order_id))]
    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        let api_response = self
            .rest
            .cancel_order(&symbol, &order_id)
            .await
            .with_contract_context(&symbol)?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::Other(
                handle_order_api_error(api_response.ret_code, api_response.ret_msg, &symbol)
                    .to_string(),
            ));
        }

        Ok(())
    }
}
