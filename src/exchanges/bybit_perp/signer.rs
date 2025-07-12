use crate::core::errors::ExchangeError;
use crate::core::kernel::Signer;
use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// Bybit Perpetual HMAC-SHA256 signer for authenticated requests using V5 API
#[derive(Debug, Clone)]
pub struct BybitPerpSigner {
    api_key: String,
    secret_key: String,
}

impl BybitPerpSigner {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
        }
    }

    /// Get current timestamp in milliseconds
    pub fn get_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Sign request for Bybit V5 API
    pub fn sign_v5_request(&self, body: &str, timestamp: u64) -> Result<String, ExchangeError> {
        let recv_window = "5000";

        // For V5 API: timestamp + api_key + recv_window + body
        let payload = format!("{}{}{}{}", timestamp, self.api_key, recv_window, body);

        // Sign with HMAC-SHA256
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .map_err(|_| ExchangeError::AuthError("Invalid secret key".to_string()))?;

        mac.update(payload.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        Ok(signature)
    }

    /// Create signature for query parameters (GET requests)
    fn create_signature_for_params(
        &self,
        timestamp: u64,
        query_string: &str,
    ) -> Result<String, ExchangeError> {
        let recv_window = "5000";

        // For V5 API signature: timestamp + api_key + recv_window + query_string
        let payload = format!(
            "{}{}{}{}",
            timestamp, self.api_key, recv_window, query_string
        );

        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .map_err(|_| ExchangeError::AuthError("Invalid secret key".to_string()))?;

        mac.update(payload.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        Ok(signature)
    }
}

impl Signer for BybitPerpSigner {
    fn sign_request(
        &self,
        method: &str,
        _endpoint: &str,
        query_string: &str,
        body: &[u8],
        timestamp: u64,
    ) -> Result<(HashMap<String, String>, Vec<(String, String)>), ExchangeError> {
        let mut headers = HashMap::new();
        headers.insert("X-BAPI-API-KEY".to_string(), self.api_key.clone());
        headers.insert("X-BAPI-TIMESTAMP".to_string(), timestamp.to_string());
        headers.insert("X-BAPI-RECV-WINDOW".to_string(), "5000".to_string());

        let signature = if method == "GET" {
            self.create_signature_for_params(timestamp, query_string)?
        } else {
            // For POST requests, use body content
            let body_str = std::str::from_utf8(body)
                .map_err(|_| ExchangeError::AuthError("Invalid body encoding".to_string()))?;
            self.sign_v5_request(body_str, timestamp)?
        };

        headers.insert("X-BAPI-SIGN".to_string(), signature);

        // No additional query parameters needed for V5 API
        let params = vec![];

        Ok((headers, params))
    }
}

// Module-level convenience functions for backward compatibility
pub fn get_timestamp() -> u64 {
    BybitPerpSigner::get_timestamp()
}

/// Legacy function for backward compatibility
pub fn sign_v5_request(
    body: &str,
    secret_key: &str,
    api_key: &str,
    timestamp: u64,
) -> Result<String, ExchangeError> {
    let signer = BybitPerpSigner::new(api_key.to_string(), secret_key.to_string());
    signer.sign_v5_request(body, timestamp)
}

/// Legacy function for backward compatibility
pub fn sign_request(
    params: &[(String, String)],
    secret_key: &str,
    api_key: &str,
    method: &str,
    endpoint: &str,
) -> Result<String, ExchangeError> {
    let signer = BybitPerpSigner::new(api_key.to_string(), secret_key.to_string());
    let timestamp = get_timestamp();

    let query_string = params
        .iter()
        .filter(|(k, _)| k != "timestamp")
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&");

    let (_, _) = signer.sign_request(method, endpoint, &query_string, &[], timestamp)?;

    if method == "GET" {
        signer.create_signature_for_params(timestamp, &query_string)
    } else {
        signer.sign_v5_request("", timestamp)
    }
}
