use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use crate::exchanges::paradex::rest::ParadexRestClient;
use async_trait::async_trait;
use tracing::instrument;

/// Account implementation for Paradex
pub struct Account<R: RestClient> {
    rest: ParadexRestClient<R>,
}

impl<R: RestClient> Account<R> {
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: ParadexRestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient + Clone + Send + Sync> AccountInfo for Account<R> {
    #[instrument(skip(self), fields(exchange = "paradex"))]
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let paradex_balances = self.rest.get_account_balances().await?;
        Ok(paradex_balances.into_iter().map(Into::into).collect())
    }

    #[instrument(skip(self), fields(exchange = "paradex"))]
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let paradex_positions = self.rest.get_positions().await?;
        Ok(paradex_positions.into_iter().map(Into::into).collect())
    }
}
