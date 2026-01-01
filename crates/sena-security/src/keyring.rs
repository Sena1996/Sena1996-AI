use sena_core::{Error, Result};
use std::collections::HashMap;
use parking_lot::RwLock;

pub struct KeyManager {
    service: String,
    cache: RwLock<HashMap<String, String>>,
}

impl KeyManager {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        if let Some(cached) = self.cache.read().get(key) {
            return Ok(Some(cached.clone()));
        }

        match keyring::Entry::new(&self.service, key) {
            Ok(entry) => match entry.get_password() {
                Ok(password) => {
                    self.cache.write().insert(key.to_string(), password.clone());
                    Ok(Some(password))
                }
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(Error::security(format!("keyring error: {}", e))),
            },
            Err(e) => Err(Error::security(format!("keyring entry error: {}", e))),
        }
    }

    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        match keyring::Entry::new(&self.service, key) {
            Ok(entry) => {
                entry
                    .set_password(value)
                    .map_err(|e| Error::security(format!("keyring set error: {}", e)))?;
                self.cache.write().insert(key.to_string(), value.to_string());
                Ok(())
            }
            Err(e) => Err(Error::security(format!("keyring entry error: {}", e))),
        }
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        match keyring::Entry::new(&self.service, key) {
            Ok(entry) => {
                match entry.delete_credential() {
                    Ok(()) => {}
                    Err(keyring::Error::NoEntry) => {}
                    Err(e) => return Err(Error::security(format!("keyring delete error: {}", e))),
                }
                self.cache.write().remove(key);
                Ok(())
            }
            Err(e) => Err(Error::security(format!("keyring entry error: {}", e))),
        }
    }

    pub fn exists(&self, key: &str) -> Result<bool> {
        self.get(key).map(|opt| opt.is_some())
    }

    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    pub fn list_cached(&self) -> Vec<String> {
        self.cache.read().keys().cloned().collect()
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new("sena")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_cache() {
        let manager = KeyManager::new("test-sena");
        assert!(manager.list_cached().is_empty());
    }
}
