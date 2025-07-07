use super::client::HyperliquidClient;
use super::types::{InfoRequest, UserState};
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position, PositionSide, conversion};
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for HyperliquidClient {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let user_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("Wallet address not available".to_string()))?;

        let request = InfoRequest::UserState {
            user: user_address.to_string(),
        };

        let response: UserState = self.post_info_request(&request).await?;

        let balances = vec![
            Balance {
                asset: "USDC".to_string(),
                free: conversion::string_to_quantity(&response.margin_summary.account_value),
                locked: conversion::string_to_quantity(&response.margin_summary.total_margin_used),
            },
            Balance {
                asset: "USDC".to_string(),
                free: conversion::string_to_quantity(&response.withdrawable),
                locked: conversion::string_to_quantity("0"),
            },
        ];

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let user_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("Wallet address not available".to_string()))?;

        let request = InfoRequest::UserState {
            user: user_address.to_string(),
        };

        let response: UserState = self.post_info_request(&request).await?;

        let positions = response
            .asset_positions
            .into_iter()
            .map(|pos| {
                let position_side = if pos.position.szi.parse::<f64>().unwrap_or(0.0) > 0.0 {
                    PositionSide::Long
                } else {
                    PositionSide::Short
                };

                Position {
                    symbol: conversion::string_to_symbol(&pos.position.coin),
                    position_side,
                    entry_price: conversion::string_to_price(&pos.position.entry_px.unwrap_or_else(|| "0".to_string())),
                    position_amount: conversion::string_to_quantity(&pos.position.szi),
                    unrealized_pnl: conversion::string_to_decimal(&pos.position.unrealized_pnl),
                    liquidation_price: None, // Not directly available in Hyperliquid response
                    leverage: conversion::string_to_decimal(&pos.position.leverage.value.to_string()),
                }
            })
            .collect();

        Ok(positions)
    }
}
