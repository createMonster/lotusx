use super::auth;
use super::client::BybitConnector;
use super::types as bybit_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position, conversion};
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for BybitConnector {
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

        let api_response: bybit_types::BybitApiResponse<bybit_types::BybitAccountResult> =
            serde_json::from_str(&response_text).map_err(|e| {
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
        // Bybit spot doesn't have positions like futures
        // Return empty positions as this is spot trading
        Ok(vec![])
    }
}
