use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::exchanges::okx::types::*;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// OKX REST API client implementation
pub struct OkxRest<R: RestClient> {
    rest_client: R,
}

impl<R: RestClient> OkxRest<R> {
    pub fn new(rest_client: R) -> Self {
        Self { rest_client }
    }

    /// Get system time from OKX
    pub async fn get_system_time(&self) -> Result<u64, ExchangeError> {
        let response_value = self
            .rest_client
            .get("/api/v5/public/time", &[], false)
            .await?;
        let response: OkxResponse<Vec<HashMap<String, String>>> =
            serde_json::from_value(response_value).map_err(|e| {
                ExchangeError::DeserializationError(format!("Failed to parse response: {}", e))
            })?;

        if response.code != "0" {
            return Err(ExchangeError::api_error(
                response.code.parse().unwrap_or(-1),
                response.msg.clone(),
            ));
        }

        let timestamp_str = response
            .data
            .first()
            .and_then(|item| item.get("ts"))
            .ok_or_else(|| {
                ExchangeError::InvalidResponseFormat("Missing timestamp in response".to_string())
            })?;

        timestamp_str
            .parse::<u64>()
            .map_err(|e| ExchangeError::InvalidResponseFormat(format!("Invalid timestamp: {}", e)))
    }

    /// Get trading instruments (markets)
    pub async fn get_instruments(&self, inst_type: &str) -> Result<Vec<OkxMarket>, ExchangeError> {
        let endpoint = "/api/v5/public/instruments";
        let query_params = &[("instType", inst_type)];

        let response_value = self.rest_client.get(endpoint, query_params, false).await?;
        let response: OkxResponse<Vec<OkxMarket>> = serde_json::from_value(response_value)
            .map_err(|e| {
                ExchangeError::DeserializationError(format!("Failed to parse response: {}", e))
            })?;

        if response.code != "0" {
            return Err(ExchangeError::api_error(
                response.code.parse().unwrap_or(-1),
                response.msg.clone(),
            ));
        }

        Ok(response.data)
    }

    /// Get ticker information
    pub async fn get_ticker(&self, inst_id: &str) -> Result<OkxTicker, ExchangeError> {
        let endpoint = "/api/v5/market/ticker";
        let query_params = &[("instId", inst_id)];

        let response_value = self.rest_client.get(endpoint, query_params, false).await?;
        let response: OkxResponse<Vec<OkxTicker>> = serde_json::from_value(response_value)
            .map_err(|e| {
                ExchangeError::DeserializationError(format!("Failed to parse response: {}", e))
            })?;

        if response.code != "0" {
            return Err(ExchangeError::api_error(
                response.code.parse().unwrap_or(-1),
                response.msg.clone(),
            ));
        }

        response
            .data
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::InvalidResponseFormat("No ticker data found".to_string()))
    }

    /// Get all tickers
    pub async fn get_tickers(&self, inst_type: &str) -> Result<Vec<OkxTicker>, ExchangeError> {
        let endpoint = "/api/v5/market/tickers";
        let query_params = &[("instType", inst_type)];

        let response_value = self.rest_client.get(endpoint, query_params, false).await?;
        let response: OkxResponse<Vec<OkxTicker>> = serde_json::from_value(response_value)
            .map_err(|e| {
                ExchangeError::DeserializationError(format!("Failed to parse response: {}", e))
            })?;

        if response.code != "0" {
            return Err(ExchangeError::api_error(
                response.code.parse().unwrap_or(-1),
                response.msg.clone(),
            ));
        }

        Ok(response.data)
    }

    /// Get order book
    pub async fn get_order_book(
        &self,
        inst_id: &str,
        sz: Option<u32>,
    ) -> Result<OkxOrderBook, ExchangeError> {
        let endpoint = "/api/v5/market/books";
        let query = if let Some(size) = sz {
            format!("instId={}&sz={}", inst_id, size)
        } else {
            format!("instId={}", inst_id)
        };

        let response: OkxResponse<Vec<OkxOrderBook>> =
            self.rest_client.get(endpoint, &query, &[]).await?;

        if response.code != "0" {
            return Err(ExchangeError::ApiError(format!(
                "OKX API error: {} - {}",
                response.code, response.msg
            )));
        }

        response
            .data
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::ParseError("No order book data found".to_string()))
    }

    /// Get recent trades
    pub async fn get_trades(
        &self,
        inst_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<OkxTrade>, ExchangeError> {
        let endpoint = "/api/v5/market/trades";
        let query = if let Some(lmt) = limit {
            format!("instId={}&limit={}", inst_id, lmt)
        } else {
            format!("instId={}", inst_id)
        };

        let response: OkxResponse<Vec<OkxTrade>> =
            self.rest_client.get(endpoint, &query, &[]).await?;

        if response.code != "0" {
            return Err(ExchangeError::ApiError(format!(
                "OKX API error: {} - {}",
                response.code, response.msg
            )));
        }

        Ok(response.data)
    }

    /// Get candlestick data
    pub async fn get_candlesticks(
        &self,
        inst_id: &str,
        bar: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<OkxKline>, ExchangeError> {
        let endpoint = "/api/v5/market/candles";
        let mut query = format!("instId={}", inst_id);

        if let Some(b) = bar {
            query.push_str(&format!("&bar={}", b));
        }
        if let Some(lmt) = limit {
            query.push_str(&format!("&limit={}", lmt));
        }

        let response: OkxResponse<Vec<Vec<String>>> =
            self.rest_client.get(endpoint, &query, &[]).await?;

        if response.code != "0" {
            return Err(ExchangeError::ApiError(format!(
                "OKX API error: {} - {}",
                response.code, response.msg
            )));
        }

        // Convert array format to OkxKline structs
        let klines = response
            .data
            .into_iter()
            .filter_map(|arr| {
                if arr.len() >= 8 {
                    Some(OkxKline {
                        ts: arr[0].clone(),
                        o: arr[1].clone(),
                        h: arr[2].clone(),
                        l: arr[3].clone(),
                        c: arr[4].clone(),
                        vol: arr[5].clone(),
                        vol_ccy: arr[6].clone(),
                        vol_ccy_quote: arr[7].clone(),
                        confirm: arr.get(8).cloned().unwrap_or_default(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(klines)
    }

    // Trading API endpoints (require authentication)

    /// Place a new order
    pub async fn place_order(
        &self,
        order: &OkxOrderRequest,
    ) -> Result<OkxOrderResponse, ExchangeError> {
        let endpoint = "/api/v5/trade/order";
        let body = serde_json::to_value(order)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))?;

        let response_value = self.rest_client.post(endpoint, &body, true).await?;
        let response: OkxResponse<Vec<OkxOrderResponse>> = serde_json::from_value(response_value)
            .map_err(|e| {
            ExchangeError::DeserializationError(format!("Failed to parse response: {}", e))
        })?;

        if response.code != "0" {
            return Err(ExchangeError::api_error(
                response.code.parse().unwrap_or(-1),
                response.msg.clone(),
            ));
        }

        response.data.into_iter().next().ok_or_else(|| {
            ExchangeError::InvalidResponseFormat("No order response data found".to_string())
        })
    }

    /// Cancel an order
    pub async fn cancel_order(
        &self,
        inst_id: &str,
        ord_id: Option<&str>,
        cl_ord_id: Option<&str>,
    ) -> Result<OkxOrderResponse, ExchangeError> {
        let endpoint = "/api/v5/trade/cancel-order";

        let mut cancel_req = serde_json::json!({
            "instId": inst_id
        });

        if let Some(id) = ord_id {
            cancel_req["ordId"] = serde_json::Value::String(id.to_string());
        }
        if let Some(cl_id) = cl_ord_id {
            cancel_req["clOrdId"] = serde_json::Value::String(cl_id.to_string());
        }

        let body = serde_json::to_vec(&cancel_req)
            .map_err(|e| ExchangeError::SerializationError(e.to_string()))?;

        let response: OkxResponse<Vec<OkxOrderResponse>> =
            self.rest_client.post(endpoint, "", &body).await?;

        if response.code != "0" {
            return Err(ExchangeError::ApiError(format!(
                "OKX API error: {} - {}",
                response.code, response.msg
            )));
        }

        response
            .data
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::ParseError("No cancel response data found".to_string()))
    }

    /// Get order details
    pub async fn get_order(
        &self,
        inst_id: &str,
        ord_id: Option<&str>,
        cl_ord_id: Option<&str>,
    ) -> Result<OkxOrder, ExchangeError> {
        let endpoint = "/api/v5/trade/order";
        let mut query = format!("instId={}", inst_id);

        if let Some(id) = ord_id {
            query.push_str(&format!("&ordId={}", id));
        }
        if let Some(cl_id) = cl_ord_id {
            query.push_str(&format!("&clOrdId={}", cl_id));
        }

        let response: OkxResponse<Vec<OkxOrder>> =
            self.rest_client.get(endpoint, &query, &[]).await?;

        if response.code != "0" {
            return Err(ExchangeError::ApiError(format!(
                "OKX API error: {} - {}",
                response.code, response.msg
            )));
        }

        response
            .data
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::ParseError("No order data found".to_string()))
    }

    /// Get pending orders
    pub async fn get_pending_orders(
        &self,
        inst_type: Option<&str>,
    ) -> Result<Vec<OkxOrder>, ExchangeError> {
        let endpoint = "/api/v5/trade/orders-pending";
        let query = if let Some(inst_type) = inst_type {
            format!("instType={}", inst_type)
        } else {
            String::new()
        };

        let response: OkxResponse<Vec<OkxOrder>> =
            self.rest_client.get(endpoint, &query, &[]).await?;

        if response.code != "0" {
            return Err(ExchangeError::ApiError(format!(
                "OKX API error: {} - {}",
                response.code, response.msg
            )));
        }

        Ok(response.data)
    }

    // Account API endpoints

    /// Get account balance
    pub async fn get_balance(&self, ccy: Option<&str>) -> Result<OkxAccountInfo, ExchangeError> {
        let endpoint = "/api/v5/account/balance";
        let query = if let Some(currency) = ccy {
            format!("ccy={}", currency)
        } else {
            String::new()
        };

        let response: OkxResponse<Vec<OkxAccountInfo>> =
            self.rest_client.get(endpoint, &query, &[]).await?;

        if response.code != "0" {
            return Err(ExchangeError::ApiError(format!(
                "OKX API error: {} - {}",
                response.code, response.msg
            )));
        }

        response
            .data
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::ParseError("No account data found".to_string()))
    }
}
