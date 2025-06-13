use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

pub fn sign_request(
    params: &[(String, String)],
    secret_key: &str,
    api_key: &str,
    _method: &str,
    _path: &str,
) -> Result<String, crate::core::errors::ExchangeError> {
    let timestamp = get_timestamp();
    
    // Build query string
    let mut query_string = String::new();
    for (key, value) in params {
        if !query_string.is_empty() {
            query_string.push('&');
        }
        query_string.push_str(&format!("{}={}", key, value));
    }
    
    // Add timestamp if not already present
    if !query_string.contains("timestamp") {
        if !query_string.is_empty() {
            query_string.push('&');
        }
        query_string.push_str(&format!("timestamp={}", timestamp));
    }
    
    // Create signature payload for Bybit
    let payload = format!("{}{}{}{}", timestamp, api_key, query_string, "");
    
    // Sign with HMAC-SHA256
    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes())
        .map_err(|_| crate::core::errors::ExchangeError::AuthError("Invalid secret key".to_string()))?;
    
    mac.update(payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    
    Ok(signature)
} 