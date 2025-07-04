use crate::core::types::{
    Balance, Market, OrderResponse, OrderSide, OrderType, Position, PositionSide, Symbol,
};
use crate::exchanges::paradex::types::{
    ParadexBalance, ParadexMarket, ParadexOrder, ParadexPosition,
};

impl From<ParadexMarket> for Market {
    fn from(market: ParadexMarket) -> Self {
        Self {
            symbol: Symbol {
                base: market.base_asset.symbol,
                quote: market.quote_asset.symbol,
                symbol: market.symbol,
            },
            status: market.status,
            base_precision: market.base_asset.decimals,
            quote_precision: market.quote_asset.decimals,
            min_qty: Some(market.min_order_size),
            max_qty: Some(market.max_order_size),
            min_price: Some(market.min_price),
            max_price: Some(market.max_price),
        }
    }
}

impl From<ParadexOrder> for OrderResponse {
    fn from(order: ParadexOrder) -> Self {
        Self {
            order_id: order.id,
            client_order_id: order.client_id,
            symbol: order.market,
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
            quantity: order.size,
            price: Some(order.price),
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
            symbol: position.market,
            position_side: if position.side == "LONG" {
                PositionSide::Long
            } else {
                PositionSide::Short
            },
            entry_price: position.average_entry_price,
            position_amount: position.size,
            unrealized_pnl: position.unrealized_pnl,
            liquidation_price: position.liquidation_price,
            leverage: position.leverage,
        }
    }
}

impl From<ParadexBalance> for Balance {
    fn from(balance: ParadexBalance) -> Self {
        Self {
            asset: balance.asset,
            free: balance.available,
            locked: balance.locked,
        }
    }
}
