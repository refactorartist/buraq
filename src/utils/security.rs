use anyhow::{Context, Error, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use mongodb::bson::uuid::Uuid;
use rand::{RngCore, rngs::OsRng};
use std::env;

use crate::utils::tokens::hmac::{HmacHashFunction, HmacKey};

/// SecretsManager provides encryption and decryption functionality
/// using a combination of a master key (from .env) and a resource-specific ID.
///
/// This ensures that each resource's data is encrypted with a unique key derived
/// from both the master key and the resource's ID.
#[derive(Debug)]
pub struct SecretsManager {
    master_key: Vec<u8>,
}

impl SecretsManager {
    /// Creates a new SecretsManager instance using the master key from .env
    pub fn new(load_dotenv: bool) -> Result<Self, Error> {
        if load_dotenv {
            dotenvy::dotenv().ok();
        }
        let master_key = env::var("BURAQ_MASTER_KEY")
            .context("BURAQ_MASTER_KEY not found in environment variables")?;

        Ok(Self {
            master_key: master_key.into_bytes(),
        })
    }

    /// Encrypts the provided text using the master key and resource ID
    ///
    /// # Arguments
    /// * `text` - The text to encrypt
    /// * `resource_id` - The resource ID (typically from a ServiceAccount)
    ///
    /// # Returns
    /// Base64-encoded encrypted data
    pub fn encrypt(&self, text: &str, resource_id: &Uuid) -> Result<String, Error> {
        // Generate a random initialization vector (IV)
        let mut iv = [0u8; 16];
        OsRng.fill_bytes(&mut iv);

        // Derive a resource-specific key using the master key and resource ID
        let resource_key = self.derive_resource_key(resource_id)?;

        // Encrypt the data
        let encrypted = self.encrypt_data(text.as_bytes(), &resource_key, &iv)?;

        // Combine IV and encrypted data
        let mut result = Vec::with_capacity(iv.len() + encrypted.len());
        result.extend_from_slice(&iv);
        result.extend_from_slice(&encrypted);

        // Return as Base64 string
        Ok(STANDARD.encode(result))
    }

    /// Decrypts the provided encrypted text using the master key and resource ID
    ///
    /// # Arguments
    /// * `encrypted_text` - Base64-encoded encrypted text
    /// * `resource_id` - The resource ID (typically from a ServiceAccount)
    ///
    /// # Returns
    /// The original decrypted text
    pub fn decrypt(&self, encrypted_text: &str, resource_id: &Uuid) -> Result<String, Error> {
        // Decode the Base64 input
        let encrypted_data = STANDARD
            .decode(encrypted_text)
            .context("Failed to decode Base64 input")?;

        // Ensure we have at least an IV (16 bytes) and some data
        if encrypted_data.len() <= 16 {
            return Err(Error::msg("Encrypted data is too short"));
        }

        // Extract IV and encrypted data
        let (iv, encrypted) = encrypted_data.split_at(16);

        // Derive resource-specific key
        let resource_key = self.derive_resource_key(resource_id)?;

        // Decrypt the data
        let decrypted = self.decrypt_data(encrypted, &resource_key, iv)?;

        // Convert to string
        String::from_utf8(decrypted).context("Failed to convert decrypted data to string")
    }

    /// Derives a resource-specific key using the master key and resource ID
    fn derive_resource_key(&self, resource_id: &Uuid) -> Result<Vec<u8>, Error> {
        let hmac_key = HmacKey::new(&self.master_key, HmacHashFunction::Sha256);
        let resource_id_bytes = resource_id.bytes().to_vec();

        hmac_key
            .sign(&resource_id_bytes)
            .map_err(|e| Error::msg(format!("Failed to derive resource key: {}", e)))
    }

    /// Encrypts data using XOR with the derived key (simple encryption for demonstration)
    /// In a production environment, consider using a more robust encryption algorithm
    fn encrypt_data(&self, data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, Error> {
        // Create a key stream by repeating the key
        let mut key_stream = Vec::with_capacity(data.len());
        let mut current_key = key.to_vec();

        // Mix the IV with the key using HMAC
        let hmac_key = HmacKey::new(&current_key, HmacHashFunction::Sha256);
        current_key = hmac_key
            .sign(iv)
            .map_err(|e| Error::msg(format!("HMAC error: {}", e)))?;

        // Generate a key stream long enough for the data
        while key_stream.len() < data.len() {
            key_stream.extend_from_slice(&current_key);

            // Evolve the key for the next block
            let hmac_key = HmacKey::new(&current_key, HmacHashFunction::Sha256);
            current_key = hmac_key
                .sign(&[0u8; 1])
                .map_err(|e| Error::msg(format!("HMAC error: {}", e)))?;
        }

        // Truncate to the exact length needed
        key_stream.truncate(data.len());

        // XOR the data with the key stream
        let encrypted: Vec<u8> = data
            .iter()
            .zip(key_stream.iter())
            .map(|(d, k)| d ^ k)
            .collect();

        Ok(encrypted)
    }

    /// Decrypts data using XOR with the derived key
    /// Since XOR is symmetric, the encryption and decryption process is identical
    fn decrypt_data(&self, data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, Error> {
        self.encrypt_data(data, key, iv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env;

    #[test]
    fn test_encrypt_decrypt() {
        temp_env::with_var("BURAQ_MASTER_KEY", Some("test-master-key-12345"), || {
            // Create a SecretsManager
            let secrets_manager = SecretsManager::new(false).unwrap();

            // Test data
            let original_text = "This is a secret message";
            let resource_id = Uuid::new();

            // Encrypt
            let encrypted = secrets_manager.encrypt(original_text, &resource_id).unwrap();

            // Decrypt
            let decrypted = secrets_manager.decrypt(&encrypted, &resource_id).unwrap();

            // Verify
            assert_eq!(original_text, decrypted);
        });
    }

    #[test]
    fn test_different_resource_ids() {
        temp_env::with_var("BURAQ_MASTER_KEY", Some("test-master-key-12345"), || {
            // Create a SecretsManager
            let secrets_manager = SecretsManager::new(false).unwrap();

            // Test data
            let original_text = "This is a secret message";
            let resource_id_1 = Uuid::new();
            let resource_id_2 = Uuid::new();

            // Encrypt with first resource ID
            let encrypted = secrets_manager
                .encrypt(original_text, &resource_id_1)
                .unwrap();

            // Try to decrypt with second resource ID (should fail or produce different output)
            let result = secrets_manager.decrypt(&encrypted, &resource_id_2);

            if let Ok(decrypted) = result {
                // If decryption succeeded, the result should be different
                assert_ne!(original_text, decrypted);
            } else {
                // Or decryption might fail, which is also acceptable
                assert!(result.is_err());
            }
        });
    }

    #[test]
    fn test_missing_env_var() {
        temp_env::with_var_unset("BURAQ_MASTER_KEY", || {
            let result = SecretsManager::new(false);
            dbg!(&result);
            assert!(result.is_err());
        });
    }
}
