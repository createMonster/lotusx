use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use crate::exchanges::paradex::auth::ParadexAuth;
use crate::exchanges::paradex::ParadexConnector;
use async_trait::async_trait;
use secrecy::ExposeSecret;

#[async_trait]
impl AccountInfo for ParadexConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let auth = ParadexAuth::with_private_key(self.config.secret_key.expose_secret().as_str())?;
        let token = auth.sign_jwt()?;

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.paradex.trade/v1/account")
            .bearer_auth(token)
            .send()
            .await?;

        let balances: Vec<Balance> = response.json().await?;
        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let auth = ParadexAuth::with_private_key(self.config.secret_key.expose_secret().as_str())?;
        let token = auth.sign_jwt()?;

        let client = reqwest::Client::new();
        let response = client
            .get("https://api.paradex.trade/v1/positions")
            .bearer_auth(token)
            .send()
            .await?;

        let positions: Vec<Position> = response.json().await?;
        Ok(positions)
    }
}
