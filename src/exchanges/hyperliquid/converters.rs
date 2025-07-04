use super::types::{LimitOrder, OrderType, TimeInForce as HLTimeInForce};
use crate::core::types::{OrderRequest, OrderResponse, OrderSide, TimeInForce};

/// Convert core `OrderRequest` to Hyperliquid `OrderRequest`
/// This is a hot path function for trading, so it's marked inline
#[inline]
pub fn convert_to_hyperliquid_order(order: &OrderRequest) -> super::types::OrderRequest {
    let is_buy = matches!(order.side, OrderSide::Buy);
    let order_type = match order.order_type {
        crate::core::types::OrderType::Limit => OrderType::Limit {
            limit: LimitOrder {
                tif: order
                    .time_in_force
                    .as_ref()
                    .map_or(HLTimeInForce::Gtc, |tif| match tif {
                        TimeInForce::GTC => HLTimeInForce::Gtc,
                        TimeInForce::IOC | TimeInForce::FOK => HLTimeInForce::Ioc,
                    }),
            },
        },
        crate::core::types::OrderType::Market => OrderType::Limit {
            limit: LimitOrder {
                tif: HLTimeInForce::Ioc,
            },
        },
        _ => OrderType::Limit {
            limit: LimitOrder {
                tif: HLTimeInForce::Gtc,
            },
        },
    };

    let price = match order.order_type {
        crate::core::types::OrderType::Market => {
            if is_buy {
                "999999999".to_string()
            } else {
                "0.000001".to_string()
            }
        }
        _ => order.price.clone().unwrap_or_else(|| "0".to_string()),
    };

    super::types::OrderRequest {
        coin: order.symbol.clone(),
        is_buy,
        sz: order.quantity.clone(),
        limit_px: price,
        order_type,
        reduce_only: false,
    }
}

/// Convert Hyperliquid `OrderResponse` to core `OrderResponse`
/// This is also a hot path function, so it's marked inline
#[inline]
pub fn convert_from_hyperliquid_response(
    response: &super::types::OrderResponse,
    original_order: &OrderRequest,
) -> OrderResponse {
    OrderResponse {
        order_id: "0".to_string(), // Hyperliquid uses different ID system
        client_order_id: String::new(),
        symbol: original_order.symbol.clone(),
        side: original_order.side.clone(),
        order_type: original_order.order_type.clone(),
        quantity: original_order.quantity.clone(),
        price: original_order.price.clone(),
        status: if response.status == "ok" {
            "NEW".to_string()
        } else {
            "REJECTED".to_string()
        },
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}
