use crate::core::errors::ExchangeError;
use async_trait::async_trait;
use base64::engine::general_purpose;
use base64::Engine;
use ed25519_dalek::{Signer as Ed25519SignerTrait, SigningKey, VerifyingKey};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

/// Result type for signing operations: (headers, `query_params`)
pub type SignatureResult = Result<(HashMap<String, String>, Vec<(String, String)>), ExchangeError>;

/// Signer trait for request authentication
///
/// This trait provides a unified interface for different authentication methods
/// used by various exchanges. Implementations handle the specific signing logic
/// for each exchange's requirements.
#[async_trait]
pub trait Signer: Send + Sync {
    /// Sign a request and return headers and query parameters
    ///
    /// # Arguments
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `endpoint` - API endpoint path
    /// * `query_string` - Query string (without leading '?')
    /// * `body` - Raw request body bytes
    /// * `timestamp` - Request timestamp in milliseconds
    ///
    /// # Returns
    /// Tuple of (headers, signed_query_params) to include in the request
    fn sign_request(
        &self,
        method: &str,
        endpoint: &str,
        query_string: &str,
        body: &[u8],
        timestamp: u64,
    ) -> SignatureResult;
}

/// HMAC-based signer for exchanges using SHA256 signatures
pub struct HmacSigner {
    api_key: String,
    secret_key: String,
    exchange_type: HmacExchangeType,
}

/// Supported HMAC exchange types
#[derive(Debug, Clone)]
pub enum HmacExchangeType {
    Binance,
    Bybit,
}

impl HmacSigner {
    /// Create a new HMAC signer
    ///
    /// # Arguments
    /// * `api_key` - API key from the exchange
    /// * `secret_key` - Secret key for signing
    /// * `exchange_type` - Which exchange format to use
    pub fn new(api_key: String, secret_key: String, exchange_type: HmacExchangeType) -> Self {
        Self {
            api_key,
            secret_key,
            exchange_type,
        }
    }

    fn sign_binance(&self, query_string: &str) -> Result<String, ExchangeError> {
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .map_err(|e| ExchangeError::AuthError(format!("Invalid secret key: {}", e)))?;

        mac.update(query_string.as_bytes());
        let result = mac.finalize();

        Ok(hex::encode(result.into_bytes()))
    }

    fn sign_bybit(
        &self,
        _method: &str,
        _endpoint: &str,
        query_string: &str,
        body: &[u8],
        timestamp: u64,
    ) -> Result<String, ExchangeError> {
        let recv_window = 5000;

        let payload = if body.is_empty() {
            format!(
                "{}{}{}{}",
                timestamp, self.api_key, recv_window, query_string
            )
        } else {
            format!(
                "{}{}{}{}",
                timestamp,
                self.api_key,
                recv_window,
                std::str::from_utf8(body).unwrap_or_default()
            )
        };

        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret_key.as_bytes())
            .map_err(|e| ExchangeError::AuthError(format!("Invalid secret key: {}", e)))?;

        mac.update(payload.as_bytes());
        let result = mac.finalize();

        Ok(hex::encode(result.into_bytes()))
    }
}

#[async_trait]
impl Signer for HmacSigner {
    fn sign_request(
        &self,
        method: &str,
        endpoint: &str,
        query_string: &str,
        body: &[u8],
        timestamp: u64,
    ) -> SignatureResult {
        match self.exchange_type {
            HmacExchangeType::Binance => {
                // For Binance, add timestamp to query string before signing
                let mut query_with_timestamp = if query_string.is_empty() {
                    format!("timestamp={}", timestamp)
                } else {
                    format!("{}&timestamp={}", query_string, timestamp)
                };

                // Add body params for POST requests
                if !body.is_empty() && method == "POST" {
                    if let Ok(body_str) = std::str::from_utf8(body) {
                        if !body_str.is_empty() {
                            query_with_timestamp = format!("{}&{}", query_with_timestamp, body_str);
                        }
                    }
                }

                let signature = self.sign_binance(&query_with_timestamp)?;

                let mut headers = HashMap::new();
                headers.insert("X-MBX-APIKEY".to_string(), self.api_key.clone());

                // Parse back to individual params
                let mut signed_params = Vec::new();
                for param in query_with_timestamp.split('&') {
                    if let Some((k, v)) = param.split_once('=') {
                        signed_params.push((k.to_string(), v.to_string()));
                    }
                }
                signed_params.push(("signature".to_string(), signature));

                Ok((headers, signed_params))
            }
            HmacExchangeType::Bybit => {
                let signature = self.sign_bybit(method, endpoint, query_string, body, timestamp)?;

                let mut headers = HashMap::new();
                headers.insert("X-BAPI-API-KEY".to_string(), self.api_key.clone());
                headers.insert("X-BAPI-TIMESTAMP".to_string(), timestamp.to_string());
                headers.insert("X-BAPI-RECV-WINDOW".to_string(), "5000".to_string());
                headers.insert("X-BAPI-SIGN".to_string(), signature);

                // Parse query string to params
                let signed_params = if query_string.is_empty() {
                    Vec::new()
                } else {
                    query_string
                        .split('&')
                        .filter_map(|param| {
                            param
                                .split_once('=')
                                .map(|(k, v)| (k.to_string(), v.to_string()))
                        })
                        .collect()
                };

                Ok((headers, signed_params))
            }
        }
    }
}

/// Ed25519-based signer for exchanges like Backpack
pub struct Ed25519Signer {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Ed25519Signer {
    /// Create a new Ed25519 signer from a base64-encoded private key
    ///
    /// # Arguments
    /// * `private_key` - Base64-encoded private key bytes
    pub fn new(private_key: &str) -> Result<Self, ExchangeError> {
        let key_bytes = general_purpose::STANDARD
            .decode(private_key)
            .map_err(|e| ExchangeError::AuthError(format!("Invalid private key format: {}", e)))?;

        if key_bytes.len() != 32 {
            return Err(ExchangeError::AuthError(
                "Invalid private key length".to_string(),
            ));
        }

        let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    fn generate_signature(
        &self,
        instruction: &str,
        params: &str,
        timestamp: u64,
        window: u64,
    ) -> String {
        let message = format!(
            "instruction={}&params={}&timestamp={}&window={}",
            instruction, params, timestamp, window
        );

        let signature = Ed25519SignerTrait::sign(&self.signing_key, message.as_bytes());
        general_purpose::STANDARD.encode(signature.to_bytes())
    }
}

#[async_trait]
impl Signer for Ed25519Signer {
    fn sign_request(
        &self,
        _method: &str,
        endpoint: &str,
        query_string: &str,
        body: &[u8],
        timestamp: u64,
    ) -> SignatureResult {
        let window = 5000;

        // For Backpack, the instruction is typically the endpoint without the leading slash
        let instruction = endpoint.trim_start_matches('/');

        let params = if body.is_empty() {
            query_string.to_string()
        } else {
            std::str::from_utf8(body).unwrap_or_default().to_string()
        };

        let signature = self.generate_signature(instruction, &params, timestamp, window);

        let mut headers = HashMap::new();
        headers.insert("X-Timestamp".to_string(), timestamp.to_string());
        headers.insert("X-Window".to_string(), window.to_string());
        headers.insert(
            "X-API-Key".to_string(),
            general_purpose::STANDARD.encode(self.verifying_key.to_bytes()),
        );
        headers.insert("X-Signature".to_string(), signature);
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // Parse query string to params
        let signed_params = if query_string.is_empty() {
            Vec::new()
        } else {
            query_string
                .split('&')
                .filter_map(|param| {
                    param
                        .split_once('=')
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                })
                .collect()
        };

        Ok((headers, signed_params))
    }
}

/// JWT-based signer for exchanges like Paradex
pub struct JwtSigner {
    #[allow(dead_code)]
    private_key: String,
    // Add JWT-specific fields as needed
}

impl JwtSigner {
    /// Create a new JWT signer
    ///
    /// # Arguments
    /// * `private_key` - Private key for JWT signing
    pub fn new(private_key: String) -> Self {
        Self { private_key }
    }
}

#[async_trait]
impl Signer for JwtSigner {
    fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        query_string: &str,
        _body: &[u8],
        _timestamp: u64,
    ) -> SignatureResult {
        // JWT signing implementation would go here
        // For now, return a placeholder
        let mut headers = HashMap::new();
        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", "jwt_token_placeholder"),
        );

        // Parse query string to params
        let signed_params = if query_string.is_empty() {
            Vec::new()
        } else {
            query_string
                .split('&')
                .filter_map(|param| {
                    param
                        .split_once('=')
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                })
                .collect()
        };

        Ok((headers, signed_params))
    }
}
