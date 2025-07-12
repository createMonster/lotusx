use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use crate::exchanges::hyperliquid::conversions;
use crate::exchanges::hyperliquid::rest::HyperliquidRest;
use async_trait::async_trait;
use tracing::instrument;

/// Account information implementation for Hyperliquid
pub struct Account<R: RestClient> {
    rest: HyperliquidRest<R>,
}

impl<R: RestClient> Account<R> {
    pub fn new(rest: HyperliquidRest<R>) -> Self {
        Self { rest }
    }

    pub fn can_sign(&self) -> bool {
        self.rest.can_sign()
    }

    pub fn wallet_address(&self) -> Option<&str> {
        self.rest.wallet_address()
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync> AccountInfo for Account<R> {
    /// Get account balance
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Account information requires authentication".to_string(),
            ));
        }

        let wallet_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("No wallet address available".to_string()))?;

        let user_state = self.rest.get_user_state(wallet_address).await?;
        Ok(conversions::convert_user_state_to_balances(&user_state))
    }

    /// Get account positions
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Account information requires authentication".to_string(),
            ));
        }

        let wallet_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("No wallet address available".to_string()))?;

        let user_state = self.rest.get_user_state(wallet_address).await?;
        Ok(conversions::convert_user_state_to_positions(&user_state))
    }
}

impl<R: RestClient> Account<R> {
    /// Get user fills/trade history (Hyperliquid-specific)
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    pub async fn get_user_fills(
        &self,
    ) -> Result<Vec<crate::exchanges::hyperliquid::types::UserFill>, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Account information requires authentication".to_string(),
            ));
        }

        let wallet_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("No wallet address available".to_string()))?;

        self.rest.get_user_fills(wallet_address).await
    }

    /// Get user state (Hyperliquid-specific)
    #[instrument(skip(self), fields(exchange = "hyperliquid"))]
    pub async fn get_user_state(
        &self,
    ) -> Result<crate::exchanges::hyperliquid::types::UserState, ExchangeError> {
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "Account information requires authentication".to_string(),
            ));
        }

        let wallet_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("No wallet address available".to_string()))?;

        self.rest.get_user_state(wallet_address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::kernel::rest::ReqwestRest;
    use crate::exchanges::hyperliquid::rest::HyperliquidRest;

    #[test]
    fn test_account_creation() {
        let rest_client = ReqwestRest::new(
            "https://api.hyperliquid.xyz".to_string(),
            "hyperliquid".to_string(),
            None,
        )
        .unwrap();
        let hyperliquid_rest = HyperliquidRest::new(rest_client, None, false);
        let account = Account::new(hyperliquid_rest);

        assert!(!account.can_sign());
        assert!(account.wallet_address().is_none());
    }
}
