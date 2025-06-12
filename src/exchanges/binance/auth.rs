use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[must_use]
pub fn generate_signature(secret: &str, query_string: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(query_string.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64
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
) -> Result<String, crate::core::errors::ExchangeError> {
    let query_string = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<String>>()
        .join("&");
    
    Ok(generate_signature(secret, &query_string))
}
