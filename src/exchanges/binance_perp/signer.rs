use crate::core::errors::ExchangeError;
use crate::core::kernel::Signer;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

pub struct BinancePerpSigner {
    api_key: String,
    secret_key: String,
}

impl BinancePerpSigner {
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

impl Signer for BinancePerpSigner {
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
