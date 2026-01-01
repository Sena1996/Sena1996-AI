use keyring::Entry;
use sena_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::OAuthToken;

const SERVICE_NAME: &str = "sena-ai";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredCredentials {
    api_key: Option<String>,
    oauth_token: Option<OAuthToken>,
}

pub struct TokenStore {
    use_keyring: bool,
    fallback_dir: PathBuf,
}

impl TokenStore {
    pub fn new() -> Result<Self> {
        let fallback_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sena")
            .join("credentials");

        std::fs::create_dir_all(&fallback_dir)
            .map_err(|e| Error::internal(format!("Failed to create credentials directory: {}", e)))?;

        let use_keyring = Self::test_keyring();

        Ok(Self {
            use_keyring,
            fallback_dir,
        })
    }

    fn test_keyring() -> bool {
        let test_entry = Entry::new(SERVICE_NAME, "test");
        match test_entry {
            Ok(entry) => {
                let _ = entry.set_password("test");
                let _ = entry.delete_credential();
                true
            }
            Err(_) => false,
        }
    }

    pub fn store_api_key(&self, provider: &str, key: &str) -> Result<()> {
        if self.use_keyring {
            self.store_keyring(&format!("{}_api_key", provider), key)?;
        } else {
            self.store_file(provider, &StoredCredentials {
                api_key: Some(key.to_string()),
                oauth_token: None,
            })?;
        }
        Ok(())
    }

    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>> {
        if self.use_keyring {
            self.get_keyring(&format!("{}_api_key", provider))
        } else {
            let creds = self.get_file(provider)?;
            Ok(creds.and_then(|c| c.api_key))
        }
    }

    pub fn store_oauth_token(&self, provider: &str, token: &OAuthToken) -> Result<()> {
        let token_json = serde_json::to_string(token)
            .map_err(|e| Error::internal(format!("Failed to serialize token: {}", e)))?;

        if self.use_keyring {
            self.store_keyring(&format!("{}_oauth", provider), &token_json)?;
        } else {
            let mut creds = self.get_file(provider)?.unwrap_or(StoredCredentials {
                api_key: None,
                oauth_token: None,
            });
            creds.oauth_token = Some(token.clone());
            self.store_file(provider, &creds)?;
        }
        Ok(())
    }

    pub fn get_oauth_token(&self, provider: &str) -> Result<Option<OAuthToken>> {
        if self.use_keyring {
            let token_json = self.get_keyring(&format!("{}_oauth", provider))?;
            if let Some(json) = token_json {
                let token: OAuthToken = serde_json::from_str(&json)
                    .map_err(|e| Error::internal(format!("Failed to parse token: {}", e)))?;
                return Ok(Some(token));
            }
            Ok(None)
        } else {
            let creds = self.get_file(provider)?;
            Ok(creds.and_then(|c| c.oauth_token))
        }
    }

    pub fn remove_credentials(&self, provider: &str) -> Result<()> {
        if self.use_keyring {
            let _ = self.delete_keyring(&format!("{}_api_key", provider));
            let _ = self.delete_keyring(&format!("{}_oauth", provider));
        }

        let path = self.fallback_dir.join(format!("{}.json", provider));
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| Error::internal(format!("Failed to remove credentials: {}", e)))?;
        }

        Ok(())
    }

    fn store_keyring(&self, key: &str, value: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|e| Error::internal(format!("Keyring error: {}", e)))?;

        entry.set_password(value)
            .map_err(|e| Error::internal(format!("Failed to store in keyring: {}", e)))?;

        Ok(())
    }

    fn get_keyring(&self, key: &str) -> Result<Option<String>> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|e| Error::internal(format!("Keyring error: {}", e)))?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::internal(format!("Failed to read from keyring: {}", e))),
        }
    }

    fn delete_keyring(&self, key: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|e| Error::internal(format!("Keyring error: {}", e)))?;

        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::internal(format!("Failed to delete from keyring: {}", e))),
        }
    }

    fn store_file(&self, provider: &str, creds: &StoredCredentials) -> Result<()> {
        let path = self.fallback_dir.join(format!("{}.json", provider));

        let json = serde_json::to_string_pretty(creds)
            .map_err(|e| Error::internal(format!("Failed to serialize credentials: {}", e)))?;

        let encrypted = crate::encryption::encrypt_string(&json, provider)?;

        std::fs::write(&path, encrypted)
            .map_err(|e| Error::internal(format!("Failed to write credentials file: {}", e)))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&path, perms)?;
        }

        Ok(())
    }

    fn get_file(&self, provider: &str) -> Result<Option<StoredCredentials>> {
        let path = self.fallback_dir.join(format!("{}.json", provider));

        if !path.exists() {
            return Ok(None);
        }

        let encrypted = std::fs::read_to_string(&path)
            .map_err(|e| Error::internal(format!("Failed to read credentials file: {}", e)))?;

        let json = crate::encryption::decrypt_string(&encrypted, provider)?;

        let creds: StoredCredentials = serde_json::from_str(&json)
            .map_err(|e| Error::internal(format!("Failed to parse credentials: {}", e)))?;

        Ok(Some(creds))
    }

    pub fn list_stored_providers(&self) -> Result<Vec<String>> {
        let mut providers = Vec::new();

        for entry in std::fs::read_dir(&self.fallback_dir)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Some(name) = path.file_stem() {
                        providers.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        Ok(providers)
    }
}
