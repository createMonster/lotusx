use super::client::BinancePerpConnector;
use super::types::{self as binance_perp_types, BinancePerpError};
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position, PositionSide};
use crate::exchanges::binance::auth; // Reuse auth from spot Binance
use async_trait::async_trait;
use tracing::{error, instrument};

#[async_trait]
impl AccountInfo for BinancePerpConnector {
    #[instrument(skip(self))]
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/fapi/v2/balance", self.base_url);
        let timestamp = auth::get_timestamp().map_err(|e| {
            BinancePerpError::auth_error(format!("Failed to generate timestamp: {}", e), None)
        })?;

        let timestamp_str = timestamp.to_string();
        let params = [("timestamp", timestamp_str.as_str())];

        let signature = auth::sign_request(
            &params
                .iter()
                .map(|(k, v)| (*k, (*v).to_string()))
                .collect::<Vec<_>>(),
            self.config.secret_key(),
            "GET",
            "/fapi/v2/balance",
        )
        .map_err(|e| {
            BinancePerpError::auth_error(format!("Failed to sign balance request: {}", e), None)
        })?;

        let signature_str = signature;
        let mut query_params = params.to_vec();
        query_params.push(("signature", signature_str.as_str()));

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .query(&query_params)
            .send()
            .await
            .map_err(|e| {
                error!(
                    url = %url,
                    error = %e,
                    "Failed to send balance request"
                );
                BinancePerpError::network_error(format!("Balance request failed: {}", e))
            })?;

        self.handle_balance_response(response).await
    }

    #[instrument(skip(self))]
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let url = format!("{}/fapi/v2/positionRisk", self.base_url);
        let timestamp = auth::get_timestamp().map_err(|e| {
            BinancePerpError::auth_error(format!("Failed to generate timestamp: {}", e), None)
        })?;

        let timestamp_str = timestamp.to_string();
        let params = [("timestamp", timestamp_str.as_str())];

        let signature = auth::sign_request(
            &params
                .iter()
                .map(|(k, v)| (*k, (*v).to_string()))
                .collect::<Vec<_>>(),
            self.config.secret_key(),
            "GET",
            "/fapi/v2/positionRisk",
        )
        .map_err(|e| {
            BinancePerpError::auth_error(format!("Failed to sign positions request: {}", e), None)
        })?;

        let signature_str = signature;
        let mut query_params = params.to_vec();
        query_params.push(("signature", signature_str.as_str()));

        let response = self
            .client
            .get(&url)
            .header("X-MBX-APIKEY", self.config.api_key())
            .query(&query_params)
            .send()
            .await
            .map_err(|e| {
                error!(
                    url = %url,
                    error = %e,
                    "Failed to send positions request"
                );
                BinancePerpError::network_error(format!("Positions request failed: {}", e))
            })?;

        self.handle_positions_response(response).await
    }
}

impl BinancePerpConnector {
    #[cold]
    #[inline(never)]
    async fn handle_balance_response(
        &self,
        response: reqwest::Response,
    ) -> Result<Vec<Balance>, ExchangeError> {
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| {
                BinancePerpError::network_error(format!("Failed to read error response: {}", e))
            })?;

            error!(
                status = %status,
                error_text = %error_text,
                "Account balance request failed"
            );

            return Err(BinancePerpError::account_error(format!(
                "Account balance request failed: {}",
                error_text
            ))
            .into());
        }

        let balances_response: Vec<binance_perp_types::BinancePerpBalance> =
            response.json().await.map_err(|e| {
                BinancePerpError::parse_error(
                    format!("Failed to parse balance response: {}", e),
                    None,
                )
            })?;

        // Use iterator chain to avoid intermediate allocations
        let balances: Vec<Balance> = balances_response
            .into_iter()
            .filter_map(|balance| {
                // Parse once and reuse to avoid multiple string parsing
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

    #[cold]
    #[inline(never)]
    async fn handle_positions_response(
        &self,
        response: reqwest::Response,
    ) -> Result<Vec<Position>, ExchangeError> {
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| {
                BinancePerpError::network_error(format!("Failed to read error response: {}", e))
            })?;

            error!(
                status = %status,
                error_text = %error_text,
                "Positions request failed"
            );

            return Err(BinancePerpError::account_error(format!(
                "Positions request failed: {}",
                error_text
            ))
            .into());
        }

        let positions_response: Vec<binance_perp_types::BinancePerpPosition> =
            response.json().await.map_err(|e| {
                BinancePerpError::parse_error(
                    format!("Failed to parse positions response: {}", e),
                    None,
                )
            })?;

        // Use iterator chain to avoid intermediate allocations
        let positions: Vec<Position> = positions_response
            .into_iter()
            .filter_map(|pos| {
                // Parse once to avoid duplicate parsing
                let position_amt: f64 = pos.position_amt.parse().unwrap_or(0.0);

                if position_amt == 0.0 {
                    None
                } else {
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
