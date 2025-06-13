use crate::core::{
    errors::ExchangeError,
    traits::AccountInfo,
    types::{Balance, Position},
};
use crate::exchanges::backpack::{
    client::BackpackConnector,
    types::{BackpackApiResponse, BackpackBalance, BackpackPosition},
};
use async_trait::async_trait;

#[async_trait]
impl AccountInfo for BackpackConnector {
    async fn get_account_balance(&self) -> Result<Vec<Balance>, ExchangeError> {
        let url = format!("{}/api/v1/account", self.base_url);

        // Create signed headers for the request
        let instruction = "account";
        let headers = self.create_signed_headers(instruction, "")?;

        let response = self.client
            .get(&url)
            .headers(headers.into_iter().map(|(k, v)| {
                (reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                 reqwest::header::HeaderValue::from_str(&v).unwrap())
            }).collect())
            .send()
            .await
            .map_err(ExchangeError::HttpError)?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get account balance: {}", response.status()),
            });
        }

        let api_response: BackpackApiResponse<Vec<BackpackBalance>> = response
            .json()
            .await
            .map_err(|e| ExchangeError::Other(format!("Failed to parse account response: {}", e)))?;

        if !api_response.success {
            return Err(ExchangeError::ApiError {
                code: api_response.error.as_ref().map(|e| e.code).unwrap_or(-1),
                message: api_response.error.map(|e| e.msg).unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        let balances = api_response.data.ok_or_else(|| {
            ExchangeError::ApiError {
                code: -1,
                message: "No account data received".to_string(),
            }
        })?;

        Ok(balances.into_iter().map(|b| Balance {
            asset: b.asset,
            free: b.free,
            locked: b.locked,
        }).collect())
    }

    async fn get_positions(&self) -> Result<Vec<Position>, ExchangeError> {
        let url = format!("{}/api/v1/positions", self.base_url);

        // Create signed headers for the request
        let instruction = "positions";
        let headers = self.create_signed_headers(instruction, "")?;

        let response = self.client
            .get(&url)
            .headers(headers.into_iter().map(|(k, v)| {
                (reqwest::header::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                 reqwest::header::HeaderValue::from_str(&v).unwrap())
            }).collect())
            .send()
            .await
            .map_err(ExchangeError::HttpError)?;

        if !response.status().is_success() {
            return Err(ExchangeError::ApiError {
                code: response.status().as_u16() as i32,
                message: format!("Failed to get positions: {}", response.status()),
            });
        }

        let api_response: BackpackApiResponse<Vec<BackpackPosition>> = response
            .json()
            .await
            .map_err(|e| ExchangeError::Other(format!("Failed to parse positions response: {}", e)))?;

        if !api_response.success {
            return Err(ExchangeError::ApiError {
                code: api_response.error.as_ref().map(|e| e.code).unwrap_or(-1),
                message: api_response.error.map(|e| e.msg).unwrap_or_else(|| "Unknown error".to_string()),
            });
        }

        let positions = api_response.data.ok_or_else(|| {
            ExchangeError::ApiError {
                code: -1,
                message: "No positions data received".to_string(),
            }
        })?;

        Ok(positions.into_iter().map(|p| Position {
            symbol: p.symbol,
            position_side: match p.side.as_str() {
                "LONG" => crate::core::types::PositionSide::Long,
                "SHORT" => crate::core::types::PositionSide::Short,
                _ => crate::core::types::PositionSide::Both,
            },
            entry_price: p.entry_price,
            position_amount: p.size,
            unrealized_pnl: p.unrealized_pnl,
            liquidation_price: Some(p.liquidation_price),
            leverage: p.leverage,
        }).collect())
    }
} 