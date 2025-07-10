use crate::core::{config::ExchangeConfig, errors::ExchangeError};
use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use secrecy::ExposeSecret;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct BackpackAuth {
    signing_key: Option<SigningKey>,
    verifying_key: Option<VerifyingKey>,
}

impl BackpackAuth {
    pub fn new(config: &ExchangeConfig) -> Result<Self, ExchangeError> {
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
    pub fn get_timestamp() -> Result<i64, ExchangeError> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .map_err(|e| ExchangeError::Other(format!("System time error: {}", e)))
    }

    /// Create WebSocket authentication message
    pub fn create_websocket_auth_message(&self) -> Result<String, ExchangeError> {
        let timestamp = Self::get_timestamp()?;
        let window = 5000; // Default window in milliseconds

        // Try different instruction names for WebSocket auth
        let auth_instruction = "subscribe"; // WebSocket might use different instruction
        let auth_params = "";
        let auth_signature =
            self.generate_signature(auth_instruction, auth_params, timestamp, window)?;

        let auth_message = json!({
            "method": "AUTH",
            "params": {
                "instruction": auth_instruction,
                "timestamp": timestamp,
                "window": window,
                "signature": auth_signature
            },
            "id": 1
        });

        Ok(serde_json::to_string(&auth_message)?)
    }

    /// Create signed headers for REST API requests
    pub fn create_signed_headers(
        &self,
        instruction: &str,
        params: &str,
    ) -> Result<std::collections::HashMap<String, String>, ExchangeError> {
        let timestamp = Self::get_timestamp()?;
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

    /// Check if authentication is available
    pub fn can_authenticate(&self) -> bool {
        self.signing_key.is_some() && self.verifying_key.is_some()
    }
}
