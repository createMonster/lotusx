use crate::core::types::{
    Balance, Market, OrderResponse, OrderSide, OrderType, Position, PositionSide, Symbol,
};
use crate::exchanges::paradex::types::{
    ParadexBalance, ParadexMarket, ParadexOrder, ParadexPosition,
};

impl From<ParadexMarket> for Market {
    fn from(market: ParadexMarket) -> Self {
        use crate::core::types::conversion;

        Self {
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
}

impl From<ParadexOrder> for OrderResponse {
    fn from(order: ParadexOrder) -> Self {
        use crate::core::types::conversion;

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
        use crate::core::types::conversion;

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
        use crate::core::types::conversion;

        Self {
            asset: balance.asset,
            free: conversion::string_to_quantity(&balance.available),
            locked: conversion::string_to_quantity(&balance.locked),
        }
    }
}
