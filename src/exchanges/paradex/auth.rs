use crate::core::errors::ExchangeError;
use jsonwebtoken::{encode, EncodingKey, Header};
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub struct ParadexAuth {
    secret_key: Option<SecretKey>,
    wallet_address: Option<String>,
    secp: Secp256k1<secp256k1::All>,
}

impl Default for ParadexAuth {
    fn default() -> Self {
        Self::new()
    }
}

impl ParadexAuth {
    pub fn new() -> Self {
        Self {
            secret_key: None,
            wallet_address: None,
            secp: Secp256k1::new(),
        }
    }

    pub fn with_private_key(private_key: &str) -> Result<Self, ExchangeError> {
        let secret_key = SecretKey::from_slice(
            &hex::decode(private_key.trim_start_matches("0x"))
                .map_err(|e| ExchangeError::AuthError(format!("Invalid private key hex: {}", e)))?,
        )
        .map_err(|e| ExchangeError::AuthError(format!("Invalid private key: {}", e)))?;

        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let wallet_address = public_key_to_address(&public_key);

        Ok(Self {
            secret_key: Some(secret_key),
            wallet_address: Some(wallet_address),
            secp,
        })
    }

    pub fn wallet_address(&self) -> Option<&str> {
        self.wallet_address.as_deref()
    }

    pub fn can_sign(&self) -> bool {
        self.secret_key.is_some()
    }

    pub fn sign_jwt(&self) -> Result<String, ExchangeError> {
        let secret_key = self.secret_key.ok_or_else(|| {
            ExchangeError::AuthError("No private key available for signing".to_string())
        })?;

        let claims = Claims {
            sub: self.wallet_address.as_ref().unwrap().to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::minutes(5)).timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret_key.as_ref()),
        )
        .map_err(|e| ExchangeError::AuthError(format!("Failed to sign JWT: {}", e)))?;

        Ok(token)
    }
}

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
