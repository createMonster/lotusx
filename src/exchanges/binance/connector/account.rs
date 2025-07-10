use crate::core::{
    errors::ExchangeError,
    kernel::RestClient,
    traits::AccountInfo,
    types::{Balance, Position},
};
use crate::exchanges::binance::rest::BinanceRestClient;
use async_trait::async_trait;
use tracing::instrument;

/// Account implementation for Binance
pub struct Account<R: RestClient> {
    rest: BinanceRestClient<R>,
}

impl<R: RestClient> Account<R> {
    /// Create a new account manager
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: BinanceRestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient> AccountInfo for Account<R> {
    #[instrument(skip(self), fields(exchange = "binance"))]
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let account_info = self.rest.get_account_info().await?;

        let balances = account_info
            .balances
            .into_iter()
            .filter_map(|balance| {
                // Parse balances safely without panicking
                let free: f64 = balance.free.parse().unwrap_or(0.0);
                let locked: f64 = balance.locked.parse().unwrap_or(0.0);

                if free > 0.0 || locked > 0.0 {
                    Some(Balance {
                        asset: balance.asset,
                        free: crate::core::types::conversion::string_to_quantity(&balance.free),
                        locked: crate::core::types::conversion::string_to_quantity(&balance.locked),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(balances)
    }

    #[instrument(skip(self), fields(exchange = "binance"))]
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        // Binance spot doesn't have positions like futures
        // Return empty positions as this is spot trading
        Ok(vec![])
    }
}
