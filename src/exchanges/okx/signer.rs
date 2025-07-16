use crate::core::errors::ExchangeError;
use crate::core::kernel::Signer;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub struct OkxSigner {
    api_key: String,
    secret_key: String,
    passphrase: String,
}

impl OkxSigner {
    pub fn new(api_key: String, secret_key: String, passphrase: String) -> Self {
        Self {
            api_key,
            secret_key,
            passphrase,
        }
    }

    /// Generate the signature for OKX API requests
    /// The prehash string format is: timestamp + method + requestPath + body
    fn generate_signature(
        &self,
        timestamp: &str,
        method: &str,
        request_path: &str,
        body: &str,
    ) -> Result<String, ExchangeError> {
        let prehash = format!("{}{}{}{}", timestamp, method, request_path, body);

        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .map_err(|e| ExchangeError::AuthError(format!("Failed to create HMAC: {}", e)))?;

        mac.update(prehash.as_bytes());
        let signature_bytes = mac.finalize().into_bytes();

        // OKX requires base64 encoding of the signature
        Ok(general_purpose::STANDARD.encode(signature_bytes))
    }

    /// Get current timestamp in ISO format as required by OKX
    fn get_timestamp() -> Result<String, ExchangeError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| ExchangeError::AuthError(format!("Failed to get timestamp: {}", e)))?
            .as_millis();

        // OKX requires timestamp in ISO format
        let datetime = chrono::DateTime::from_timestamp_millis(timestamp as i64)
            .ok_or_else(|| ExchangeError::AuthError("Invalid timestamp".to_string()))?;

        Ok(datetime.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
    }
}

impl Signer for OkxSigner {
    fn sign_request(
        &self,
        method: &str,
        endpoint: &str,
        query_string: &str,
        body: &[u8],
        _timestamp: u64, // We generate our own timestamp for OKX
    ) -> Result<(HashMap<String, String>, Vec<(String, String)>), ExchangeError> {
        // Generate ISO timestamp for OKX
        let timestamp = Self::get_timestamp()?;

        // Build request path with query string if present
        let request_path = if query_string.is_empty() {
            endpoint.to_string()
        } else {
            format!("{}?{}", endpoint, query_string)
        };

        // Convert body to string
        let body_str = std::str::from_utf8(body)
            .map_err(|e| ExchangeError::AuthError(format!("Invalid body encoding: {}", e)))?;

        // Generate signature
        let signature = self.generate_signature(&timestamp, method, &request_path, body_str)?;

        // Prepare headers - OKX requires specific header names
        let mut headers = HashMap::new();
        headers.insert("OK-ACCESS-KEY".to_string(), self.api_key.clone());
        headers.insert("OK-ACCESS-SIGN".to_string(), signature);
        headers.insert("OK-ACCESS-TIMESTAMP".to_string(), timestamp);
        headers.insert("OK-ACCESS-PASSPHRASE".to_string(), self.passphrase.clone());
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // No query parameters needed for OKX auth
        let query_params = Vec::new();

        Ok((headers, query_params))
    }
}
