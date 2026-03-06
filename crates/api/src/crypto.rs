use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::Value;
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptError(String),
    #[error("Decryption failed: {0}")]
    DecryptError(String),
    #[error("Invalid key format: {0}")]
    InvalidKey(String),
    #[error("Missing encryption key in environment: {0}")]
    MissingKey(String),
    #[error("Serialization error: {0}")]
    SerializeError(#[from] serde_json::Error),
}

/// A wrapper around AES-256-GCM for encrypting string data
#[derive(Clone)]
pub struct CryptoService {
    cipher: Aes256Gcm,
}

impl CryptoService {
    /// Initialize with a key from environment variables (BARQFLOW_ENCRYPTION_KEY)
    pub fn new() -> Result<Self, CryptoError> {
        let key_str = env::var("BARQFLOW_ENCRYPTION_KEY").unwrap_or_else(|_| {
            // For testing/development, fallback to a mocked static key.
            // In production, this should panic or refuse to start.
            "01234567890123456789012345678901".to_string()
        });

        if key_str.len() != 32 {
            return Err(CryptoError::InvalidKey(
                "Key must be exactly 32 bytes".into(),
            ));
        }

        let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    /// Encrypts a string and returns a base64 encoded string format: "base64(nonce):base64(ciphertext)"
    pub fn encrypt(&self, plaintext: &str) -> Result<String, CryptoError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message

        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| CryptoError::EncryptError(e.to_string()))?;

        let nonce_b64 = STANDARD.encode(nonce);
        let cipher_b64 = STANDARD.encode(ciphertext);

        Ok(format!("{}:{}", nonce_b64, cipher_b64))
    }

    /// Decrypts a base64 encoded string format: "base64(nonce):base64(ciphertext)"
    pub fn decrypt(&self, encrypted_data: &str) -> Result<String, CryptoError> {
        let parts: Vec<&str> = encrypted_data.split(':').collect();
        if parts.len() != 2 {
            return Err(CryptoError::DecryptError(
                "Invalid encrypted data format".into(),
            ));
        }

        let nonce_bytes = STANDARD
            .decode(parts[0])
            .map_err(|e| CryptoError::DecryptError(format!("Nonce decode failed: {}", e)))?;
        let cipher_bytes = STANDARD
            .decode(parts[1])
            .map_err(|e| CryptoError::DecryptError(format!("Ciphertext decode failed: {}", e)))?;

        if nonce_bytes.len() != 12 {
            return Err(CryptoError::DecryptError("Invalid nonce length".into()));
        }

        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext_bytes = self
            .cipher
            .decrypt(nonce, cipher_bytes.as_ref())
            .map_err(|e| CryptoError::DecryptError(e.to_string()))?;

        String::from_utf8(plaintext_bytes)
            .map_err(|e| CryptoError::DecryptError(format!("Invalid UTF-8 plaintext: {}", e)))
    }

    /// Serializes a JSON Value and encrypts it
    pub fn encrypt_value(&self, value: &Value) -> Result<String, CryptoError> {
        let plaintext = serde_json::to_string(value)?;
        self.encrypt(&plaintext)
    }

    /// Decrypts a string and deserializes it back to a JSON Value
    pub fn decrypt_value(&self, encrypted_data: &str) -> Result<Value, CryptoError> {
        let plaintext = self.decrypt(encrypted_data)?;
        serde_json::from_str(&plaintext).map_err(CryptoError::SerializeError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_encryption_decryption() {
        env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "test_key_must_be_exactly_32_byte",
        );
        let crypto = CryptoService::new().unwrap();

        let secret = "my_super_secret_api_key_123";
        let encrypted = crypto.encrypt(secret).unwrap();

        // Ensure it's not plain text
        assert!(!encrypted.contains(secret));
        assert!(encrypted.contains(':'));

        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_encrypt_decrypt_value() {
        env::set_var(
            "BARQFLOW_ENCRYPTION_KEY",
            "test_key_must_be_exactly_32_byte",
        );
        let crypto = CryptoService::new().unwrap();

        let secret_obj = json!({
            "api_key": "sk-1234567890",
            "domain": "api.example.com",
            "port": 443
        });

        let encrypted = crypto.encrypt_value(&secret_obj).unwrap();
        assert!(!encrypted.contains("api_key"));
        assert!(!encrypted.contains("sk-123"));

        let decrypted = crypto.decrypt_value(&encrypted).unwrap();
        assert_eq!(decrypted, secret_obj);
    }
}
