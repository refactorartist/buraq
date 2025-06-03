use anyhow::{Error, Context};
use openssl::rsa::Rsa;
use openssl::pkey::{Private, Public};
use openssl::pkey::PKey;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RsaKeyLength {
    B2048,
    B3072,
    B4096,
    B8192,
}

impl RsaKeyLength {
    pub fn as_bits(&self) -> usize {
        match self {
            RsaKeyLength::B2048 => 2048,
            RsaKeyLength::B3072 => 3072,
            RsaKeyLength::B4096 => 4096,
            RsaKeyLength::B8192 => 8192,
        }
    }

    pub fn from_bits(bits: u32) -> Option<Self> {
        match bits {
            2048 => Some(RsaKeyLength::B2048),
            3072 => Some(RsaKeyLength::B3072),
            4096 => Some(RsaKeyLength::B4096),
            8192 => Some(RsaKeyLength::B8192),
            _ => None,
        }
    }

    pub fn all() -> &'static [Self] {
        &[
            RsaKeyLength::B2048,
            RsaKeyLength::B3072,
            RsaKeyLength::B4096,
            RsaKeyLength::B8192,
        ]
    }
}


pub type RsaPrivateKey = PKey<Private>;
pub type RsaPublicKey = PKey<Public>;

pub fn generate_rsa_key_pair(rsa_key_length: RsaKeyLength) -> Result<(RsaPrivateKey, RsaPublicKey), Error> {
    // Generate RSA key pair
    let rsa = Rsa::generate(rsa_key_length.as_bits() as u32)
        .context("Failed to generate RSA key")?;
    
    // Create private key
    let private_key = PKey::from_rsa(rsa)
        .context("Failed to create private key")?;
    
    // Create public key from private key
    let public_key = private_key.public_key_to_pem()
        .context("Failed to extract public key")?;
    let public_key = PKey::public_key_from_pem(&public_key)
        .context("Failed to create public key")?;
    
    Ok((private_key, public_key))
}



#[cfg(test)]
mod tests {
    use openssl::rsa::Padding;

    use super::*;

    #[test]
    fn test_rsa_key_length_as_bits() {
        assert_eq!(RsaKeyLength::B2048.as_bits(), 2048);
        assert_eq!(RsaKeyLength::B3072.as_bits(), 3072);
        assert_eq!(RsaKeyLength::B4096.as_bits(), 4096);
        assert_eq!(RsaKeyLength::B8192.as_bits(), 8192);
    }

    #[test]
    fn test_rsa_key_length_from_bits() {
        assert_eq!(RsaKeyLength::from_bits(2048), Some(RsaKeyLength::B2048));
        assert_eq!(RsaKeyLength::from_bits(3072), Some(RsaKeyLength::B3072));
        assert_eq!(RsaKeyLength::from_bits(4096), Some(RsaKeyLength::B4096));
        assert_eq!(RsaKeyLength::from_bits(8192), Some(RsaKeyLength::B8192));

    }

    #[test]
    fn test_rsa_key_length_all() {
        let all = RsaKeyLength::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&RsaKeyLength::B2048));
        assert!(all.contains(&RsaKeyLength::B3072));
        assert!(all.contains(&RsaKeyLength::B4096));
        assert!(all.contains(&RsaKeyLength::B8192));
    }

    #[test]
    fn test_generate_rsa_key_pair_success() {
        // Test with all supported key lengths
        for &key_length in RsaKeyLength::all() {
            let start = std::time::Instant::now();
            let result = generate_rsa_key_pair(key_length);
            let duration = start.elapsed();
            println!("generate_rsa_key_pair({:?}) took {:?}", key_length, duration);
            assert!(result.is_ok(), "Failed to generate key pair for {:?}", key_length);
            
            let (private_key, public_key) = result.unwrap();
            
            // Verify the key sizes match what was requested
            let rsa_private = private_key.rsa().unwrap();
            let rsa_public = public_key.rsa().unwrap();
            
            // Check key sizes (OpenSSL rounds up to the nearest byte)
            let private_bits = rsa_private.size() * 8;
            let public_bits = rsa_public.size() * 8;
            let expected_bits = key_length.as_bits() as u32;
            assert!(private_bits >= expected_bits && private_bits < expected_bits + 8,
                   "Private key size {} doesn't match expected {}", private_bits, expected_bits);
            assert!(public_bits >= expected_bits && public_bits < expected_bits + 8,
                   "Public key size {} doesn't match expected {}", public_bits, expected_bits);
            
            // Test encryption/decryption with the generated keys
            test_encryption_decryption(private_key, public_key);
        }
    }
    
    #[test]
    fn test_generate_rsa_key_pair_different_keys() {
        // Generate two key pairs and verify they're different
        let (priv1, pub1) = generate_rsa_key_pair(RsaKeyLength::B2048).unwrap();
        let (_priv2, pub2) = generate_rsa_key_pair(RsaKeyLength::B2048).unwrap();
        
        // Get the raw public key data for comparison
        let pub1_data = pub1.public_key_to_der().unwrap();
        let pub2_data = pub2.public_key_to_der().unwrap();
        
        // Public keys should be different between generations
        assert_ne!(pub1_data, pub2_data, "Public keys should be different between generations");
        
        // The public key should match the one derived from the private key
        let derived_pub_data = priv1.public_key_to_der().unwrap();
        assert_eq!(
            pub1_data, derived_pub_data,
            "Public key should match the one derived from private key"
        );
    }
    
    // Helper function to test encryption/decryption with a key pair
    fn test_encryption_decryption(private_key: RsaPrivateKey, public_key: RsaPublicKey) {
        // Test data to encrypt
        let data = b"Hello, RSA encryption test!";
        
        // Encrypt with public key
        let rsa_public = public_key.rsa().unwrap();
        let mut encrypted = vec![0; rsa_public.size() as usize];
        let encrypted_len = rsa_public
            .public_encrypt(data, &mut encrypted, Padding::PKCS1)
            .expect("Failed to encrypt data");
            
        // Decrypt with private key
        let rsa_private = private_key.rsa().unwrap();
        let mut decrypted = vec![0; rsa_private.size() as usize];
        let decrypted_len = rsa_private
            .private_decrypt(&encrypted[..encrypted_len], &mut decrypted, Padding::PKCS1)
            .expect("Failed to decrypt data");
            
        // Verify the decrypted data matches the original
        assert_eq!(&decrypted[..decrypted_len], data);
    }
}
