use crate::core::{
    errors::ExchangeError,
    kernel::RestClient,
    traits::AccountInfo,
    types::{Balance, Position},
};
use crate::exchanges::binance_perp::{
    conversions::{convert_binance_perp_balance, convert_binance_perp_position},
    rest::BinancePerpRestClient,
};
use async_trait::async_trait;
use tracing::instrument;

/// Account information implementation for Binance Perpetual
pub struct Account<R: RestClient> {
    rest: BinancePerpRestClient<R>,
}

impl<R: RestClient> Account<R> {
    /// Create a new account info source
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: BinancePerpRestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient> AccountInfo for Account<R> {
    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let account_info = self.rest.get_account_info().await?;
        let balances = account_info
            .assets
            .iter()
            .map(convert_binance_perp_balance)
            .filter(|balance| {
                balance.free.value() > rust_decimal::Decimal::ZERO
                    || balance.locked.value() > rust_decimal::Decimal::ZERO
            })
            .collect();
        Ok(balances)
    }

    #[instrument(skip(self), fields(exchange = "binance_perp"))]
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let positions = self.rest.get_positions().await?;
        let converted_positions = positions
            .iter()
            .map(convert_binance_perp_position)
            .filter(|position| position.position_amount.value() != rust_decimal::Decimal::ZERO)
            .collect();
        Ok(converted_positions)
    }
}
