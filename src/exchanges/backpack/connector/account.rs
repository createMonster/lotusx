use crate::core::{
    errors::ExchangeError,
    kernel::RestClient,
    traits::AccountInfo,
    types::{Balance, Position},
};
use crate::exchanges::backpack::rest::BackpackRestClient;
use async_trait::async_trait;
use tracing::instrument;

/// Account implementation for Backpack
pub struct Account<R: RestClient> {
    rest: BackpackRestClient<R>,
}

impl<R: RestClient> Account<R> {
    /// Create a new account manager
    pub fn new(rest: &R) -> Self
    where
        R: Clone,
    {
        Self {
            rest: BackpackRestClient::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient> AccountInfo for Account<R> {
    #[instrument(skip(self), fields(exchange = "backpack"))]
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let balance_map = self.rest.get_balances().await?;

        // Convert BackpackBalanceMap to Vec<Balance>
        let balances: Vec<Balance> = balance_map
            .0
            .into_iter()
            .map(|(asset, asset_balance)| Balance {
                asset,
                free: crate::core::types::conversion::string_to_quantity(&asset_balance.available),
                locked: crate::core::types::conversion::string_to_quantity(&asset_balance.locked),
            })
            .collect();

        Ok(balances)
    }

    #[instrument(skip(self), fields(exchange = "backpack"))]
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let position_responses = self.rest.get_positions().await?;

        // Convert Vec<BackpackPositionResponse> to Vec<Position>
        let positions: Vec<Position> = position_responses
            .into_iter()
            .map(|pos_resp| Position {
                symbol: crate::core::types::conversion::string_to_symbol(&pos_resp.symbol),
                position_side: {
                    let net_qty: f64 = pos_resp.net_quantity.parse().unwrap_or(0.0);
                    if net_qty > 0.0 {
                        crate::core::types::PositionSide::Long
                    } else if net_qty < 0.0 {
                        crate::core::types::PositionSide::Short
                    } else {
                        crate::core::types::PositionSide::Both
                    }
                },
                entry_price: crate::core::types::conversion::string_to_price(&pos_resp.entry_price),
                position_amount: crate::core::types::conversion::string_to_quantity(
                    &pos_resp.net_quantity,
                ),
                unrealized_pnl: crate::core::types::conversion::string_to_decimal(
                    &pos_resp.pnl_unrealized,
                ),
                liquidation_price: Some(crate::core::types::conversion::string_to_price(
                    &pos_resp.est_liquidation_price,
                )),
                leverage: crate::core::types::conversion::string_to_decimal("1.0"), // Default leverage if not available
            })
            .collect();

        Ok(positions)
    }
}
