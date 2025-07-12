use crate::core::errors::ExchangeError;
use crate::core::kernel::Signer;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub struct BinanceSigner {
    api_key: String,
    secret_key: String,
}

impl BinanceSigner {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self {
            api_key,
            secret_key,
        }
    }

    fn generate_signature(&self, query_string: &str) -> Result<String, ExchangeError> {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .map_err(|e| ExchangeError::AuthError(format!("Failed to create HMAC: {}", e)))?;
        mac.update(query_string.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }
}

impl Signer for BinanceSigner {
    fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        query_string: &str,
        _body: &[u8],
        timestamp: u64,
    ) -> Result<(HashMap<String, String>, Vec<(String, String)>), ExchangeError> {
        // Build the full query string with timestamp
        let full_query = if query_string.is_empty() {
            format!("timestamp={}", timestamp)
        } else {
            format!("{}&timestamp={}", query_string, timestamp)
        };

        // Generate signature
        let signature = self.generate_signature(&full_query)?;

        // Prepare headers
        let mut headers = HashMap::new();
        headers.insert("X-MBX-APIKEY".to_string(), self.api_key.clone());

        // Prepare additional query parameters
        let params = vec![
            ("timestamp".to_string(), timestamp.to_string()),
            ("signature".to_string(), signature),
        ];

        Ok((headers, params))
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn get_timestamp() -> Result<u64, ExchangeError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .map_err(|e| ExchangeError::Other(format!("System time error: {}", e)))
}

#[must_use]
pub fn build_query_string(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

/// Legacy function - kept for backward compatibility
pub fn generate_signature(secret: &str, query_string: &str) -> Result<String, ExchangeError> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| ExchangeError::AuthError(format!("Failed to create HMAC: {}", e)))?;
    mac.update(query_string.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

/// Legacy function - kept for backward compatibility
pub fn sign_request(
    params: &[(&str, String)],
    secret: &str,
    _method: &str,
    _endpoint: &str,
) -> Result<String, ExchangeError> {
    let query_string = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&");

    generate_signature(secret, &query_string)
}
