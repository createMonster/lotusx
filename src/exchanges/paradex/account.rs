use super::auth::ParadexAuth;
use super::client::ParadexConnector;
use super::types::{ParadexBalance, ParadexPosition};
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use async_trait::async_trait;
use secrecy::ExposeSecret;
use tracing::{error, instrument};

#[async_trait]
impl AccountInfo for ParadexConnector {
    #[instrument(skip(self), fields(exchange = "paradex"))]
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        if !self.can_trade() {
            return Err(ExchangeError::AuthError(
                "Missing API credentials for account access".to_string(),
            ));
        }

        let auth = ParadexAuth::with_private_key(self.config.secret_key.expose_secret().as_str())
            .map_err(|e| {
            error!(error = %e, "Failed to create auth");
            ExchangeError::AuthError(format!("Authentication setup failed: {}", e))
        })?;

        let token = auth.sign_jwt().map_err(|e| {
            error!(error = %e, "Failed to sign JWT");
            ExchangeError::AuthError(format!("JWT signing failed: {}", e))
        })?;

        let url = format!("{}/v1/account", self.base_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send account balance request");
                ExchangeError::NetworkError(format!("Account balance request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!(
                status = %status,
                error_text = %error_text,
                "Account balance request failed"
            );

            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Account balance request failed: {}", error_text),
            });
        }

        let balances: Vec<ParadexBalance> = response.json().await.map_err(|e| {
            error!(error = %e, "Failed to parse account balance response");
            ExchangeError::Other(format!("Failed to parse account balance response: {}", e))
        })?;

        Ok(balances.into_iter().map(Into::into).collect())
    }

    #[instrument(skip(self), fields(exchange = "paradex"))]
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        if !self.can_trade() {
            return Err(ExchangeError::AuthError(
                "Missing API credentials for position access".to_string(),
            ));
        }

        let auth = ParadexAuth::with_private_key(self.config.secret_key.expose_secret().as_str())
            .map_err(|e| {
            error!(error = %e, "Failed to create auth");
            ExchangeError::AuthError(format!("Authentication setup failed: {}", e))
        })?;

        let token = auth.sign_jwt().map_err(|e| {
            error!(error = %e, "Failed to sign JWT");
            ExchangeError::AuthError(format!("JWT signing failed: {}", e))
        })?;

        let url = format!("{}/v1/positions", self.base_url);

        let response = self
            .client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to send positions request");
                ExchangeError::NetworkError(format!("Positions request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!(
                status = %status,
                error_text = %error_text,
                "Positions request failed"
            );

            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Positions request failed: {}", error_text),
            });
        }

        let positions: Vec<ParadexPosition> = response.json().await.map_err(|e| {
            error!(error = %e, "Failed to parse positions response");
            ExchangeError::Other(format!("Failed to parse positions response: {}", e))
        })?;

        Ok(positions.into_iter().map(Into::into).collect())
    }
}
