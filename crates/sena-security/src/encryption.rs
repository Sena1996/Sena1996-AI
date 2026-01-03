use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sena_core::{Error, Result};
use sha2::{Digest, Sha256};

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    pub fn new(key: &[u8; KEY_SIZE]) -> Self {
        Self {
            cipher: Aes256Gcm::new(key.into()),
        }
    }

    pub fn from_password(password: &str) -> Self {
        let key = derive_key(password);
        Self::new(&key)
    }

    pub fn generate_key() -> [u8; KEY_SIZE] {
        let mut key = [0u8; KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        key
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| Error::security(format!("encryption failed: {}", e)))?;

        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend(ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < NONCE_SIZE {
            return Err(Error::security("ciphertext too short"));
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| Error::security(format!("decryption failed: {}", e)))
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<String> {
        let encrypted = self.encrypt(plaintext.as_bytes())?;
        Ok(BASE64.encode(encrypted))
    }

    pub fn decrypt_string(&self, ciphertext: &str) -> Result<String> {
        let encrypted = BASE64
            .decode(ciphertext)
            .map_err(|e| Error::security(format!("base64 decode failed: {}", e)))?;
        let decrypted = self.decrypt(&encrypted)?;
        String::from_utf8(decrypted)
            .map_err(|e| Error::security(format!("utf8 decode failed: {}", e)))
    }
}

fn derive_key(password: &str) -> [u8; KEY_SIZE] {
    const SALT: &[u8] = b"sena-pbkdf2-salt-v2";
    const ITERATIONS: u32 = 100_000;

    let mut key = [0u8; KEY_SIZE];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), SALT, ITERATIONS, &mut key);
    key
}

pub fn hash_string(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn encrypt_string(plaintext: &str, password: &str) -> Result<String> {
    let encryptor = Encryptor::from_password(password);
    encryptor.encrypt_string(plaintext)
}

pub fn decrypt_string(ciphertext: &str, password: &str) -> Result<String> {
    let encryptor = Encryptor::from_password(password);
    encryptor.decrypt_string(ciphertext)
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let encryptor = Encryptor::from_password("test-password");
        let plaintext = "Hello, World!";
        let encrypted = encryptor.encrypt_string(plaintext).unwrap();
        let decrypted = encryptor.decrypt_string(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_hash_string() {
        let hash1 = hash_string("test");
        let hash2 = hash_string("test");
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }
}
