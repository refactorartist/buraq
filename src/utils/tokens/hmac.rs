use hmac::{Hmac, Mac};
use rand::{RngCore, rngs::OsRng};
use sha2::{Sha256, Sha512};
use sha3::{Sha3_256, Sha3_512};

// Type aliases for HMAC implementations
type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;
type HmacSha3_256 = Hmac<Sha3_256>;
type HmacSha3_512 = Hmac<Sha3_512>;

// Helper macro to create HMAC instances with fully qualified syntax
macro_rules! create_hmac {
    ($hmac_type:ty, $key:expr) => {
        <$hmac_type as Mac>::new_from_slice($key).map_err(|e| e.to_string())?
    };
}

#[derive(Debug)]
pub struct HmacKey {
    key: Vec<u8>,
    hash_function: HmacHashFunction,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HmacHashFunction {
    Sha256,
    Sha512,
    Sha3_256,
    Sha3_512,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HmacKeyLength {
    B128,
    B192,
    B256,
    B384,
    B512,
}

impl HmacKeyLength {
    pub fn as_bytes(&self) -> usize {
        match self {
            HmacKeyLength::B128 => 16,
            HmacKeyLength::B192 => 24,
            HmacKeyLength::B256 => 32,
            HmacKeyLength::B384 => 48,
            HmacKeyLength::B512 => 64,
        }
    }

    pub fn as_bits(&self) -> usize {
        self.as_bytes() * 8
    }
}

impl HmacHashFunction {
    pub fn name(&self) -> &'static str {
        match self {
            HmacHashFunction::Sha256 => "SHA256",
            HmacHashFunction::Sha512 => "SHA512",
            HmacHashFunction::Sha3_256 => "SHA3_256",
            HmacHashFunction::Sha3_512 => "SHA3_512",
        }
    }

    pub fn recommended_by_length(&self) -> HmacKeyLength {
        match self {
            HmacHashFunction::Sha256 => HmacKeyLength::B256,
            HmacHashFunction::Sha512 => HmacKeyLength::B512,
            HmacHashFunction::Sha3_256 => HmacKeyLength::B256,
            HmacHashFunction::Sha3_512 => HmacKeyLength::B512,
        }
    }

    pub fn output_size_bytes(&self) -> usize {
        match self {
            HmacHashFunction::Sha256 => 32,
            HmacHashFunction::Sha512 => 64,
            HmacHashFunction::Sha3_256 => 32,
            HmacHashFunction::Sha3_512 => 64,
        }
    }

    pub fn sign(&self, key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
        match self {
            HmacHashFunction::Sha256 => {
                let mut mac = create_hmac!(HmacSha256, key);
                Mac::update(&mut mac, data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            HmacHashFunction::Sha512 => {
                let mut mac = create_hmac!(HmacSha512, key);
                Mac::update(&mut mac, data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            HmacHashFunction::Sha3_256 => {
                let mut mac = create_hmac!(HmacSha3_256, key);
                Mac::update(&mut mac, data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
            HmacHashFunction::Sha3_512 => {
                let mut mac = create_hmac!(HmacSha3_512, key);
                Mac::update(&mut mac, data);
                Ok(mac.finalize().into_bytes().to_vec())
            }
        }
    }

    pub fn verify(&self, key: &[u8], data: &[u8], signature: &[u8]) -> Result<bool, String> {
        let computed = self.sign(key, data)?;
        Ok(computed == signature)
    }
}

impl HmacKey {
    pub fn new(key: &[u8], hash_function: HmacHashFunction) -> Self {
        Self {
            key: key.to_vec(),
            hash_function,
        }
    }

    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        self.hash_function.sign(&self.key, data)
    }

    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, String> {
        self.hash_function.verify(&self.key, data, signature)
    }
}

pub fn generate_hmac_key(
    hash_function: HmacHashFunction,
    key_length: HmacKeyLength,
) -> Result<HmacKey, String> {
    let key_length = key_length.as_bytes();
    let mut key_bytes = vec![0u8; key_length];
    let mut rng = OsRng;
    rng.fill_bytes(&mut key_bytes);
    let hmac_key = HmacKey::new(&key_bytes, hash_function);
    Ok(hmac_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_MESSAGE: &[u8] = b"test message for HMAC";
    const TEST_KEY: &[u8] = b"test-key-123";

    #[test]
    fn test_hash_function_name() {
        assert_eq!(HmacHashFunction::Sha256.name(), "SHA256");
        assert_eq!(HmacHashFunction::Sha512.name(), "SHA512");
        assert_eq!(HmacHashFunction::Sha3_256.name(), "SHA3_256");
        assert_eq!(HmacHashFunction::Sha3_512.name(), "SHA3_512");
    }

    #[test]
    fn test_hash_function_recommended_key_length() {
        assert_eq!(
            HmacHashFunction::Sha256.recommended_by_length(),
            HmacKeyLength::B256
        );
        assert_eq!(
            HmacHashFunction::Sha512.recommended_by_length(),
            HmacKeyLength::B512
        );
        assert_eq!(
            HmacHashFunction::Sha3_256.recommended_by_length(),
            HmacKeyLength::B256
        );
        assert_eq!(
            HmacHashFunction::Sha3_512.recommended_by_length(),
            HmacKeyLength::B512
        );
    }

    #[test]
    fn test_hash_function_output_size() {
        assert_eq!(HmacHashFunction::Sha256.output_size_bytes(), 32);
        assert_eq!(HmacHashFunction::Sha512.output_size_bytes(), 64);
        assert_eq!(HmacHashFunction::Sha3_256.output_size_bytes(), 32);
        assert_eq!(HmacHashFunction::Sha3_512.output_size_bytes(), 64);
    }

    #[test]
    fn test_hmac_key_length_bytes() {
        assert_eq!(HmacKeyLength::B128.as_bytes(), 16);
        assert_eq!(HmacKeyLength::B192.as_bytes(), 24);
        assert_eq!(HmacKeyLength::B256.as_bytes(), 32);
        assert_eq!(HmacKeyLength::B384.as_bytes(), 48);
        assert_eq!(HmacKeyLength::B512.as_bytes(), 64);
    }

    #[test]
    fn test_hmac_key_length_bits() {
        assert_eq!(HmacKeyLength::B128.as_bits(), 128);
        assert_eq!(HmacKeyLength::B192.as_bits(), 192);
        assert_eq!(HmacKeyLength::B256.as_bits(), 256);
        assert_eq!(HmacKeyLength::B384.as_bits(), 384);
        assert_eq!(HmacKeyLength::B512.as_bits(), 512);
    }

    #[test]
    fn test_hmac_sign_verify() {
        let test_cases = [
            (
                b"key".as_slice(),
                b"The quick brown fox jumps over the lazy dog".as_slice(),
                HmacHashFunction::Sha256,
            ),
            (
                b"key".as_slice(),
                b"The quick brown fox jumps over the lazy dog".as_slice(),
                HmacHashFunction::Sha512,
            ),
            (
                b"key".as_slice(),
                b"The quick brown fox jumps over the lazy dog".as_slice(),
                HmacHashFunction::Sha3_256,
            ),
            (
                b"key".as_slice(),
                b"The quick brown fox jumps over the lazy dog".as_slice(),
                HmacHashFunction::Sha3_512,
            ),
        ];

        for &(key, data, hash_func) in &test_cases {
            // Test sign
            let signature = hash_func.sign(key, data).expect("Signing failed");

            // Test verify with correct signature
            let is_valid = hash_func
                .verify(key, data, &signature)
                .expect("Verification failed");
            assert!(is_valid, "Verification failed for {:?}", hash_func);

            // Test verify with incorrect signature
            let mut bad_signature = signature.clone();
            if !bad_signature.is_empty() {
                bad_signature[0] = bad_signature[0].wrapping_add(1);
                let is_valid = hash_func
                    .verify(key, data, &bad_signature)
                    .expect("Verification failed");
                assert!(
                    !is_valid,
                    "Verification should fail with incorrect signature for {:?}",
                    hash_func
                );
            }
        }
    }

    #[test]
    fn test_hmac_with_empty_key() {
        let empty_key = vec![];
        let hash_funcs = [
            HmacHashFunction::Sha256,
            HmacHashFunction::Sha512,
            HmacHashFunction::Sha3_256,
            HmacHashFunction::Sha3_512,
        ];

        for &hash_func in hash_funcs.iter() {
            let key = HmacKey::new(&empty_key, hash_func);
            let signature = key.sign(TEST_MESSAGE).expect("Signing failed");
            assert!(
                key.verify(TEST_MESSAGE, &signature)
                    .expect("Verification failed"),
                "HMAC verification with empty key failed for {:?}",
                hash_func
            );
        }
    }

    #[test]
    fn test_hmac_with_empty_message() {
        let empty_message = b"";
        let hash_funcs = [
            HmacHashFunction::Sha256,
            HmacHashFunction::Sha512,
            HmacHashFunction::Sha3_256,
            HmacHashFunction::Sha3_512,
        ];

        for &hash_func in hash_funcs.iter() {
            let key = HmacKey::new(TEST_KEY, hash_func);
            let signature = key.sign(empty_message).expect("Signing failed");
            assert!(
                key.verify(empty_message, &signature)
                    .expect("Verification failed"),
                "HMAC verification with empty message failed for {:?}",
                hash_func
            );
        }
    }

    #[test]
    fn test_generate_hmac_key() {
        let hash_funcs = [
            HmacHashFunction::Sha256,
            HmacHashFunction::Sha512,
            HmacHashFunction::Sha3_256,
            HmacHashFunction::Sha3_512,
        ];

        for &hash_func in hash_funcs.iter() {
            let key_length = hash_func.recommended_by_length();
            let key =
                generate_hmac_key(hash_func, key_length).expect("Failed to generate HMAC key");

            // Verify the generated key has the correct length
            assert_eq!(
                key.key.len(),
                key_length.as_bytes(),
                "Generated key has incorrect length for {:?}",
                hash_func
            );

            // Test that the key can be used to sign and verify
            let signature = key
                .sign(TEST_MESSAGE)
                .expect("Signing with generated key failed");
            assert!(
                key.verify(TEST_MESSAGE, &signature)
                    .expect("Verification with generated key failed"),
                "HMAC verification with generated key failed for {:?}",
                hash_func
            );
        }
    }
}
