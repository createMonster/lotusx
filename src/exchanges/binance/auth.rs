use crate::core::errors::ExchangeError;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub fn generate_signature(secret: &str, query_string: &str) -> Result<String, ExchangeError> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| ExchangeError::AuthError(format!("Failed to create HMAC: {}", e)))?;
    mac.update(query_string.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
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

/// Sign a request with the given parameters
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
