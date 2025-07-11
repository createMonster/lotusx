use crate::core::errors::ExchangeError;
use crate::core::kernel::{SignatureResult, Signer};

use jsonwebtoken::{encode, EncodingKey, Header};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

/// JWT Claims for Paradex authentication
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

/// Paradex JWT-based signer implementation
pub struct ParadexSigner {
    secret_key: SecretKey,
    wallet_address: String,
    _secp: Secp256k1<secp256k1::All>,
}

impl ParadexSigner {
    /// Create a new Paradex signer with a private key
    pub fn new(private_key: String) -> Result<Self, ExchangeError> {
        let secret_key = SecretKey::from_slice(
            &hex::decode(private_key.trim_start_matches("0x"))
                .map_err(|e| ExchangeError::AuthError(format!("Invalid private key hex: {}", e)))?,
        )
        .map_err(|e| ExchangeError::AuthError(format!("Invalid private key: {}", e)))?;

        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let wallet_address = public_key_to_address(&public_key);

        Ok(Self {
            secret_key,
            wallet_address,
            _secp: secp,
        })
    }

    /// Get the wallet address derived from the private key
    pub fn wallet_address(&self) -> &str {
        &self.wallet_address
    }

    /// Sign a JWT token with the private key
    pub fn sign_jwt(&self) -> Result<String, ExchangeError> {
        let claims = Claims {
            sub: self.wallet_address.clone(),
            exp: (chrono::Utc::now() + chrono::Duration::minutes(5))
                .timestamp()
                .try_into()
                .unwrap_or(0),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_ref()),
        )
        .map_err(|e| ExchangeError::AuthError(format!("Failed to sign JWT: {}", e)))?;

        Ok(token)
    }
}

impl Signer for ParadexSigner {
    fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        query_string: &str,
        _body: &[u8],
        _timestamp: u64,
    ) -> SignatureResult {
        // For Paradex, we create a JWT token for authentication
        // The other parameters are not used as JWT contains its own payload
        match self.sign_jwt() {
            Ok(token) => {
                let mut headers = HashMap::new();
                headers.insert("Authorization".to_string(), format!("Bearer {}", token));

                // Parse query string to params if provided
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
            Err(e) => Err(e),
        }
    }
}

/// Convert a public key to an Ethereum-style address
fn public_key_to_address(public_key: &PublicKey) -> String {
    let public_key_bytes = public_key.serialize_uncompressed();

    // Remove the 0x04 prefix for uncompressed key
    let key_without_prefix = &public_key_bytes[1..];

    // Hash with Keccak256
    let mut hasher = Keccak256::new();
    hasher.update(key_without_prefix);
    let hash = hasher.finalize();

    // Take the last 20 bytes and format as hex address
    let address_bytes = &hash[12..];
    format!("0x{}", hex::encode(address_bytes))
}
