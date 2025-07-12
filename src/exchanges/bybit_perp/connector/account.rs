use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::AccountInfo;
use crate::core::types::{conversion, Balance, Position, PositionSide};
use crate::exchanges::bybit_perp::rest::BybitPerpRestClient;
use async_trait::async_trait;

/// Account implementation for Bybit Perpetual
pub struct Account<R: RestClient> {
    rest: BybitPerpRestClient<R>,
}

impl<R: RestClient> Account<R> {
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: BybitPerpRestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient> AccountInfo for Account<R> {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let api_response = self.rest.get_account_balance().await?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::NetworkError(format!(
                "Bybit API error ({}): {}",
                api_response.ret_code, api_response.ret_msg
            )));
        }

        let balances = api_response
            .result
            .list
            .into_iter()
            .flat_map(|account_list| account_list.coin.into_iter())
            .filter(|balance| {
                let wallet_balance: f64 = balance.wallet_balance.parse().unwrap_or(0.0);
                let equity: f64 = balance.equity.parse().unwrap_or(0.0);
                wallet_balance > 0.0 || equity > 0.0
            })
            .map(|balance| Balance {
                asset: balance.coin,
                free: conversion::string_to_quantity(&balance.equity), // Use equity as available balance (after margin)
                locked: conversion::string_to_quantity(&balance.locked),
            })
            .collect();

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let api_response = self.rest.get_positions(Some("USDT")).await?;

        if api_response.ret_code != 0 {
            return Err(ExchangeError::NetworkError(format!(
                "Bybit API error ({}): {}",
                api_response.ret_code, api_response.ret_msg
            )));
        }

        let positions = api_response
            .result
            .list
            .into_iter()
            .filter(|position| {
                let size: f64 = position.size.parse().unwrap_or(0.0);
                size != 0.0
            })
            .map(|position| {
                let position_side = match position.side.as_str() {
                    "Sell" => PositionSide::Short,
                    _ => PositionSide::Long,
                };

                Position {
                    symbol: conversion::string_to_symbol(&position.symbol),
                    position_side,
                    entry_price: conversion::string_to_price(&position.entry_price),
                    position_amount: conversion::string_to_quantity(&position.size),
                    unrealized_pnl: conversion::string_to_decimal(&position.unrealised_pnl),
                    liquidation_price: Some(conversion::string_to_price(
                        &position.liquidation_price,
                    )),
                    leverage: conversion::string_to_decimal(&position.leverage),
                }
            })
            .collect();

        Ok(positions)
    }
}
