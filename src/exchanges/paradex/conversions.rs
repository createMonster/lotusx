use crate::core::types::{
    conversion, Balance, FundingRate, Kline, Market, OrderResponse, OrderSide, OrderType, Position,
    PositionSide, Symbol,
};
use crate::exchanges::paradex::types::{
    ParadexBalance, ParadexFundingRate, ParadexMarket, ParadexOrder, ParadexPosition,
};
use serde_json::Value;

/// Convert `ParadexMarket` to Market
pub fn convert_paradex_market(market: ParadexMarket) -> Market {
    Market {
        symbol: Symbol::new(market.base_asset.symbol, market.quote_asset.symbol)
            .unwrap_or_else(|_| conversion::string_to_symbol(&market.symbol)),
        status: market.status,
        base_precision: market.base_asset.decimals,
        quote_precision: market.quote_asset.decimals,
        min_qty: Some(conversion::string_to_quantity(&market.min_order_size)),
        max_qty: Some(conversion::string_to_quantity(&market.max_order_size)),
        min_price: Some(conversion::string_to_price(&market.min_price)),
        max_price: Some(conversion::string_to_price(&market.max_price)),
    }
}

/// Convert `ParadexFundingRate` to `FundingRate`
pub fn convert_paradex_funding_rate(rate: ParadexFundingRate) -> FundingRate {
    FundingRate {
        symbol: conversion::string_to_symbol(&rate.symbol),
        funding_rate: Some(conversion::string_to_decimal(&rate.funding_rate)),
        previous_funding_rate: None,
        next_funding_rate: None,
        funding_time: None,
        next_funding_time: Some(rate.next_funding_time),
        mark_price: Some(conversion::string_to_price(&rate.mark_price)),
        index_price: Some(conversion::string_to_price(&rate.index_price)),
        timestamp: rate.timestamp,
    }
}

/// Convert JSON kline data to Kline
pub fn convert_paradex_kline(data: &Value, symbol: &str) -> Option<Kline> {
    // Paradex kline format: [timestamp, open, high, low, close, volume]
    let array = data.as_array()?;
    if array.len() < 6 {
        return None;
    }

    let timestamp = array[0]
        .as_i64()
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    Some(Kline {
        symbol: conversion::string_to_symbol(symbol),
        open_time: timestamp,
        close_time: timestamp + 60000, // Add 1 minute as default interval
        interval: "1m".to_string(),    // Default to 1m
        open_price: array[1]
            .as_str()
            .map(conversion::string_to_price)
            .unwrap_or_default(),
        high_price: array[2]
            .as_str()
            .map(conversion::string_to_price)
            .unwrap_or_default(),
        low_price: array[3]
            .as_str()
            .map(conversion::string_to_price)
            .unwrap_or_default(),
        close_price: array[4]
            .as_str()
            .map(conversion::string_to_price)
            .unwrap_or_default(),
        volume: array[5]
            .as_str()
            .map(conversion::string_to_volume)
            .unwrap_or_default(),
        number_of_trades: 0, // Not available from this data format
        final_bar: true,     // Assume final
    })
}

impl From<ParadexMarket> for Market {
    fn from(market: ParadexMarket) -> Self {
        convert_paradex_market(market)
    }
}

impl From<ParadexOrder> for OrderResponse {
    fn from(order: ParadexOrder) -> Self {
        Self {
            order_id: order.id,
            client_order_id: order.client_id,
            symbol: conversion::string_to_symbol(&order.market),
            side: if order.side == "BUY" {
                OrderSide::Buy
            } else {
                OrderSide::Sell
            },
            order_type: match order.order_type.as_str() {
                "LIMIT" => OrderType::Limit,
                "STOP_MARKET" => OrderType::StopLoss,
                "STOP_LIMIT" => OrderType::StopLossLimit,
                "TAKE_PROFIT_MARKET" => OrderType::TakeProfit,
                "TAKE_PROFIT_LIMIT" => OrderType::TakeProfitLimit,
                _ => OrderType::Market, // Default fallback for MARKET and unknown types
            },
            quantity: conversion::string_to_quantity(&order.size),
            price: Some(conversion::string_to_price(&order.price)),
            status: order.status,
            timestamp: chrono::DateTime::parse_from_rfc3339(&order.created_at)
                .unwrap_or_else(|_| chrono::Utc::now().into())
                .timestamp_millis(),
        }
    }
}

impl From<ParadexPosition> for Position {
    fn from(position: ParadexPosition) -> Self {
        Self {
            symbol: conversion::string_to_symbol(&position.market),
            position_side: if position.side == "LONG" {
                PositionSide::Long
            } else {
                PositionSide::Short
            },
            entry_price: conversion::string_to_price(&position.average_entry_price),
            position_amount: conversion::string_to_quantity(&position.size),
            unrealized_pnl: conversion::string_to_decimal(&position.unrealized_pnl),
            liquidation_price: position
                .liquidation_price
                .map(|p| conversion::string_to_price(&p)),
            leverage: conversion::string_to_decimal(&position.leverage),
        }
    }
}

impl From<ParadexBalance> for Balance {
    fn from(balance: ParadexBalance) -> Self {
        Self {
            asset: balance.asset,
            free: conversion::string_to_quantity(&balance.available),
            locked: conversion::string_to_quantity(&balance.locked),
        }
    }
}
