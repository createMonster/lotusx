use super::connector::BackpackConnector;
use super::converters::{convert_balance, convert_position};
use super::types::{BackpackBalance, BackpackPosition};
use crate::core::errors::ExchangeError;
use crate::core::kernel::{RestClient, WsSession};
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use crate::exchanges::backpack::codec::BackpackCodec;
use async_trait::async_trait;
use tracing::{error, instrument};

#[async_trait]
impl<R: RestClient, W: WsSession<BackpackCodec>> AccountInfo for BackpackConnector<R, W> {
    #[instrument(skip(self), fields(exchange = "backpack"))]
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        if !self.can_authenticate() {
            return Err(ExchangeError::AuthError(
                "Missing API credentials for account access".to_string(),
            ));
        }

        let response = self.get_balances().await?;

        // Parse the response based on the expected API format
        // The Backpack API could return either a Vec<BackpackBalance> or a BackpackBalanceMap
        // We'll try to parse both formats

        // First try to parse as a Vec<BackpackBalance>
        if let Ok(balances) = serde_json::from_value::<Vec<BackpackBalance>>(response.clone()) {
            return Ok(balances.into_iter().map(convert_balance).collect());
        }

        // If that fails, try to parse as BackpackBalanceMap
        if let Ok(balance_map) =
            serde_json::from_value::<super::types::BackpackBalanceMap>(response.clone())
        {
            let balances: Vec<Balance> = balance_map
                .0
                .into_iter()
                .map(|(asset, asset_balance)| Balance {
                    asset,
                    free: crate::core::types::conversion::string_to_quantity(
                        &asset_balance.available,
                    ),
                    locked: crate::core::types::conversion::string_to_quantity(
                        &asset_balance.locked,
                    ),
                })
                .collect();
            return Ok(balances);
        }

        // If neither format works, log the error and return empty vec
        error!(
            response = ?response,
            "Failed to parse Backpack balance response in any known format"
        );

        Err(ExchangeError::Other(
            "Failed to parse Backpack balance response: unknown format".to_string(),
        ))
    }

    #[instrument(skip(self), fields(exchange = "backpack"))]
    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        if !self.can_authenticate() {
            return Err(ExchangeError::AuthError(
                "Missing API credentials for position access".to_string(),
            ));
        }

        let response = self.get_positions().await?;

        // Parse the response based on the expected API format
        // The Backpack API could return either a Vec<BackpackPosition> or Vec<BackpackPositionResponse>

        // First try to parse as Vec<BackpackPosition>
        if let Ok(positions) = serde_json::from_value::<Vec<BackpackPosition>>(response.clone()) {
            return Ok(positions.into_iter().map(convert_position).collect());
        }

        // If that fails, try to parse as Vec<BackpackPositionResponse>
        if let Ok(position_responses) =
            serde_json::from_value::<Vec<super::types::BackpackPositionResponse>>(response.clone())
        {
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
                    entry_price: crate::core::types::conversion::string_to_price(
                        &pos_resp.entry_price,
                    ),
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
            return Ok(positions);
        }

        // If neither format works, log the error and return empty vec
        error!(
            response = ?response,
            "Failed to parse Backpack position response in any known format"
        );

        Err(ExchangeError::Other(
            "Failed to parse Backpack position response: unknown format".to_string(),
        ))
    }
}
