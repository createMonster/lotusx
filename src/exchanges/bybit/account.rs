use super::auth;
use super::client::BybitConnector;
use super::types as bybit_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for BybitConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/v5/account/wallet-balance?accountType=SPOT", self.base_url);
        let timestamp = auth::get_timestamp();

        let params = vec![("timestamp".to_string(), timestamp.to_string())];

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            self.config.api_key(),
            "GET",
            "/v5/account/wallet-balance",
        )?;

        let mut query_params = params;
        query_params.push(("sign".to_string(), signature));

        let response = self
            .client
            .get(&url)
            .header("X-BAPI-API-KEY", self.config.api_key())
            .header("X-BAPI-TIMESTAMP", timestamp.to_string())
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

        let account_info: bybit_types::BybitAccountInfo = response.json().await?;

        if account_info.ret_code != 0 {
            return Err(ExchangeError::NetworkError(format!(
                "Bybit API error: {}",
                account_info.ret_msg
            )));
        }

        let balances = account_info
            .result
            .list
            .into_iter()
            .flat_map(|account_list| account_list.coin.into_iter())
            .filter(|balance| {
                let available: f64 = balance.available_balance.parse().unwrap_or(0.0);
                let wallet: f64 = balance.wallet_balance.parse().unwrap_or(0.0);
                available > 0.0 || wallet > 0.0
            })
            .map(|balance| Balance {
                asset: balance.coin,
                free: balance.available_balance,
                locked: balance.frozen_balance,
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