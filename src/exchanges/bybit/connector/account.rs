use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use crate::exchanges::bybit::conversions::convert_bybit_balance;
use crate::exchanges::bybit::rest::BybitRestClient;
use crate::exchanges::bybit::types::{BybitAccountResult, BybitApiResponse};
use async_trait::async_trait;

/// Account implementation for Bybit
pub struct Account<R: RestClient> {
    rest: BybitRestClient<R>,
}

impl<R: RestClient> Account<R> {
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: BybitRestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient + Send + Sync> AccountInfo for Account<R> {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let response: BybitApiResponse<BybitAccountResult> = self
            .rest
            .get_json(
                "/v5/account/wallet-balance",
                &[("accountType", "UNIFIED")],
                true,
            )
            .await?;

        let mut balances = Vec::new();
        for account in response.result.list {
            for coin_balance in account.coin {
                let balance = convert_bybit_balance(&coin_balance)?;
                balances.push(balance);
            }
        }

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        // Bybit spot trading doesn't have positions like perpetuals do
        // Return empty vector for spot accounts
        Ok(Vec::new())
    }
}
