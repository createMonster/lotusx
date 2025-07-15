use crate::core::{
    errors::ExchangeError,
    types::{
        Balance, Kline, KlineInterval, Market, MarketDataType, OrderSide, OrderType, Price,
        Quantity, Symbol, Ticker, TimeInForce, Trade, Volume,
    },
};
use crate::exchanges::bybit::types::{
    BybitCoinBalance, BybitKlineData, BybitMarket, BybitTicker, BybitTrade,
};
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;

/// Convert Bybit market data to unified Market type
pub fn convert_bybit_market(market: &BybitMarket) -> Result<Market, ExchangeError> {
    Ok(Market {
        symbol: Symbol::new(market.base_coin.clone(), market.quote_coin.clone())
            .unwrap_or_else(|_| Symbol::default()),
        status: market.status.clone(),
        base_precision: market.base_precision.unwrap_or(8) as i32,
        quote_precision: market.quote_precision.unwrap_or(8) as i32,
        min_qty: market
            .min_qty
            .clone()
            .and_then(|s| Quantity::from_str(&s).ok()),
        max_qty: market
            .max_qty
            .clone()
            .and_then(|s| Quantity::from_str(&s).ok()),
        min_price: market
            .min_price
            .clone()
            .and_then(|s| Price::from_str(&s).ok()),
        max_price: market
            .max_price
            .clone()
            .and_then(|s| Price::from_str(&s).ok()),
    })
}

/// Convert Bybit market to Symbol (helper function)
pub fn convert_bybit_market_to_symbol(bybit_market: &BybitMarket) -> Symbol {
    Symbol::new(
        bybit_market.base_coin.clone(),
        bybit_market.quote_coin.clone(),
    )
    .unwrap_or_else(|_| Symbol::default())
}

/// Convert Bybit ticker to unified Ticker type
pub fn convert_bybit_ticker(ticker: &BybitTicker, symbol: &str) -> Result<Ticker, ExchangeError> {
    let symbol_obj = Symbol::from_string(symbol)
        .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid symbol: {}", e)))?;

    Ok(Ticker {
        symbol: symbol_obj,
        price: Price::from_str(&ticker.last_price)
            .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid price: {}", e)))?,
        price_change: Price::from_str("0.0").unwrap(), // Default as we don't have this data
        price_change_percent: Decimal::from_str("0.0").unwrap(), // Default
        high_price: ticker
            .high_price_24h
            .as_ref()
            .and_then(|s| Price::from_str(s).ok())
            .unwrap_or_else(|| Price::from_str("0.0").unwrap()),
        low_price: ticker
            .low_price_24h
            .as_ref()
            .and_then(|s| Price::from_str(s).ok())
            .unwrap_or_else(|| Price::from_str("0.0").unwrap()),
        volume: ticker
            .volume_24h
            .as_ref()
            .and_then(|s| Volume::from_str(s).ok())
            .unwrap_or_else(|| Volume::from_str("0.0").unwrap()),
        quote_volume: Volume::from_str("0.0").unwrap(), // Default
        open_time: ticker.time.unwrap_or(0),
        close_time: ticker.time.unwrap_or(0),
        count: 0, // Default
    })
}

/// Convert Bybit balance to unified Balance type
pub fn convert_bybit_balance(balance: &BybitCoinBalance) -> Result<Balance, ExchangeError> {
    Ok(Balance {
        asset: balance.coin.clone(),
        free: Quantity::from_str(&balance.wallet_balance)
            .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid balance: {}", e)))?,
        locked: Quantity::from_str(&balance.locked).map_err(|e| {
            ExchangeError::InvalidParameters(format!("Invalid locked balance: {}", e))
        })?,
    })
}

/// Convert Bybit kline data to unified Kline type
pub fn convert_bybit_kline(
    kline: &BybitKlineData,
    symbol: &str,
    interval: &str,
) -> Result<Kline, ExchangeError> {
    let symbol_obj = Symbol::from_string(symbol)
        .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid symbol: {}", e)))?;

    Ok(Kline {
        symbol: symbol_obj,
        open_time: kline.start_time,
        close_time: kline.end_time,
        interval: interval.to_string(),
        open_price: Price::from_str(&kline.open_price)
            .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid open price: {}", e)))?,
        high_price: Price::from_str(&kline.high_price)
            .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid high price: {}", e)))?,
        low_price: Price::from_str(&kline.low_price)
            .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid low price: {}", e)))?,
        close_price: Price::from_str(&kline.close_price)
            .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid close price: {}", e)))?,
        volume: Volume::from_str(&kline.volume)
            .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid volume: {}", e)))?,
        number_of_trades: 0, // Default as we don't have this in BybitKlineData
        final_bar: true,
    })
}

/// Convert Bybit trade to unified Trade type
pub fn convert_bybit_trade(trade: &BybitTrade, symbol: &str) -> Result<Trade, ExchangeError> {
    let symbol_obj = Symbol::from_string(symbol)
        .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid symbol: {}", e)))?;

    let trade_id = trade.id.parse::<i64>().unwrap_or(0);

    Ok(Trade {
        symbol: symbol_obj,
        id: trade_id,
        price: Price::from_str(&trade.price)
            .map_err(|e| ExchangeError::InvalidParameters(format!("Invalid trade price: {}", e)))?,
        quantity: Quantity::from_str(&trade.qty).map_err(|e| {
            ExchangeError::InvalidParameters(format!("Invalid trade quantity: {}", e))
        })?,
        time: trade.time,
        is_buyer_maker: trade.is_buyer_maker.unwrap_or(false),
    })
}

/// Convert order side to Bybit format
pub fn convert_order_side(side: &OrderSide) -> String {
    match side {
        OrderSide::Buy => "Buy".to_string(),
        OrderSide::Sell => "Sell".to_string(),
    }
}

/// Convert order type to Bybit format
pub fn convert_order_type(order_type: &OrderType) -> String {
    match order_type {
        OrderType::Market => "Market".to_string(),
        OrderType::Limit => "Limit".to_string(),
        OrderType::StopLoss => "StopMarket".to_string(),
        OrderType::StopLossLimit => "StopLimit".to_string(),
        OrderType::TakeProfit => "TakeProfit".to_string(),
        OrderType::TakeProfitLimit => "TakeProfitLimit".to_string(),
    }
}

/// Convert time in force to Bybit format
pub fn convert_time_in_force(tif: &TimeInForce) -> String {
    match tif {
        TimeInForce::GTC => "GTC".to_string(),
        TimeInForce::IOC => "IOC".to_string(),
        TimeInForce::FOK => "FOK".to_string(),
    }
}

/// Convert interval to Bybit-specific interval string
pub fn kline_interval_to_bybit_string(interval: KlineInterval) -> &'static str {
    match interval {
        KlineInterval::Minutes1 => "1",
        KlineInterval::Minutes3 => "3",
        KlineInterval::Minutes5 => "5",
        KlineInterval::Minutes15 => "15",
        KlineInterval::Minutes30 => "30",
        KlineInterval::Hours1 => "60",
        KlineInterval::Hours2 => "120",
        KlineInterval::Hours4 => "240",
        KlineInterval::Hours6 => "360",
        KlineInterval::Hours8 => "480",
        KlineInterval::Hours12 => "720",
        KlineInterval::Days1 => "D",
        KlineInterval::Days3 => "3D",
        KlineInterval::Weeks1 => "W",
        KlineInterval::Months1 => "M",
    }
}

/// Parse WebSocket message from Bybit V5 API
#[allow(clippy::too_many_lines)]
pub fn parse_websocket_message(value: Value) -> Option<MarketDataType> {
    // Handle Bybit V5 WebSocket message format
    if let Some(topic) = value.get("topic").and_then(|t| t.as_str()) {
        if let Some(data) = value.get("data") {
            // Parse ticker data
            if topic.starts_with("tickers.") {
                if let Some(ticker_data) = data.as_object() {
                    let symbol = topic.strip_prefix("tickers.").unwrap_or("").to_string();
                    let symbol_obj = Symbol::from_string(&symbol).ok()?;

                    return Some(MarketDataType::Ticker(Ticker {
                        symbol: symbol_obj,
                        price: ticker_data
                            .get("lastPrice")
                            .and_then(|p| p.as_str())
                            .and_then(|s| Price::from_str(s).ok())
                            .unwrap_or_else(|| Price::from_str("0").unwrap()),
                        price_change: ticker_data
                            .get("price24hChg")
                            .and_then(|c| c.as_str())
                            .and_then(|s| Price::from_str(s).ok())
                            .unwrap_or_else(|| Price::from_str("0").unwrap()),
                        price_change_percent: ticker_data
                            .get("price24hPcnt")
                            .and_then(|c| c.as_str())
                            .and_then(|s| Decimal::from_str(s).ok())
                            .unwrap_or_else(|| Decimal::from_str("0").unwrap()),
                        high_price: ticker_data
                            .get("highPrice24h")
                            .and_then(|h| h.as_str())
                            .and_then(|s| Price::from_str(s).ok())
                            .unwrap_or_else(|| Price::from_str("0").unwrap()),
                        low_price: ticker_data
                            .get("lowPrice24h")
                            .and_then(|l| l.as_str())
                            .and_then(|s| Price::from_str(s).ok())
                            .unwrap_or_else(|| Price::from_str("0").unwrap()),
                        volume: ticker_data
                            .get("volume24h")
                            .and_then(|v| v.as_str())
                            .and_then(|s| Volume::from_str(s).ok())
                            .unwrap_or_else(|| Volume::from_str("0").unwrap()),
                        quote_volume: ticker_data
                            .get("turnover24h")
                            .and_then(|q| q.as_str())
                            .and_then(|s| Volume::from_str(s).ok())
                            .unwrap_or_else(|| Volume::from_str("0").unwrap()),
                        open_time: 0,
                        close_time: 0,
                        count: 0,
                    }));
                }
            }

            // Parse trade data
            if topic.starts_with("publicTrade.") {
                if let Some(trades) = data.as_array() {
                    let symbol = topic.strip_prefix("publicTrade.").unwrap_or("").to_string();
                    let symbol_obj = Symbol::from_string(&symbol).ok()?;

                    for trade in trades {
                        if let Some(trade_obj) = trade.as_object() {
                            return Some(MarketDataType::Trade(Trade {
                                symbol: symbol_obj,
                                id: trade_obj
                                    .get("i")
                                    .and_then(|i| i.as_str())
                                    .and_then(|s| s.parse::<i64>().ok())
                                    .unwrap_or(0),
                                price: trade_obj
                                    .get("p")
                                    .and_then(|p| p.as_str())
                                    .and_then(|s| Price::from_str(s).ok())
                                    .unwrap_or_else(|| Price::from_str("0").unwrap()),
                                quantity: trade_obj
                                    .get("v")
                                    .and_then(|q| q.as_str())
                                    .and_then(|s| Quantity::from_str(s).ok())
                                    .unwrap_or_else(|| Quantity::from_str("0").unwrap()),
                                time: trade_obj.get("T").and_then(|t| t.as_i64()).unwrap_or(0),
                                is_buyer_maker: trade_obj.get("S").and_then(|s| s.as_str())
                                    == Some("Buy"),
                            }));
                        }
                    }
                }
            }

            // Parse kline data
            if topic.contains("kline.") {
                if let Some(klines) = data.as_array() {
                    let topic_parts: Vec<&str> = topic.split('.').collect();
                    if topic_parts.len() >= 3 {
                        let symbol = topic_parts[2].to_string();
                        let interval = topic_parts[1].to_string();
                        let symbol_obj = Symbol::from_string(&symbol).ok()?;

                        for kline in klines {
                            if let Some(kline_obj) = kline.as_object() {
                                return Some(MarketDataType::Kline(Kline {
                                    symbol: symbol_obj,
                                    open_time: kline_obj
                                        .get("start")
                                        .and_then(|t| t.as_i64())
                                        .unwrap_or(0),
                                    close_time: kline_obj
                                        .get("end")
                                        .and_then(|t| t.as_i64())
                                        .unwrap_or(0),
                                    interval,
                                    open_price: kline_obj
                                        .get("open")
                                        .and_then(|p| p.as_str())
                                        .and_then(|s| Price::from_str(s).ok())
                                        .unwrap_or_else(|| Price::from_str("0").unwrap()),
                                    high_price: kline_obj
                                        .get("high")
                                        .and_then(|p| p.as_str())
                                        .and_then(|s| Price::from_str(s).ok())
                                        .unwrap_or_else(|| Price::from_str("0").unwrap()),
                                    low_price: kline_obj
                                        .get("low")
                                        .and_then(|p| p.as_str())
                                        .and_then(|s| Price::from_str(s).ok())
                                        .unwrap_or_else(|| Price::from_str("0").unwrap()),
                                    close_price: kline_obj
                                        .get("close")
                                        .and_then(|p| p.as_str())
                                        .and_then(|s| Price::from_str(s).ok())
                                        .unwrap_or_else(|| Price::from_str("0").unwrap()),
                                    volume: kline_obj
                                        .get("volume")
                                        .and_then(|v| v.as_str())
                                        .and_then(|s| Volume::from_str(s).ok())
                                        .unwrap_or_else(|| Volume::from_str("0").unwrap()),
                                    number_of_trades: 0,
                                    final_bar: true,
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
