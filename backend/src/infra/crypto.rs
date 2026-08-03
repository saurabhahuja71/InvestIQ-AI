//! AES-256-GCM helpers for optional field-level encryption.

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;

use crate::error::{AppError, AppResult};

pub struct AesCipher {
    cipher: Aes256Gcm,
}

impl AesCipher {
    pub fn from_base64_key(key_b64: &str) -> AppResult<Self> {
        let key = B64
            .decode(key_b64)
            .map_err(|e| AppError::Internal(format!("aes key b64: {e}")))?;
        if key.len() != 32 {
            return Err(AppError::Internal("AES key must be 32 bytes".into()));
        }
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| AppError::Internal(format!("aes init: {e}")))?;
        Ok(Self { cipher })
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> AppResult<String> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| AppError::Internal(format!("encrypt: {e}")))?;
        let mut out = nonce_bytes.to_vec();
        out.extend(ciphertext);
        Ok(B64.encode(out))
    }

    pub fn decrypt(&self, payload_b64: &str) -> AppResult<Vec<u8>> {
        let raw = B64
            .decode(payload_b64)
            .map_err(|_| AppError::BadRequest("invalid ciphertext".into()))?;
        if raw.len() < 13 {
            return Err(AppError::BadRequest("ciphertext too short".into()));
        }
        let (nonce_bytes, ct) = raw.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ct)
            .map_err(|_| AppError::BadRequest("decrypt failed".into()))
    }

    /// Encrypt UTF-8 text; returns base64 payload.
    pub fn encrypt_str(&self, plaintext: &str) -> AppResult<String> {
        self.encrypt(plaintext.as_bytes())
    }

    /// Decrypt to UTF-8 string.
    pub fn decrypt_str(&self, payload_b64: &str) -> AppResult<String> {
        let bytes = self.decrypt(payload_b64)?;
        String::from_utf8(bytes).map_err(|_| AppError::BadRequest("invalid utf-8 plaintext".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::STANDARD;

    fn test_cipher() -> AesCipher {
        let key = STANDARD.encode([7u8; 32]);
        AesCipher::from_base64_key(&key).expect("cipher")
    }

    #[test]
    fn roundtrip() {
        let c = test_cipher();
        let enc = c.encrypt_str("secret-note").unwrap();
        let dec = c.decrypt_str(&enc).unwrap();
        assert_eq!(dec, "secret-note");
    }

    #[test]
    fn bad_key_length() {
        let key = STANDARD.encode([1u8; 16]);
        assert!(AesCipher::from_base64_key(&key).is_err());
    }

    #[test]
    fn decrypt_garbage_is_bad_request() {
        let c = test_cipher();
        let err = c.decrypt("not-valid-cipher").unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
