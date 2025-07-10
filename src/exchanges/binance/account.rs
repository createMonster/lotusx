use super::connector::BinanceConnector;
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClient, WsSession};
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use crate::exchanges::binance::codec::BinanceCodec;
use async_trait::async_trait;
use tracing::instrument;

/// AccountInfo trait implementation for Binance
#[async_trait]
impl<R: RestClient, W: WsSession<BinanceCodec>> AccountInfo for BinanceConnector<R, W> {
    #[instrument(skip(self), fields(exchange = "binance"))]
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let account_info = self.get_account_info().await?;

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
