use super::client::BybitPerpConnector;
use super::types as bybit_perp_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position, PositionSide, conversion};
use crate::exchanges::bybit::auth; // Reuse auth from spot Bybit
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for BybitPerpConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/v5/account/wallet-balance", self.base_url);
        let timestamp = auth::get_timestamp();

        let params = vec![
            ("accountType".to_string(), "UNIFIED".to_string()),
            ("timestamp".to_string(), timestamp.to_string()),
        ];

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            self.config.api_key(),
            "GET",
            "/v5/account/wallet-balance",
        )?;

        // Only include non-auth parameters in query
        let query_params = vec![("accountType", "UNIFIED")];

        let response = self
            .client
            .get(&url)
            .header("X-BAPI-API-KEY", self.config.api_key())
            .header("X-BAPI-TIMESTAMP", timestamp.to_string())
            .header("X-BAPI-RECV-WINDOW", "5000")
            .header("X-BAPI-SIGN", &signature)
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Account balance request failed: {}",
                error_text
            )));
        }

        let response_text = response.text().await?;

        let api_response: bybit_perp_types::BybitPerpApiResponse<
            bybit_perp_types::BybitPerpAccountResult,
        > = serde_json::from_str(&response_text).map_err(|e| {
            ExchangeError::NetworkError(format!(
                "Failed to parse Bybit response: {}. Response was: {}",
                e, response_text
            ))
        })?;

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
        let url = format!("{}/v5/position/list", self.base_url);
        let timestamp = auth::get_timestamp();

        let params = vec![
            ("category".to_string(), "linear".to_string()),
            ("settleCoin".to_string(), "USDT".to_string()),
            ("timestamp".to_string(), timestamp.to_string()),
        ];

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            self.config.api_key(),
            "GET",
            "/v5/position/list",
        )?;

        // Only include non-auth parameters in query
        let query_params = vec![("category", "linear"), ("settleCoin", "USDT")];

        let response = self
            .client
            .get(&url)
            .header("X-BAPI-API-KEY", self.config.api_key())
            .header("X-BAPI-TIMESTAMP", timestamp.to_string())
            .header("X-BAPI-RECV-WINDOW", "5000")
            .header("X-BAPI-SIGN", &signature)
            .query(&query_params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(ExchangeError::NetworkError(format!(
                "Positions request failed: {}",
                error_text
            )));
        }

        let response_text = response.text().await?;

        let api_response: bybit_perp_types::BybitPerpApiResponse<
            bybit_perp_types::BybitPerpPositionResult,
        > = serde_json::from_str(&response_text).map_err(|e| {
            ExchangeError::NetworkError(format!(
                "Failed to parse Bybit response: {}. Response was: {}",
                e, response_text
            ))
        })?;

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
                    liquidation_price: Some(conversion::string_to_price(&position.liquidation_price)),
                    leverage: conversion::string_to_decimal(&position.leverage),
                }
            })
            .collect();

        Ok(positions)
    }
}
