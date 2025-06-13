use crate::core::{config::ExchangeConfig, errors::ExchangeError, traits::ExchangeConnector};
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use reqwest::Client;
use secrecy::ExposeSecret;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct BackpackConnector {
    pub(crate) client: Client,
    #[allow(dead_code)]
    pub(crate) config: ExchangeConfig,
    pub(crate) base_url: String,
    pub(crate) signing_key: Option<SigningKey>,
    pub(crate) verifying_key: Option<VerifyingKey>,
}

impl BackpackConnector {
    pub fn new(config: ExchangeConfig) -> Result<Self, ExchangeError> {
        let base_url = if config.testnet {
            "https://api.backpack.exchange".to_string() // Backpack doesn't have a testnet
        } else {
            config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.backpack.exchange".to_string())
        };

        let (signing_key, verifying_key) = {
            let secret_bytes = base64::engine::general_purpose::STANDARD
                .decode(config.secret_key.expose_secret())
                .map_err(|e| ExchangeError::AuthError(format!("Invalid secret key: {}", e)))?;

            if secret_bytes.len() != 32 {
                return Err(ExchangeError::AuthError(
                    "Secret key must be 32 bytes".to_string(),
                ));
            }

            let mut key_bytes = [0u8; 32];
            key_bytes.copy_from_slice(&secret_bytes);

            let signing_key = SigningKey::from_bytes(&key_bytes);
            let verifying_key = signing_key.verifying_key();

            (Some(signing_key), Some(verifying_key))
        };

        Ok(Self {
            client: Client::new(),
            config,
            base_url,
            signing_key,
            verifying_key,
        })
    }

    /// Generate signature for Backpack Exchange API requests
    pub fn generate_signature(
        &self,
        instruction: &str,
        params: &str,
        timestamp: i64,
        window: i64,
    ) -> Result<String, ExchangeError> {
        let signing_key = self
            .signing_key
            .as_ref()
            .ok_or_else(|| ExchangeError::AuthError("No signing key available".to_string()))?;

        // Create the signing string according to Backpack's specification
        let signing_string = if params.is_empty() {
            format!(
                "instruction={}&timestamp={}&window={}",
                instruction, timestamp, window
            )
        } else {
            format!(
                "instruction={}&{}&timestamp={}&window={}",
                instruction, params, timestamp, window
            )
        };

        // Sign the message
        let signature = signing_key.sign(signing_string.as_bytes());

        // Return base64 encoded signature
        Ok(base64::engine::general_purpose::STANDARD.encode(signature.to_bytes()))
    }

    /// Get current timestamp in milliseconds
    pub(crate) fn get_timestamp() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    /// Create signed headers for authenticated requests
    pub(crate) fn create_signed_headers(
        &self,
        instruction: &str,
        params: &str,
    ) -> Result<std::collections::HashMap<String, String>, ExchangeError> {
        let timestamp = Self::get_timestamp();
        let window = 5000; // Default window in milliseconds
        let signature = self.generate_signature(instruction, params, timestamp, window)?;
        let api_key = self
            .verifying_key
            .as_ref()
            .ok_or_else(|| ExchangeError::AuthError("No verifying key available".to_string()))?;

        let mut headers = std::collections::HashMap::new();
        headers.insert("X-Timestamp".to_string(), timestamp.to_string());
        headers.insert("X-Window".to_string(), window.to_string());
        headers.insert(
            "X-API-Key".to_string(),
            base64::engine::general_purpose::STANDARD.encode(api_key.to_bytes()),
        );
        headers.insert("X-Signature".to_string(), signature);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        Ok(headers)
    }

    /// Create query string from parameters
    pub(crate) fn create_query_string(params: &[(String, String)]) -> String {
        let mut sorted_params = params.to_vec();
        sorted_params.sort_by(|a, b| a.0.cmp(&b.0));

        sorted_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }
}

impl ExchangeConnector for BackpackConnector {}
