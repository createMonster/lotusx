use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::core::traits::AccountInfo;
use crate::core::types::{Balance, Position};
use crate::exchanges::okx::{rest::OkxRest, types::OkxAccountInfo};
use async_trait::async_trait;

/// OKX account implementation
pub struct Account<R: RestClient> {
    rest: OkxRest<R>,
}

impl<R: RestClient + Clone> Account<R> {
    pub fn new(rest: &R) -> Self {
        Self {
            rest: OkxRest::new(rest.clone()),
        }
    }
}

#[async_trait]
impl<R: RestClient + Send + Sync> AccountInfo for Account<R> {
    async fn get_account_info(&self) -> Result<Vec<Balance>, ExchangeError> {
        // Get account balance from OKX
        let okx_account = self.rest.get_balance(None).await?;

        // Convert OKX balance details to core Balance format
        let mut balances = Vec::new();
        for okx_balance in okx_account.details {
            // Parse numeric values
            let total = okx_balance
                .eq
                .parse::<f64>()
                .map_err(|e| ExchangeError::ParseError(format!("Invalid total balance: {}", e)))?;

            let available = okx_balance.avail_bal.parse::<f64>().map_err(|e| {
                ExchangeError::ParseError(format!("Invalid available balance: {}", e))
            })?;

            let locked = okx_balance
                .frozen_bal
                .parse::<f64>()
                .map_err(|e| ExchangeError::ParseError(format!("Invalid locked balance: {}", e)))?;

            // Only include balances that have some value
            if total > 0.0 || available > 0.0 || locked > 0.0 {
                balances.push(Balance {
                    asset: okx_balance.ccy,
                    free: available,
                    locked,
                    total,
                });
            }
        }

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        // OKX spot trading doesn't have positions in the traditional sense
        // For spot trading, positions are essentially zero since we're trading actual assets
        // However, we could return positions if there are any margin positions

        // For now, return empty positions as this is spot trading
        // In the future, this could be extended to support margin trading positions
        Ok(Vec::new())
    }
}

impl<R: RestClient + Send + Sync> Account<R> {
    /// Get account information with specific currency filter
    pub async fn get_balance_for_currency(
        &self,
        currency: &str,
    ) -> Result<Option<Balance>, ExchangeError> {
        let okx_account = self.rest.get_balance(Some(currency)).await?;

        // Find the specific currency in the balance details
        for okx_balance in okx_account.details {
            if okx_balance.ccy.eq_ignore_ascii_case(currency) {
                let total = okx_balance.eq.parse::<f64>().map_err(|e| {
                    ExchangeError::ParseError(format!("Invalid total balance: {}", e))
                })?;

                let available = okx_balance.avail_bal.parse::<f64>().map_err(|e| {
                    ExchangeError::ParseError(format!("Invalid available balance: {}", e))
                })?;

                let locked = okx_balance.frozen_bal.parse::<f64>().map_err(|e| {
                    ExchangeError::ParseError(format!("Invalid locked balance: {}", e))
                })?;

                return Ok(Some(Balance {
                    asset: okx_balance.ccy,
                    free: available,
                    locked,
                    total,
                }));
            }
        }

        Ok(None)
    }

    /// Get account summary with USD value
    pub async fn get_account_summary(&self) -> Result<AccountSummary, ExchangeError> {
        let okx_account = self.rest.get_balance(None).await?;

        let total_equity_usd = okx_account
            .total_eq
            .parse::<f64>()
            .map_err(|e| ExchangeError::ParseError(format!("Invalid total equity: {}", e)))?;

        let isolated_equity_usd = okx_account
            .iso_eq
            .parse::<f64>()
            .map_err(|e| ExchangeError::ParseError(format!("Invalid isolated equity: {}", e)))?;

        let available_equity_usd = okx_account
            .adj_eq
            .parse::<f64>()
            .map_err(|e| ExchangeError::ParseError(format!("Invalid adjusted equity: {}", e)))?;

        Ok(AccountSummary {
            total_equity_usd,
            isolated_equity_usd,
            available_equity_usd,
            account_level: okx_account.acct_lv,
            position_mode: okx_account.pos_mode,
        })
    }
}

/// Account summary information
#[derive(Debug, Clone)]
pub struct AccountSummary {
    pub total_equity_usd: f64,
    pub isolated_equity_usd: f64,
    pub available_equity_usd: f64,
    pub account_level: String,
    pub position_mode: String,
}
