use super::client::HyperliquidClient;
use super::types::{InfoRequest, UserState};
use crate::core::errors::ExchangeError;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position, PositionSide};
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for HyperliquidClient {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let user_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("Wallet address not available".to_string()))?;

        let request = InfoRequest::UserState {
            user: user_address.to_string(),
        };

        let response: UserState = self.post_info_request(&request).await?;

        let balances = vec![
            Balance {
                asset: "USDC".to_string(),
                free: response.margin_summary.account_value,
                locked: response.margin_summary.total_margin_used,
            },
            Balance {
                asset: "USDC".to_string(),
                free: response.withdrawable,
                locked: "0".to_string(),
            },
        ];

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let user_address = self
            .wallet_address()
            .ok_or_else(|| ExchangeError::AuthError("Wallet address not available".to_string()))?;

        let request = InfoRequest::UserState {
            user: user_address.to_string(),
        };

        let response: UserState = self.post_info_request(&request).await?;

        let positions = response
            .asset_positions
            .into_iter()
            .map(|pos| {
                let position_side = if pos.position.szi.parse::<f64>().unwrap_or(0.0) > 0.0 {
                    PositionSide::Long
                } else {
                    PositionSide::Short
                };

                Position {
                    symbol: pos.position.coin,
                    position_side,
                    entry_price: pos.position.entry_px.unwrap_or_else(|| "0".to_string()),
                    position_amount: pos.position.szi,
                    unrealized_pnl: pos.position.unrealized_pnl,
                    liquidation_price: None, // Not directly available in Hyperliquid response
                    leverage: pos.position.leverage.value.to_string(),
                }
            })
            .collect();

        Ok(positions)
    }
}
