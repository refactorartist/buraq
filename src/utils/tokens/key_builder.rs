//! KeyBuilder module provides functionality to generate cryptographic keys
//! for various JWT algorithms.

use anyhow::{Context, Error, Result};
use jsonwebtoken::Algorithm;
use openssl::pkey::PKey;
use std::str;

use crate::utils::tokens::{
    hmac::{self, HmacHashFunction, HmacKeyLength},
    rsa::{self, RsaKeyLength},
};

/// KeyBuilder is responsible for generating cryptographic keys based on the JWT algorithm.
/// It provides methods to generate keys for various algorithms including HMAC, RSA, and more.
///
/// # Examples
///
/// ```
/// use jsonwebtoken::Algorithm;
/// use buraq::utils::tokens::key_builder::KeyBuilder;
///
/// let key_builder = KeyBuilder::new();
/// // Generate an HMAC key for HS256
/// let key_pair = key_builder.generate_key(Algorithm::HS256).unwrap();
/// // Generate an RSA key for RS256
/// let rsa_key_pair = key_builder.generate_key(Algorithm::RS256).unwrap();
/// ```
pub struct KeyBuilder;

/// Represents a pair of keys (private and public) for asymmetric algorithms.
/// For symmetric algorithms like HMAC, only the private_key field is used.
pub struct KeyPair {
    /// The private key in a format appropriate for the algorithm.
    /// For HMAC, this is the shared secret key.
    pub private_key: Vec<u8>,

    /// The public key, if applicable (for asymmetric algorithms like RSA).
    /// This is None for symmetric algorithms like HMAC.
    pub public_key: Option<Vec<u8>>,
}

impl KeyBuilder {
    /// Creates a new KeyBuilder instance
    pub fn new() -> Self {
        KeyBuilder
    }

    /// Loads a private key from a PEM string and extracts its public key
    ///
    /// # Arguments
    /// * `pem_private_key` - A string containing the private key in PEM format
    ///
    /// # Returns
    /// A `KeyPair` containing both the private key and the derived public key
    ///
    /// # Errors
    /// Returns an error if the private key is invalid or if the public key cannot be extracted
    pub fn from_private_key_pem(pem_private_key: &str) -> Result<KeyPair> {
        // Load the private key from PEM string
        let private_key = PKey::private_key_from_pem(pem_private_key.as_bytes())
            .context("Failed to load private key from PEM")?;

        // Convert private key to PEM format
        let private_pem = private_key
            .private_key_to_pem_pkcs8()
            .context("Failed to encode private key to PEM")?;

        // Extract public key from the private key
        let public_key = private_key
            .public_key_to_pem()
            .context("Failed to extract public key from private key")?;

        // Convert the public key to PEM format
        let public_pem = PKey::public_key_from_pem(&public_key)
            .context("Failed to create public key from private key")?
            .public_key_to_pem()
            .context("Failed to encode public key")?;

        Ok(KeyPair {
            private_key: private_pem,
            public_key: Some(public_pem),
        })
    }

    /// Generates a key or key pair based on the specified JWT algorithm
    pub fn generate_key(&self, algorithm: Algorithm) -> Result<KeyPair> {
        match algorithm {
            // HMAC algorithms
            Algorithm::HS256 => self.generate_hmac_key(HmacHashFunction::Sha256, None),
            Algorithm::HS384 => self.generate_hmac_key(HmacHashFunction::Sha256, None), // Using SHA256 as it's sufficient
            Algorithm::HS512 => self.generate_hmac_key(HmacHashFunction::Sha512, None),

            // RSA algorithms
            Algorithm::RS256 => self.generate_rsa_key(RsaKeyLength::B2048),
            Algorithm::RS384 => self.generate_rsa_key(RsaKeyLength::B3072),
            Algorithm::RS512 => self.generate_rsa_key(RsaKeyLength::B4096),

            // RSA-PSS algorithms
            Algorithm::PS256 => self.generate_rsa_key(RsaKeyLength::B2048),
            Algorithm::PS384 => self.generate_rsa_key(RsaKeyLength::B3072),
            Algorithm::PS512 => self.generate_rsa_key(RsaKeyLength::B4096),

            // ECDSA algorithms (not implemented yet)
            Algorithm::ES256 | Algorithm::ES384 => {
                Err(Error::msg("ECDSA key generation is not yet implemented"))
            }

            // EdDSA algorithm (not implemented yet)
            Algorithm::EdDSA => Err(Error::msg("EdDSA key generation is not yet implemented")),
        }
    }

    /// Generates an HMAC key with the specified hash function and optional key length
    pub fn generate_hmac_key(
        &self,
        hash_function: HmacHashFunction,
        key_length: Option<HmacKeyLength>,
    ) -> Result<KeyPair> {
        let key_len = key_length.unwrap_or_else(|| hash_function.recommended_by_length());
        let hmac_key = hmac::generate_hmac_key(hash_function, key_len)
            .map_err(|e| Error::msg(format!("Failed to generate HMAC key: {}", e)))?;

        // For HMAC, we only have a single key (symmetric)
        // Sign an empty message to get the key bytes
        let test_data = b"";
        let signature = hmac_key
            .sign(test_data)
            .map_err(|e| Error::msg(format!("Failed to sign with HMAC key: {}", e)))?;

        Ok(KeyPair {
            private_key: signature,
            public_key: None,
        })
    }

    /// Generates an RSA key pair with the specified key length
    pub fn generate_rsa_key(&self, key_length: RsaKeyLength) -> Result<KeyPair> {
        // Generate the private key
        let private_key = rsa::generate_rsa_key_pair(key_length)?.0; // We only need the private key

        // Convert private key to PEM format
        let private_pem = private_key
            .private_key_to_pem_pkcs8()
            .map_err(|e| Error::msg(format!("Failed to encode private key: {}", e)))?;

        // Extract public key from the private key
        let public_key = private_key
            .public_key_to_pem()
            .map_err(|e| Error::msg(format!("Failed to extract public key: {}", e)))?;

        // Convert the public key to PEM format
        let public_pem = PKey::public_key_from_pem(&public_key)
            .map_err(|e| {
                Error::msg(format!(
                    "Failed to create public key from private key: {}",
                    e
                ))
            })?
            .public_key_to_pem()
            .map_err(|e| Error::msg(format!("Failed to encode public key: {}", e)))?;

        Ok(KeyPair {
            private_key: private_pem,
            public_key: Some(public_pem),
        })
    }

    /// Generates a key for a specific algorithm with a custom key length (for HMAC)
    pub fn generate_key_with_length(
        &self,
        algorithm: Algorithm,
        key_length: Option<usize>,
    ) -> Result<KeyPair> {
        match algorithm {
            Algorithm::HS256 | Algorithm::HS384 | Algorithm::HS512 => {
                let hash_function = match algorithm {
                    Algorithm::HS256 => HmacHashFunction::Sha256,
                    Algorithm::HS384 => HmacHashFunction::Sha256, // Using SHA256 as it's sufficient
                    Algorithm::HS512 => HmacHashFunction::Sha512,
                    _ => unreachable!(),
                };

                let key_len = key_length
                    .map(|len| {
                        // Map the key length to the closest HmacKeyLength
                        if len <= 16 {
                            HmacKeyLength::B128
                        } else if len <= 24 {
                            HmacKeyLength::B192
                        } else if len <= 32 {
                            HmacKeyLength::B256
                        } else if len <= 48 {
                            HmacKeyLength::B384
                        } else {
                            HmacKeyLength::B512
                        }
                    })
                    .unwrap_or_else(|| hash_function.recommended_by_length());

                self.generate_hmac_key(hash_function, Some(key_len))
            }

            // For RSA, key length is handled by the RsaKeyLength enum
            _ => self.generate_key(algorithm),
        }
    }
}

impl Default for KeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::Algorithm;

    #[test]
    fn test_generate_hmac_key() {
        let builder = KeyBuilder::new();

        // Test HS256
        let result = builder.generate_key(Algorithm::HS256);
        assert!(result.is_ok());
        let key_pair = result.unwrap();
        assert!(key_pair.public_key.is_none());
        assert!(!key_pair.private_key.is_empty());

        // Test HS512
        let result = builder.generate_key(Algorithm::HS512);
        assert!(result.is_ok());
        let key_pair = result.unwrap();
        assert!(key_pair.public_key.is_none());
        assert!(!key_pair.private_key.is_empty());
    }

    #[test]
    fn test_generate_rsa_key() {
        let builder = KeyBuilder::new();

        // Test RS256
        let result = builder.generate_key(Algorithm::RS256);
        assert!(result.is_ok());
        let key_pair = result.unwrap();
        assert!(key_pair.public_key.is_some());
        assert!(!key_pair.private_key.is_empty());

        // The private key should be in PEM format
        let private_key_str = String::from_utf8_lossy(&key_pair.private_key);
        assert!(private_key_str.contains("PRIVATE KEY"));

        // The public key should be in PEM format
        let public_key_str = String::from_utf8_lossy(key_pair.public_key.as_ref().unwrap());
        assert!(public_key_str.contains("PUBLIC KEY"));
    }

    #[test]
    fn test_generate_key_with_length() {
        let builder = KeyBuilder::new();

        // Test with custom key length for HMAC
        let result = builder.generate_key_with_length(Algorithm::HS256, Some(32));
        assert!(result.is_ok());
        let key_pair = result.unwrap();
        assert_eq!(key_pair.private_key.len(), 32);

        // Test with default key length
        let result = builder.generate_key_with_length(Algorithm::HS256, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unsupported_algorithms() {
        let builder = KeyBuilder::new();

        // Test ES256 (not implemented)
        let result = builder.generate_key(Algorithm::ES256);
        assert!(result.is_err());

        // Test EdDSA (not implemented)
        let result = builder.generate_key(Algorithm::EdDSA);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_private_key_pem() {
        // Generate a test RSA private key
        let builder = KeyBuilder::new();
        let key_pair = builder.generate_key(Algorithm::RS256).unwrap();

        // Convert private key bytes to string
        let private_key_pem = String::from_utf8(key_pair.private_key)
            .expect("Failed to convert private key to string");

        // Load the private key and extract public key
        let loaded_pair = KeyBuilder::from_private_key_pem(&private_key_pem)
            .expect("Failed to load private key from PEM");

        // Verify we got both keys
        assert!(!loaded_pair.private_key.is_empty());
        assert!(loaded_pair.public_key.is_some());

        // The public key should be in PEM format
        let public_key_pem = loaded_pair.public_key.unwrap();
        let public_key_str = String::from_utf8_lossy(&public_key_pem);
        assert!(public_key_str.contains("PUBLIC KEY"));
    }

    #[test]
    fn test_from_private_key_pem_invalid() {
        // Test with an invalid PEM string
        let result = KeyBuilder::from_private_key_pem("invalid pem string");
        assert!(result.is_err());

        // Test with empty string
        let result = KeyBuilder::from_private_key_pem("");
        assert!(result.is_err());
    }
}
