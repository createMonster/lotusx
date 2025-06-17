use super::client::BinancePerpConnector;
use super::types as binance_perp_types;
use crate::core::errors::{ExchangeError, ResultExt};
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position, PositionSide};
use crate::exchanges::binance::auth; // Reuse auth from spot Binance
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for BinancePerpConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/fapi/v2/balance", self.base_url);
        let timestamp = auth::get_timestamp()?;

        let params = vec![("timestamp", timestamp.to_string())];

        let signature =
            auth::sign_request(&params, self.config.secret_key(), "GET", "/fapi/v2/balance")
                .with_exchange_context(|| format!("Failed to sign balance request: url={}", url))?;

        let mut query_params = params;
        query_params.push(("signature", signature));

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .query(&query_params)
            .send()
            .await
            .with_exchange_context(|| format!("Failed to send balance request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .with_exchange_context(|| "Failed to read balance error response".to_string())?;
            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Account balance request failed: {}", error_text),
            });
        }

        let balances_response: Vec<binance_perp_types::BinancePerpBalance> = response
            .json()
            .await
            .with_exchange_context(|| "Failed to parse balance response".to_string())?;

        let balances = balances_response
            .into_iter()
            .filter_map(|balance| {
                let available: f64 = balance.available_balance.parse().unwrap_or(0.0);
                let balance_amt: f64 = balance.balance.parse().unwrap_or(0.0);

                if available > 0.0 || balance_amt > 0.0 {
                    Some(Balance {
                        asset: balance.asset,
                        free: balance.available_balance,
                        locked: balance.balance,
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let url = format!("{}/fapi/v2/positionRisk", self.base_url);
        let timestamp = auth::get_timestamp()?;

        let params = vec![("timestamp", timestamp.to_string())];

        let signature = auth::sign_request(
            &params,
            self.config.secret_key(),
            "GET",
            "/fapi/v2/positionRisk",
        )
        .with_exchange_context(|| format!("Failed to sign positions request: url={}", url))?;

        let mut query_params = params;
        query_params.push(("signature", signature));

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .query(&query_params)
            .send()
            .await
            .with_exchange_context(|| format!("Failed to send positions request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .with_exchange_context(|| "Failed to read positions error response".to_string())?;
            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Positions request failed: {}", error_text),
            });
        }

        let positions_response: Vec<binance_perp_types::BinancePerpPosition> = response
            .json()
            .await
            .with_exchange_context(|| "Failed to parse positions response".to_string())?;

        let positions = positions_response
            .into_iter()
            .filter_map(|pos| {
                let size: f64 = pos.position_amt.parse().unwrap_or(0.0);
                if size == 0.0 {
                    None
                } else {
                    let position_amt: f64 = pos.position_amt.parse().unwrap_or(0.0);
                    let position_side = if position_amt > 0.0 {
                        PositionSide::Long
                    } else {
                        PositionSide::Short
                    };

                    Some(Position {
                        symbol: pos.symbol,
                        position_side,
                        entry_price: pos.entry_price,
                        position_amount: pos.position_amt,
                        unrealized_pnl: pos.un_realized_pnl,
                        liquidation_price: Some(pos.liquidation_price),
                        leverage: pos.leverage,
                    })
                }
            })
            .collect();

        Ok(positions)
    }
}
