use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const KEYRING_SERVICE: &str = "sena-hub";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialSource {
    Keychain,
    ConfigFile,
    Environment,
    NotSet,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StorageType {
    Keychain,
    ConfigFile,
}

#[derive(Debug, Clone, Serialize)]
pub struct CredentialStatus {
    pub provider_id: String,
    pub has_credential: bool,
    pub source: CredentialSource,
    pub is_valid: Option<bool>,
    pub can_import_from_env: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageOptions {
    pub keychain_available: bool,
    pub config_file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CredentialsConfig {
    credentials: HashMap<String, ProviderCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProviderCredentials {
    fields: HashMap<String, String>,
}

pub struct CredentialManager {
    keychain_available: bool,
    config_path: PathBuf,
}

impl CredentialManager {
    pub fn new() -> Self {
        let keychain_available = Self::test_keychain_access();
        let config_path = Self::get_config_path();

        Self {
            keychain_available,
            config_path,
        }
    }

    fn test_keychain_access() -> bool {
        let test_key = "sena_keychain_test";
        let test_value = "test_value_12345";

        let entry = match keyring::Entry::new(KEYRING_SERVICE, test_key) {
            Ok(e) => e,
            Err(_) => return false,
        };

        if entry.set_password(test_value).is_err() {
            return false;
        }

        let new_entry = match keyring::Entry::new(KEYRING_SERVICE, test_key) {
            Ok(e) => e,
            Err(_) => return false,
        };

        match new_entry.get_password() {
            Ok(retrieved) => {
                let _ = new_entry.delete_credential();
                retrieved == test_value
            }
            Err(_) => false,
        }
    }

    fn get_config_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".sena")
            .join("credentials.toml")
    }

    pub fn storage_options(&self) -> StorageOptions {
        StorageOptions {
            keychain_available: self.keychain_available,
            config_file_path: self.config_path.to_string_lossy().to_string(),
        }
    }

    pub fn store(
        &self,
        provider_id: &str,
        field_id: &str,
        value: &str,
        storage: StorageType,
    ) -> Result<(), String> {
        match storage {
            StorageType::Keychain => self.store_in_keychain(provider_id, field_id, value),
            StorageType::ConfigFile => self.store_in_config(provider_id, field_id, value),
        }
    }

    fn store_in_keychain(
        &self,
        provider_id: &str,
        field_id: &str,
        value: &str,
    ) -> Result<(), String> {
        let key = format!("{}_{}", provider_id, field_id);

        let entry = keyring::Entry::new(KEYRING_SERVICE, &key)
            .map_err(|e| format!("Keychain entry error: {}", e))?;

        entry
            .set_password(value)
            .map_err(|e| format!("Keychain store error: {}", e))
    }

    fn store_in_config(
        &self,
        provider_id: &str,
        field_id: &str,
        value: &str,
    ) -> Result<(), String> {
        let mut config = self.load_config()?;

        config
            .credentials
            .entry(provider_id.to_string())
            .or_default()
            .fields
            .insert(field_id.to_string(), value.to_string());

        self.save_config(&config)
    }

    pub fn get(
        &self,
        provider_id: &str,
        field_id: &str,
        env_var: Option<&str>,
    ) -> Option<(String, CredentialSource)> {
        if let Some(var_name) = env_var {
            if let Ok(value) = std::env::var(var_name) {
                if !value.is_empty() {
                    return Some((value, CredentialSource::Environment));
                }
            }
        }

        if let Some(value) = self.get_from_keychain(provider_id, field_id) {
            return Some((value, CredentialSource::Keychain));
        }

        if let Some(value) = self.get_from_config(provider_id, field_id) {
            return Some((value, CredentialSource::ConfigFile));
        }

        None
    }

    fn get_from_keychain(&self, provider_id: &str, field_id: &str) -> Option<String> {
        let key = format!("{}_{}", provider_id, field_id);
        let entry = keyring::Entry::new(KEYRING_SERVICE, &key).ok()?;
        entry.get_password().ok()
    }

    fn get_from_config(&self, provider_id: &str, field_id: &str) -> Option<String> {
        let config = self.load_config().ok()?;
        config
            .credentials
            .get(provider_id)?
            .fields
            .get(field_id)
            .cloned()
    }

    pub fn delete(&self, provider_id: &str, field_id: &str) -> Result<(), String> {
        self.delete_from_keychain(provider_id, field_id);
        self.delete_from_config(provider_id, field_id)?;
        Ok(())
    }

    fn delete_from_keychain(&self, provider_id: &str, field_id: &str) {
        let key = format!("{}_{}", provider_id, field_id);
        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, &key) {
            let _ = entry.delete_credential();
        }
    }

    fn delete_from_config(&self, provider_id: &str, field_id: &str) -> Result<(), String> {
        let mut config = self.load_config()?;

        if let Some(provider) = config.credentials.get_mut(provider_id) {
            provider.fields.remove(field_id);
            if provider.fields.is_empty() {
                config.credentials.remove(provider_id);
            }
        }

        self.save_config(&config)
    }

    pub fn get_credential_status(
        &self,
        provider_id: &str,
        field_id: &str,
        env_var: Option<&str>,
    ) -> CredentialStatus {
        let credential = self.get(provider_id, field_id, env_var);

        let can_import = env_var
            .map(|var| std::env::var(var).is_ok())
            .unwrap_or(false);

        match credential {
            Some((_, source)) => CredentialStatus {
                provider_id: provider_id.to_string(),
                has_credential: true,
                source,
                is_valid: None,
                can_import_from_env: can_import,
            },
            None => CredentialStatus {
                provider_id: provider_id.to_string(),
                has_credential: false,
                source: CredentialSource::NotSet,
                is_valid: None,
                can_import_from_env: can_import,
            },
        }
    }

    pub fn import_from_env(
        &self,
        provider_id: &str,
        field_id: &str,
        env_var: &str,
        storage: StorageType,
    ) -> Result<(), String> {
        let value =
            std::env::var(env_var).map_err(|_| format!("Environment variable {} not set", env_var))?;

        if value.is_empty() {
            return Err(format!("Environment variable {} is empty", env_var));
        }

        self.store(provider_id, field_id, &value, storage)
    }

    fn load_config(&self) -> Result<CredentialsConfig, String> {
        if !self.config_path.exists() {
            return Ok(CredentialsConfig::default());
        }

        let content = std::fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Cannot read config: {}", e))?;

        toml::from_str(&content).map_err(|e| format!("Cannot parse config: {}", e))
    }

    fn save_config(&self, config: &CredentialsConfig) -> Result<(), String> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create config dir: {}", e))?;
        }

        let content =
            toml::to_string_pretty(config).map_err(|e| format!("Cannot serialize config: {}", e))?;

        std::fs::write(&self.config_path, content)
            .map_err(|e| format!("Cannot write config: {}", e))
    }
}

impl Default for CredentialManager {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn validate_api_key(provider_id: &str, api_key: &str) -> Result<bool, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let result = match provider_id {
        "claude" => {
            client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("content-type", "application/json")
                .body(r#"{"model":"claude-3-haiku-20240307","max_tokens":1,"messages":[{"role":"user","content":"hi"}]}"#)
                .send()
                .await
        }
        "openai" => {
            client
                .get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
        }
        "gemini" => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                api_key
            );
            client.get(&url).send().await
        }
        "mistral" => {
            client
                .get("https://api.mistral.ai/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
        }
        "ollama" => {
            let base_url = if api_key.is_empty() {
                "http://localhost:11434"
            } else {
                api_key
            };
            let url = format!("{}/api/tags", base_url);
            client.get(&url).send().await
        }
        _ => return Err(format!("Unknown provider: {}", provider_id)),
    };

    match result {
        Ok(response) => {
            let status = response.status();
            if status.is_success() || status.as_u16() == 401 {
                Ok(status.is_success())
            } else {
                let error_text = response.text().await.unwrap_or_default();
                if error_text.contains("invalid") || error_text.contains("unauthorized") {
                    Ok(false)
                } else {
                    Ok(status.is_success())
                }
            }
        }
        Err(e) => {
            if e.is_timeout() {
                Err("Request timed out - check your network connection".to_string())
            } else if e.is_connect() {
                Err("Could not connect to API server".to_string())
            } else {
                Err(format!("Validation request failed: {}", e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_manager_creation() {
        let manager = CredentialManager::new();
        let options = manager.storage_options();
        assert!(!options.config_file_path.is_empty());
    }

    #[test]
    fn test_storage_options() {
        let manager = CredentialManager::new();
        let options = manager.storage_options();
        assert!(options.config_file_path.contains("credentials.toml"));
    }
}
