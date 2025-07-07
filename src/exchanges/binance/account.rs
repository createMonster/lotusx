use super::auth;
use super::client::BinanceConnector;
use super::types as binance_types;
use crate::core::errors::{ExchangeError, ResultExt};
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position, conversion};
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for BinanceConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/api/v3/account", self.base_url);
        let timestamp = auth::get_timestamp()?;

        let params = vec![("timestamp", timestamp.to_string())];

        let signature =
            auth::sign_request(&params, self.config.secret_key(), "GET", "/api/v3/account")
                .with_exchange_context(|| {
                    format!("Failed to sign account balance request: url={}", url)
                })?;

        let mut query_params = params;
        query_params.push(("signature", signature));

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .query(&query_params)
            .send()
            .await
            .with_exchange_context(|| {
                format!("Failed to send account balance request to {}", url)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .with_exchange_context(|| "Failed to read error response body".to_string())?;
            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Account balance request failed: {}", error_text),
            });
        }

        let account_info: binance_types::BinanceAccountInfo = response
            .json()
            .await
            .with_exchange_context(|| "Failed to parse account balance response".to_string())?;

        let balances = account_info
            .balances
            .into_iter()
            .filter_map(|balance| {
                // Parse balances safely without panicking
                let free: f64 = balance.free.parse().unwrap_or(0.0);
                let locked: f64 = balance.locked.parse().unwrap_or(0.0);

                if free > 0.0 || locked > 0.0 {
                    Some(Balance {
                        asset: balance.asset,
                        free: conversion::string_to_quantity(&balance.free),
                        locked: conversion::string_to_quantity(&balance.locked),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        // Binance spot doesn't have positions like futures
        // Return empty positions as this is spot trading
        Ok(vec![])
    }
}
