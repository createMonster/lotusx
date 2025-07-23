use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse, OrderSide};
use crate::exchanges::okx::{conversions, rest::OkxRest, types::OkxOrderRequest};
use async_trait::async_trait;

/// OKX trading implementation
#[derive(Debug)]
pub struct Trading<R: RestClient> {
    rest: OkxRest<R>,
}

impl<R: RestClient + Clone> Trading<R> {
    pub fn new(rest: &R) -> Self {
        Self {
            rest: OkxRest::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient + Send + Sync> OrderPlacer for Trading<R> {
    async fn place_order(&self, order: OrderRequest) -> Result<OrderResponse, ExchangeError> {
        // Convert core order request to OKX format
        let inst_id = conversions::convert_symbol_to_okx_inst_id(&order.symbol);
        let side = conversions::convert_order_side_to_okx(order.side.clone());
        let ord_type = conversions::convert_order_type_to_okx(
            order.order_type.clone(),
            order.time_in_force.clone(),
        );

        // Build OKX order request
        let mut okx_order = OkxOrderRequest {
            inst_id,
            td_mode: "cash".to_string(), // For spot trading
            side,
            ord_type: ord_type.clone(),
            sz: order.quantity.to_string(),
            px: None,
            cl_ord_id: None,
            tag: None,
            tgt_ccy: None,
            ban_amend: None,
        };

        // Set price for limit orders
        if let Some(price) = order.price {
            if ord_type != "market" {
                okx_order.px = Some(price.to_string());
            }
        }

        // Set target currency for market orders
        if ord_type == "market" {
            okx_order.tgt_ccy = match order.side {
                OrderSide::Buy => Some("quote_ccy".to_string()),
                OrderSide::Sell => Some("base_ccy".to_string()),
            };
        }

        // Place the order
        let okx_response = self.rest.place_order(&okx_order).await?;

        // Convert response to core format
        Ok(OrderResponse {
            order_id: okx_response.ord_id,
            client_order_id: okx_response.cl_ord_id.unwrap_or_default(),
            symbol: order.symbol,
            side: order.side,
            order_type: order.order_type,
            quantity: order.quantity,
            price: order.price,
            status: if okx_response.s_code == "0" {
                "NEW".to_string()
            } else {
                "REJECTED".to_string()
            },
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    async fn cancel_order(&self, symbol: String, order_id: String) -> Result<(), ExchangeError> {
        // Cancel the order
        let _okx_response = self
            .rest
            .cancel_order(&symbol, Some(&order_id), None)
            .await?;

        // Return success if no error occurred
        Ok(())
    }
}
