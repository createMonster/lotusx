use super::client::BybitPerpConnector;
use super::types as bybit_perp_types;
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position, PositionSide};
use crate::exchanges::bybit::auth; // Reuse auth from spot Bybit
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for BybitPerpConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/v5/account/wallet-balance?accountType=CONTRACT", self.base_url);
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

        let balances_response: Vec<bybit_perp_types::BybitPerpBalance> = response.json().await?;

        let balances = balances_response
            .into_iter()
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
        let url = format!("{}/v5/position/list?category=linear", self.base_url);
        let timestamp = auth::get_timestamp();

        let params = vec![("timestamp".to_string(), timestamp.to_string())];

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            self.config.api_key(),
            "GET",
            "/v5/position/list",
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
                "Positions request failed: {}",
                error_text
            )));
        }

        let positions_response: Vec<bybit_perp_types::BybitPerpPosition> = response.json().await?;

        let positions = positions_response
            .into_iter()
            .filter(|position| {
                let size: f64 = position.size.parse().unwrap_or(0.0);
                size != 0.0
            })
            .map(|position| {
                let position_side = match position.side.as_str() {
                    "Buy" => PositionSide::Long,
                    "Sell" => PositionSide::Short,
                    _ => PositionSide::Long,
                };

                Position {
                    symbol: position.symbol,
                    position_side,
                    entry_price: position.entry_price,
                    position_amount: position.size,
                    unrealized_pnl: position.unrealised_pnl,
                    liquidation_price: Some(position.liquidation_price),
                    leverage: position.leverage,
                }
            })
            .collect();

        Ok(positions)
    }
} 