use super::types::{ExchangeRequest, SignedAction};
use crate::core::errors::ExchangeError;
use crate::core::kernel::signer::{SignatureResult, Signer};
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use serde_json::{json, Value};
use sha3::{Digest, Keccak256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Hyperliquid signer implementation using secp256k1 (Ethereum-style signatures)
#[derive(Clone)]
pub struct HyperliquidSigner {
    secret_key: Option<SecretKey>,
    wallet_address: Option<String>,
    secp: Secp256k1<secp256k1::All>,
}

impl Default for HyperliquidSigner {
    fn default() -> Self {
        Self::new()
    }
}

impl HyperliquidSigner {
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

    pub fn with_wallet_address(wallet_address: &str) -> Self {
        Self {
            secret_key: None,
            wallet_address: Some(wallet_address.to_string()),
            secp: Secp256k1::new(),
        }
    }

    pub fn wallet_address(&self) -> Option<&str> {
        self.wallet_address.as_deref()
    }

    pub fn can_sign(&self) -> bool {
        self.secret_key.is_some()
    }

    pub fn sign_l1_action(
        &self,
        action: Value,
        vault_address: Option<String>,
        nonce: Option<u64>,
    ) -> Result<ExchangeRequest, ExchangeError> {
        let secret_key = self.secret_key.ok_or_else(|| {
            ExchangeError::AuthError("No private key available for signing".to_string())
        })?;

        let nonce = nonce.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        });

        // Create the signing data according to Hyperliquid's format
        let connection_id = json!({
            "chain": "Arbitrum",
            "chainId": "0xa4b1",
            "name": "Exchange",
            "verifyingContract": "0x0000000000000000000000000000000000000000",
            "version": "1"
        });

        let agent = json!({
            "source": if vault_address.is_some() { "a" } else { "b" }
        });

        let is_mainnet = true; // TODO: Make this configurable
        let signing_data = json!({
            "action": action,
            "nonce": nonce,
            "connectionId": connection_id,
            "agent": agent,
            "isMainnet": is_mainnet
        });

        let signing_hash =
            serde_json::to_string(&signing_data).map_err(ExchangeError::JsonError)?;

        // Hash the signing data with Keccak256
        let mut hasher = Keccak256::new();
        hasher.update(signing_hash.as_bytes());
        let hash = hasher.finalize();

        // Sign the hash
        let message = Message::from_digest_slice(&hash)
            .map_err(|e| ExchangeError::AuthError(format!("Failed to create message: {}", e)))?;

        let signature = self.secp.sign_ecdsa(&message, &secret_key);

        // Format signature for Hyperliquid (r + s + v format)
        let signature_bytes = signature.serialize_compact();
        let mut sig_with_recovery = [0u8; 65];
        sig_with_recovery[..64].copy_from_slice(&signature_bytes);
        sig_with_recovery[64] = 27; // Recovery ID for Ethereum-style signatures

        let signature_hex = format!("0x{}", hex::encode(sig_with_recovery));

        let signed_action = SignedAction {
            action,
            nonce,
            signature: signature_hex,
        };

        Ok(ExchangeRequest {
            action: signed_action,
            vault_address,
        })
    }
}

impl Signer for HyperliquidSigner {
    fn sign_request(
        &self,
        _method: &str,
        _endpoint: &str,
        _query_string: &str,
        _body: &[u8],
        _timestamp: u64,
    ) -> SignatureResult {
        // For Hyperliquid, we don't use standard HTTP signing
        // Instead, we use the exchange-specific L1 action signing
        if !self.can_sign() {
            return Err(ExchangeError::AuthError(
                "No private key available for signing".to_string(),
            ));
        }

        // For HTTP requests, we typically don't need to sign them in Hyperliquid
        // The signing is done for exchange actions via sign_l1_action
        Ok((std::collections::HashMap::new(), Vec::new()))
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

// Utility function to generate nonces
pub fn generate_nonce() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signer_creation() {
        let signer = HyperliquidSigner::new();
        assert!(!signer.can_sign());
        assert!(signer.wallet_address().is_none());
    }

    #[test]
    fn test_wallet_address_creation() {
        let signer =
            HyperliquidSigner::with_wallet_address("0x1234567890123456789012345678901234567890");
        assert!(!signer.can_sign());
        assert_eq!(
            signer.wallet_address(),
            Some("0x1234567890123456789012345678901234567890")
        );
    }
}
