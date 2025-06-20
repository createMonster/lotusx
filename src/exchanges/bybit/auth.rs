use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Sign request for Bybit V5 API
pub fn sign_v5_request(
    body: &str,
    secret_key: &str,
    api_key: &str,
    timestamp: u64,
) -> Result<String, crate::core::errors::ExchangeError> {
    let recv_window = "5000";

    // For V5 API: timestamp + api_key + recv_window + body
    let payload = format!("{}{}{}{}", timestamp, api_key, recv_window, body);

    // Sign with HMAC-SHA256
    let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).map_err(|_| {
        crate::core::errors::ExchangeError::AuthError("Invalid secret key".to_string())
    })?;

    mac.update(payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    Ok(signature)
}

/// Legacy sign request for V2 API (deprecated)
pub fn sign_request(
    params: &[(String, String)],
    secret_key: &str,
    api_key: &str,
    method: &str,
    _path: &str,
) -> Result<String, crate::core::errors::ExchangeError> {
    let recv_window = "5000"; // 5 seconds

    // Extract timestamp from params
    let timestamp = params
        .iter()
        .find(|(key, _)| key == "timestamp")
        .map_or_else(|| get_timestamp().to_string(), |(_, value)| value.clone());

    // Build query string for GET requests, excluding signature-related params AND timestamp
    if method == "GET" {
        let mut query_params = Vec::new();
        for (key, value) in params {
            if key != "sign" && key != "timestamp" {
                query_params.push(format!("{}={}", key, value));
            }
        }
        let query_string = query_params.join("&");

        // For V5 API signature: timestamp + api_key + recv_window + query_string
        let payload = format!("{}{}{}{}", timestamp, api_key, recv_window, query_string);

        // Sign with HMAC-SHA256
        let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).map_err(|_| {
            crate::core::errors::ExchangeError::AuthError("Invalid secret key".to_string())
        })?;

        mac.update(payload.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        Ok(signature)
    } else {
        // For POST requests, build form data, excluding signature-related params AND timestamp
        let mut form_params = Vec::new();
        for (key, value) in params {
            if key != "sign" && key != "timestamp" {
                form_params.push(format!("{}={}", key, value));
            }
        }
        let form_data = form_params.join("&");

        // For V5 API signature: timestamp + api_key + recv_window + form_data
        let payload = format!("{}{}{}{}", timestamp, api_key, recv_window, form_data);

        // Sign with HMAC-SHA256
        let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes()).map_err(|_| {
            crate::core::errors::ExchangeError::AuthError("Invalid secret key".to_string())
        })?;

        mac.update(payload.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        Ok(signature)
    }
}
