use super::auth;
use super::client::BinanceConnector;
use super::types as binance_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for BinanceConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/api/v3/account", self.base_url);
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;

        let params = vec![("timestamp", timestamp.to_string())];

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            "GET",
            "/api/v3/account",
        )?;

        let mut query_params = params;
        query_params.push(("signature", signature));

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
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

        let account_info: binance_types::BinanceAccountInfo = response.json().await?;

        let balances = account_info
            .balances
            .into_iter()
            .filter(|balance| {
                let free: f64 = balance.free.parse().unwrap_or(0.0);
                let locked: f64 = balance.locked.parse().unwrap_or(0.0);
                free > 0.0 || locked > 0.0
            })
            .map(|balance| Balance {
                asset: balance.asset,
                free: balance.free,
                locked: balance.locked,
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