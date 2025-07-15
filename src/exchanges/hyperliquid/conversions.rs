use super::types::OrderRequest as HyperliquidOrderRequest;
use super::types::{
    AssetInfo, Candle, LimitOrder, OrderType, TimeInForce as HLTimeInForce, UserState,
};
use crate::core::types::{
    conversion, Balance, Kline, KlineInterval, Market, OrderRequest, OrderResponse, OrderSide,
    Position, TimeInForce,
};

/// Convert core `OrderRequest` to Hyperliquid `OrderRequest`
/// This is a hot path function for trading, so it's marked inline
#[inline]
pub fn convert_order_request_to_hyperliquid(
    order: &OrderRequest,
) -> Result<super::types::OrderRequest, crate::core::errors::ExchangeError> {
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
                conversion::string_to_price("999999999")
            } else {
                conversion::string_to_price("0.000001")
            }
        }
        _ => order
            .price
            .unwrap_or_else(|| conversion::string_to_price("0")),
    };

    Ok(HyperliquidOrderRequest {
        coin: order.symbol.to_string(),
        is_buy,
        sz: order.quantity.to_string(),
        limit_px: price.to_string(),
        order_type,
        reduce_only: false,
    })
}

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
                conversion::string_to_price("999999999")
            } else {
                conversion::string_to_price("0.000001")
            }
        }
        _ => order
            .price
            .unwrap_or_else(|| conversion::string_to_price("0")),
    };

    HyperliquidOrderRequest {
        coin: order.symbol.to_string(),
        is_buy,
        sz: order.quantity.to_string(),
        limit_px: price.to_string(),
        order_type,
        reduce_only: false,
    }
}

/// Convert Hyperliquid `OrderResponse` to core `OrderResponse`
/// This is also a hot path function, so it's marked inline
#[inline]
pub fn convert_hyperliquid_order_response_to_generic(
    response: &super::types::OrderResponse,
    original_order: &OrderRequest,
) -> Result<OrderResponse, crate::core::errors::ExchangeError> {
    Ok(OrderResponse {
        order_id: "0".to_string(), // Hyperliquid uses different ID system
        client_order_id: String::new(),
        symbol: original_order.symbol.clone(),
        side: original_order.side.clone(),
        order_type: original_order.order_type.clone(),
        quantity: original_order.quantity,
        price: original_order.price,
        status: if response.status == "ok" {
            "NEW".to_string()
        } else {
            "REJECTED".to_string()
        },
        timestamp: chrono::Utc::now().timestamp_millis(),
    })
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
        quantity: original_order.quantity,
        price: original_order.price,
        status: if response.status == "ok" {
            "NEW".to_string()
        } else {
            "REJECTED".to_string()
        },
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}

/// Convert `AssetInfo` to Market
#[inline]
pub fn convert_asset_to_market(asset: AssetInfo) -> Market {
    Market {
        symbol: conversion::string_to_symbol(&asset.name),
        status: "TRADING".to_string(),
        base_precision: 6,
        quote_precision: 6,
        min_qty: Some(conversion::string_to_quantity("0.001")),
        max_qty: Some(conversion::string_to_quantity("1000000")),
        min_price: Some(conversion::string_to_price("0.000001")),
        max_price: Some(conversion::string_to_price("1000000")),
    }
}

/// Convert `UserState` to Balance vector
#[inline]
pub fn convert_user_state_to_balances(user_state: &UserState) -> Vec<Balance> {
    let balances = vec![Balance {
        asset: "USD".to_string(),
        free: conversion::string_to_quantity(&user_state.margin_summary.account_value.to_string()),
        locked: conversion::string_to_quantity("0"),
    }];

    balances
}

/// Convert `UserState` to Position vector
#[inline]
pub fn convert_user_state_to_positions(user_state: &UserState) -> Vec<Position> {
    use crate::core::types::PositionSide;

    user_state
        .asset_positions
        .iter()
        .map(|pos| Position {
            symbol: conversion::string_to_symbol(&pos.position.coin),
            position_side: if pos.position.szi.parse::<f64>().unwrap_or(0.0) > 0.0 {
                PositionSide::Long
            } else {
                PositionSide::Short
            },
            entry_price: pos.position.entry_px.as_ref().map_or_else(
                || conversion::string_to_price("0"),
                |px| conversion::string_to_price(px),
            ),
            position_amount: conversion::string_to_quantity(&pos.position.szi),
            unrealized_pnl: conversion::string_to_decimal(&pos.position.unrealized_pnl),
            liquidation_price: None, // Not available in response
            leverage: rust_decimal::Decimal::from(pos.position.leverage.value),
        })
        .collect()
}

/// Convert Candle to Kline
#[inline]
#[allow(clippy::cast_possible_wrap)]
pub fn convert_candle_to_kline(candle: &Candle, symbol: &str, interval: KlineInterval) -> Kline {
    Kline {
        symbol: conversion::string_to_symbol(symbol),
        open_time: candle.time.min(i64::MAX as u64) as i64,
        close_time: (candle.time.min(i64::MAX as u64) as i64).saturating_add(60000), // Add 1 minute (default)
        interval: format!("{:?}", interval),
        open_price: conversion::string_to_price(&candle.open),
        high_price: conversion::string_to_price(&candle.high),
        low_price: conversion::string_to_price(&candle.low),
        close_price: conversion::string_to_price(&candle.close),
        volume: conversion::string_to_volume(&candle.volume),
        number_of_trades: candle.num_trades as i64,
        final_bar: true,
    }
}

/// Convert `KlineInterval` to Hyperliquid interval string
#[inline]
pub fn convert_kline_interval_to_hyperliquid(interval: KlineInterval) -> String {
    match interval {
        // Seconds1 removed - not commonly supported
        KlineInterval::Minutes1 => "1m".to_string(),
        KlineInterval::Minutes3 => "3m".to_string(),
        KlineInterval::Minutes5 => "5m".to_string(),
        KlineInterval::Minutes15 => "15m".to_string(),
        KlineInterval::Minutes30 => "30m".to_string(),
        KlineInterval::Hours1 => "1h".to_string(),
        KlineInterval::Hours2 => "2h".to_string(),
        KlineInterval::Hours4 => "4h".to_string(),
        KlineInterval::Hours6 => "6h".to_string(),
        KlineInterval::Hours8 => "8h".to_string(),
        KlineInterval::Hours12 => "12h".to_string(),
        KlineInterval::Days1 => "1d".to_string(),
        KlineInterval::Days3 => "3d".to_string(),
        KlineInterval::Weeks1 => "1w".to_string(),
        KlineInterval::Months1 => "1M".to_string(),
    }
}
