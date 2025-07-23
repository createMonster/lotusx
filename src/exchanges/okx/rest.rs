use crate::core::errors::ExchangeError;
use crate::core::kernel::RestClient;
use crate::exchanges::okx::types::{
    OkxAccountInfo, OkxKline, OkxMarket, OkxOrder, OkxOrderBook, OkxOrderRequest, OkxOrderResponse,
    OkxResponse, OkxTicker, OkxTrade,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;

/// OKX REST API client implementation
#[derive(Debug)]
pub struct OkxRest<R: RestClient> {
    rest_client: R,
}

impl<R: RestClient> OkxRest<R> {
    pub fn new(rest_client: R) -> Self {
        Self { rest_client }
    }

    /// Maps OKX error codes to appropriate `ExchangeError` variants
    ///
    /// This function provides a comprehensive mapping of OKX error codes to
    /// more specific `ExchangeError` variants, making error handling more precise.
    fn map_okx_error(&self, code: &str, message: &str) -> ExchangeError {
        match code {
            // Authentication errors
            "50001" => ExchangeError::AuthError(format!("Invalid API key: {}", message)),
            "50002" => ExchangeError::AuthError(format!("Invalid signature: {}", message)),
            "50003" => ExchangeError::AuthError(format!("Invalid passphrase: {}", message)),
            "50004" => ExchangeError::AuthError(format!("Invalid timestamp: {}", message)),
            "50005" => ExchangeError::AuthError(format!("API key expired: {}", message)),

            // Rate limit errors
            "50006" | "50007" | "50008" => ExchangeError::RateLimitExceeded(format!(
                "OKX rate limit exceeded: {} - {}",
                code, message
            )),

            // Invalid parameter errors
            "51000" | "51001" | "51002" | "51003" | "51004" | "51005" => {
                ExchangeError::InvalidParameters(format!(
                    "Invalid parameter: {} - {}",
                    code, message
                ))
            }

            // Server errors
            "50009" | "50010" | "50011" | "50012" => {
                ExchangeError::ServerError(format!("OKX server error: {} - {}", code, message))
            }

            // Order errors
            "51006" | "51007" | "51008" => ExchangeError::ApiError {
                code: code.parse().unwrap_or(-1),
                message: format!("Order error: {} - {}", code, message),
            },
            "51009" => {
                ExchangeError::InvalidParameters(format!("Insufficient balance: {}", message))
            }
            "51010" => {
                ExchangeError::InvalidParameters(format!("Order size exceeds limit: {}", message))
            }
            "51011" => {
                ExchangeError::InvalidParameters(format!("Order price exceeds limit: {}", message))
            }

            // Market errors
            "51100" | "51101" | "51102" => ExchangeError::ApiError {
                code: code.parse().unwrap_or(-1),
                message: format!("Market error: {} - {}", code, message),
            },
            "51103" => ExchangeError::ApiError {
                code: code.parse().unwrap_or(-1),
                message: format!("Market closed: {}", message),
            },

            // Account errors
            "51200" | "51201" | "51202" => {
                ExchangeError::AuthError(format!("Account error: {} - {}", code, message))
            }

            // Default case - generic API error
            _ => ExchangeError::ApiError {
                code: code.parse().unwrap_or(-1),
                message: message.to_string(),
            },
        }
    }

    /// Generic handler for OKX API responses
    ///
    /// This function handles the common pattern of deserializing OKX responses
    /// and checking for error codes, with proper error mapping.
    fn handle_response<T>(&self, response_value: Value) -> Result<T, ExchangeError>
    where
        T: DeserializeOwned,
    {
        // Parse the response into OkxResponse structure
        let response: OkxResponse<T> = serde_json::from_value(response_value).map_err(|e| {
            ExchangeError::DeserializationError(format!("Failed to parse OKX response: {}", e))
        })?;

        // Check if the response contains an error
        if response.code != "0" {
            return Err(self.map_okx_error(&response.code, &response.msg));
        }

        Ok(response.data)
    }

    /// Generic handler for OKX API responses that return a vector where we need the first item
    ///
    /// This function handles the common pattern of deserializing OKX responses that return
    /// a vector where we're only interested in the first item.
    fn handle_single_item_response<T>(
        &self,
        response_value: Value,
        error_msg: &str,
    ) -> Result<T, ExchangeError>
    where
        T: DeserializeOwned,
    {
        let items: Vec<T> = self.handle_response(response_value)?;

        items
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::InvalidResponseFormat(error_msg.to_string()))
    }

    /// Get system time from OKX
    pub async fn get_system_time(&self) -> Result<u64, ExchangeError> {
        let response_value = self
            .rest_client
            .get("/api/v5/public/time", &[], false)
            .await?;

        let items: Vec<HashMap<String, String>> = self.handle_response(response_value)?;

        let timestamp_str = items
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
        self.handle_response(response_value)
    }

    /// Get ticker information
    pub async fn get_ticker(&self, inst_id: &str) -> Result<OkxTicker, ExchangeError> {
        let endpoint = "/api/v5/market/ticker";
        let query_params = &[("instId", inst_id)];

        let response_value = self.rest_client.get(endpoint, query_params, false).await?;
        self.handle_single_item_response(response_value, "No ticker data found")
    }

    /// Get all tickers
    pub async fn get_tickers(&self, inst_type: &str) -> Result<Vec<OkxTicker>, ExchangeError> {
        let endpoint = "/api/v5/market/tickers";
        let query_params = &[("instType", inst_type)];

        let response_value = self.rest_client.get(endpoint, query_params, false).await?;
        self.handle_response(response_value)
    }

    /// Get order book
    pub async fn get_order_book(
        &self,
        inst_id: &str,
        sz: Option<u32>,
    ) -> Result<OkxOrderBook, ExchangeError> {
        let endpoint = "/api/v5/market/books";
        let sz_str = sz.map(|s| s.to_string());
        let mut query_params = vec![("instId", inst_id)];
        if let Some(ref sz_val) = sz_str {
            query_params.push(("sz", sz_val.as_str()));
        }

        let response_value = self.rest_client.get(endpoint, &query_params, false).await?;
        self.handle_single_item_response(response_value, "No order book data found")
    }

    /// Get recent trades
    pub async fn get_trades(
        &self,
        inst_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<OkxTrade>, ExchangeError> {
        let endpoint = "/api/v5/market/trades";
        let limit_str = limit.map(|l| l.to_string());
        let mut query_params = vec![("instId", inst_id)];
        if let Some(ref limit_val) = limit_str {
            query_params.push(("limit", limit_val.as_str()));
        }

        let response_value = self.rest_client.get(endpoint, &query_params, false).await?;
        self.handle_response(response_value)
    }

    /// Get candlestick data
    pub async fn get_candlesticks(
        &self,
        inst_id: &str,
        bar: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<OkxKline>, ExchangeError> {
        let endpoint = "/api/v5/market/candles";
        let mut query_params = vec![("instId", inst_id)];

        let bar_str;
        if let Some(b) = bar {
            bar_str = b.to_string();
            query_params.push(("bar", &bar_str));
        }

        let limit_str;
        if let Some(lmt) = limit {
            limit_str = lmt.to_string();
            query_params.push(("limit", &limit_str));
        }

        let response_value = self.rest_client.get(endpoint, &query_params, false).await?;
        let response: OkxResponse<Vec<Vec<String>>> = serde_json::from_value(response_value)
            .map_err(|e| {
                ExchangeError::DeserializationError(format!("Failed to parse response: {}", e))
            })?;

        if response.code != "0" {
            return Err(self.map_okx_error(&response.code, &response.msg));
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
        self.handle_single_item_response(response_value, "No order response data found")
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

        let response_value = self.rest_client.post(endpoint, &cancel_req, true).await?;
        self.handle_single_item_response(response_value, "No cancel response data found")
    }

    /// Get order details
    pub async fn get_order(
        &self,
        inst_id: &str,
        ord_id: Option<&str>,
        cl_ord_id: Option<&str>,
    ) -> Result<OkxOrder, ExchangeError> {
        let endpoint = "/api/v5/trade/order";
        let mut query_params = vec![("instId", inst_id)];

        let ord_id_str;
        if let Some(id) = ord_id {
            ord_id_str = id.to_string();
            query_params.push(("ordId", &ord_id_str));
        }

        let cl_ord_id_str;
        if let Some(cl_id) = cl_ord_id {
            cl_ord_id_str = cl_id.to_string();
            query_params.push(("clOrdId", &cl_ord_id_str));
        }

        let response_value = self.rest_client.get(endpoint, &query_params, true).await?;
        self.handle_single_item_response(response_value, "No order data found")
    }

    /// Get pending orders
    pub async fn get_pending_orders(
        &self,
        inst_type: Option<&str>,
    ) -> Result<Vec<OkxOrder>, ExchangeError> {
        let endpoint = "/api/v5/trade/orders-pending";
        let query_params =
            inst_type.map_or_else(Vec::new, |inst_type| vec![("instType", inst_type)]);

        let response_value = self.rest_client.get(endpoint, &query_params, true).await?;
        self.handle_response(response_value)
    }

    // Account API endpoints

    /// Get account balance
    pub async fn get_balance(&self, ccy: Option<&str>) -> Result<OkxAccountInfo, ExchangeError> {
        let endpoint = "/api/v5/account/balance";
        let query_params = ccy.map_or_else(Vec::new, |currency| vec![("ccy", currency)]);

        let response_value = self.rest_client.get(endpoint, &query_params, true).await?;
        self.handle_single_item_response(response_value, "No account data found")
    }
}
