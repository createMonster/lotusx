use crate::core::{
    errors::{ExchangeError, ResultExt},
    traits::AccountInfo,
    types::{Balance, Position},
};
use crate::exchanges::backpack::{
    client::BackpackConnector,
    types::{BackpackBalanceMap, BackpackPositionResponse},
};
use async_trait::async_trait;

// Helper function to create headers safely
fn create_headers_safe(
    headers: std::collections::HashMap<String, String>,
) -> Result<reqwest::header::HeaderMap, ExchangeError> {
    let mut header_map = reqwest::header::HeaderMap::new();

    for (k, v) in headers {
        let header_name = reqwest::header::HeaderName::from_bytes(k.as_bytes())
            .map_err(|e| ExchangeError::Other(format!("Invalid header name '{}': {}", k, e)))?;
        let header_value = reqwest::header::HeaderValue::from_str(&v)
            .map_err(|e| ExchangeError::Other(format!("Invalid header value '{}': {}", v, e)))?;
        header_map.insert(header_name, header_value);
    }

    Ok(header_map)
}

#[async_trait]
impl AccountInfo for BackpackConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/api/v1/capital", self.base_url);

        // Create signed headers for the request - use correct instruction name
        let instruction = "balanceQuery";
        let headers = self
            .create_signed_headers(instruction, "")
            .with_exchange_context(|| format!("url={}", url))?;

        let response = self
            .client
            .get(&url)
            .headers(create_headers_safe(headers)?)
            .send()
            .await
            .with_exchange_context(|| format!("Failed to send request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error body".to_string());
            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Failed to get account balance: {} - {}", status, error_body),
            });
        }

        // Backpack API returns balances as a map of asset -> balance info
        let balance_map: BackpackBalanceMap = response
            .json()
            .await
            .with_exchange_context(|| "Failed to parse account balance response".to_string())?;

        // Convert the balance map to our Balance struct
        let balances = balance_map
            .0
            .into_iter()
            .filter(|(_, balance)| {
                // Only include balances that have some value
                balance.available.parse::<f64>().unwrap_or(0.0) > 0.0
                    || balance.locked.parse::<f64>().unwrap_or(0.0) > 0.0
                    || balance.staked.parse::<f64>().unwrap_or(0.0) > 0.0
            })
            .map(|(asset, balance)| Balance {
                asset,
                free: crate::core::types::conversion::string_to_quantity(&balance.available),
                locked: {
                    // Combine locked and staked for the locked field
                    let locked: f64 = balance.locked.parse().unwrap_or(0.0);
                    let staked: f64 = balance.staked.parse().unwrap_or(0.0);
                    crate::core::types::conversion::string_to_quantity(&(locked + staked).to_string())
                },
            })
            .collect();

        Ok(balances)
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let url = format!("{}/api/v1/position", self.base_url);

        // Create signed headers for the request - use correct instruction name
        let instruction = "positionQuery";
        let headers = self
            .create_signed_headers(instruction, "")
            .with_exchange_context(|| format!("url={}", url))?;

        let response = self
            .client
            .get(&url)
            .headers(create_headers_safe(headers)?)
            .send()
            .await
            .with_exchange_context(|| format!("Failed to send request to {}", url))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error body".to_string());
            return Err(ExchangeError::ApiError {
                code: status.as_u16() as i32,
                message: format!("Failed to get positions: {} - {}", status, error_body),
            });
        }

        // Backpack API returns positions directly as an array
        let positions: Vec<BackpackPositionResponse> = response
            .json()
            .await
            .with_exchange_context(|| "Failed to parse positions response".to_string())?;

        Ok(positions.into_iter()
            .filter(|p| p.net_quantity.parse::<f64>().unwrap_or(0.0) != 0.0) // Only include non-zero positions
            .map(|p| {
                let net_quantity: f64 = p.net_quantity.parse().unwrap_or(0.0);
                let position_side = if net_quantity > 0.0 {
                    crate::core::types::PositionSide::Long
                } else if net_quantity < 0.0 {
                    crate::core::types::PositionSide::Short
                } else {
                    crate::core::types::PositionSide::Both
                };

                Position {
                    symbol: p.symbol,
                    position_side,
                    entry_price: p.entry_price,
                    position_amount: p.net_quantity,
                    unrealized_pnl: p.pnl_unrealized,
                    liquidation_price: if p.est_liquidation_price == "0" || p.est_liquidation_price.is_empty() {
                        None
                    } else {
                        Some(p.est_liquidation_price)
                    },
                    leverage: "1".to_string(), // Default leverage, not provided by API
                }
            })
            .collect())
    }
}
