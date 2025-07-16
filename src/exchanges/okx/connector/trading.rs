use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::OrderPlacer;
use crate::core::types::{OrderRequest, OrderResponse, OrderSide, OrderType, TimeInForce};
use crate::exchanges::okx::{
    conversions,
    rest::OkxRest,
    types::{OkxOrderRequest, OkxOrderResponse},
};
use async_trait::async_trait;

/// OKX trading implementation
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
        let side = conversions::convert_order_side_to_okx(order.side);
        let ord_type =
            conversions::convert_order_type_to_okx(order.order_type, order.time_in_force);

        // Build OKX order request
        let mut okx_order = OkxOrderRequest {
            inst_id,
            td_mode: "cash".to_string(), // For spot trading
            side,
            ord_type: ord_type.clone(),
            sz: order.quantity.to_string(),
            px: None,
            cl_ord_id: order.client_order_id.clone(),
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

    async fn cancel_order(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> Result<OrderResponse, ExchangeError> {
        // Cancel the order
        let okx_response = self.rest.cancel_order(symbol, Some(order_id), None).await?;

        // Get the symbol from the order details to construct response
        let symbol_obj = crate::core::types::conversion::string_to_symbol(symbol);

        Ok(OrderResponse {
            order_id: okx_response.ord_id,
            client_order_id: okx_response.cl_ord_id.unwrap_or_default(),
            symbol: symbol_obj,
            side: OrderSide::Buy, // We don't have this info from cancel response
            order_type: OrderType::Limit, // We don't have this info from cancel response
            quantity: 0.0,        // We don't have this info from cancel response
            price: None,
            status: if okx_response.s_code == "0" {
                "CANCELED".to_string()
            } else {
                "CANCEL_REJECTED".to_string()
            },
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    async fn get_order_status(
        &self,
        symbol: &str,
        order_id: &str,
    ) -> Result<OrderResponse, ExchangeError> {
        // Get order details
        let okx_order = self.rest.get_order(symbol, Some(order_id), None).await?;

        // Parse order details
        let symbol_obj = crate::core::types::conversion::string_to_symbol(&okx_order.inst_id);
        let side = match okx_order.side.as_str() {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => {
                return Err(ExchangeError::ParseError(format!(
                    "Unknown order side: {}",
                    okx_order.side
                )))
            }
        };

        let order_type = match okx_order.ord_type.as_str() {
            "market" => OrderType::Market,
            "limit" => OrderType::Limit,
            "post_only" => OrderType::LimitMaker,
            "fok" => OrderType::Limit, // FOK is a time-in-force, not order type
            "ioc" => OrderType::Limit, // IOC is a time-in-force, not order type
            _ => OrderType::Limit,
        };

        let quantity = okx_order
            .sz
            .parse::<f64>()
            .map_err(|e| ExchangeError::ParseError(format!("Invalid quantity: {}", e)))?;

        let price = if !okx_order.px.is_empty() && okx_order.px != "0" {
            Some(
                okx_order
                    .px
                    .parse::<f64>()
                    .map_err(|e| ExchangeError::ParseError(format!("Invalid price: {}", e)))?,
            )
        } else {
            None
        };

        let filled_quantity = okx_order
            .acc_fill_sz
            .parse::<f64>()
            .map_err(|e| ExchangeError::ParseError(format!("Invalid filled quantity: {}", e)))?;

        let average_price =
            if !okx_order.avg_px.is_empty() && okx_order.avg_px != "0" {
                Some(okx_order.avg_px.parse::<f64>().map_err(|e| {
                    ExchangeError::ParseError(format!("Invalid average price: {}", e))
                })?)
            } else {
                None
            };

        let timestamp = okx_order
            .c_time
            .parse::<u64>()
            .map_err(|e| ExchangeError::ParseError(format!("Invalid timestamp: {}", e)))?;

        // Calculate fees if available
        let fees = if !okx_order.fee.is_empty() && okx_order.fee != "0" {
            Some(okx_order.fee.parse::<f64>().unwrap_or(0.0).abs()) // OKX fees are negative
        } else {
            None
        };

        Ok(OrderResponse {
            order_id: okx_order.ord_id,
            client_order_id: okx_order.cl_ord_id,
            symbol: symbol_obj,
            side,
            order_type,
            quantity,
            price,
            status: conversions::convert_okx_order_state(&okx_order.state),
            filled_quantity,
            remaining_quantity: quantity - filled_quantity,
            average_price,
            fees,
            timestamp,
            exchange: "okx".to_string(),
        })
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
    ) -> Result<Vec<OrderResponse>, ExchangeError> {
        // Get pending orders from OKX
        let okx_orders = self.rest.get_pending_orders(Some("SPOT")).await?;

        let mut orders = Vec::new();
        for okx_order in okx_orders {
            // Filter by symbol if specified
            if let Some(symbol) = symbol {
                if okx_order.inst_id != symbol {
                    continue;
                }
            }

            // Convert to core order response format
            match self.convert_okx_order_to_response(okx_order).await {
                Ok(order_response) => orders.push(order_response),
                Err(e) => {
                    log::warn!("Failed to convert OKX order: {}", e);
                }
            }
        }

        Ok(orders)
    }
}

impl<R: RestClient + Send + Sync> Trading<R> {
    /// Helper method to convert OKX order to OrderResponse
    async fn convert_okx_order_to_response(
        &self,
        okx_order: crate::exchanges::okx::types::OkxOrder,
    ) -> Result<OrderResponse, ExchangeError> {
        let symbol_obj = crate::core::types::conversion::string_to_symbol(&okx_order.inst_id);
        let side = match okx_order.side.as_str() {
            "buy" => OrderSide::Buy,
            "sell" => OrderSide::Sell,
            _ => {
                return Err(ExchangeError::ParseError(format!(
                    "Unknown order side: {}",
                    okx_order.side
                )))
            }
        };

        let order_type = match okx_order.ord_type.as_str() {
            "market" => OrderType::Market,
            "limit" => OrderType::Limit,
            "post_only" => OrderType::LimitMaker,
            _ => OrderType::Limit,
        };

        let quantity = okx_order
            .sz
            .parse::<f64>()
            .map_err(|e| ExchangeError::ParseError(format!("Invalid quantity: {}", e)))?;

        let price = if !okx_order.px.is_empty() && okx_order.px != "0" {
            Some(
                okx_order
                    .px
                    .parse::<f64>()
                    .map_err(|e| ExchangeError::ParseError(format!("Invalid price: {}", e)))?,
            )
        } else {
            None
        };

        let filled_quantity = okx_order
            .acc_fill_sz
            .parse::<f64>()
            .map_err(|e| ExchangeError::ParseError(format!("Invalid filled quantity: {}", e)))?;

        let average_price =
            if !okx_order.avg_px.is_empty() && okx_order.avg_px != "0" {
                Some(okx_order.avg_px.parse::<f64>().map_err(|e| {
                    ExchangeError::ParseError(format!("Invalid average price: {}", e))
                })?)
            } else {
                None
            };

        let timestamp = okx_order.c_time.parse::<u64>().map_err(|e| {
            ExchangeError::InvalidResponseFormat(format!("Invalid timestamp: {}", e))
        })? as i64;

        let fees = if !okx_order.fee.is_empty() && okx_order.fee != "0" {
            Some(okx_order.fee.parse::<f64>().unwrap_or(0.0).abs())
        } else {
            None
        };

        Ok(OrderResponse {
            order_id: okx_order.ord_id,
            client_order_id: okx_order.cl_ord_id.unwrap_or_default(),
            symbol: symbol_obj,
            side,
            order_type,
            quantity,
            price,
            status: conversions::convert_okx_order_state(&okx_order.state),
            timestamp,
        })
    }
}
