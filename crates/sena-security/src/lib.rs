pub mod auth;
mod encryption;
mod keyring;
mod ratelimit;
mod sanitizer;

use base64::Engine;

pub use auth::{
    AuthCredentials, AuthManager, AuthMethod, CredentialStatus, OAuthClient, OAuthConfig,
    OAuthToken, ProviderAuth, TokenStore,
};
pub use encryption::{decrypt_string, encrypt_string, hash_string, Encryptor};
pub use keyring::KeyManager;
pub use ratelimit::RateLimiter;
pub use sanitizer::Sanitizer;

pub struct SecurityContext {
    pub key_manager: KeyManager,
    pub encryptor: Option<Encryptor>,
    pub sanitizer: Sanitizer,
    pub rate_limiter: RateLimiter,
}

impl SecurityContext {
    pub fn new(config: &sena_core::config::SecurityConfig) -> Self {
        let key_manager = KeyManager::new(&config.keyring_service);

        let encryptor = if config.encryption_enabled {
            let master_key = key_manager
                .get("master_key")
                .ok()
                .flatten()
                .unwrap_or_else(|| {
                    let key = Encryptor::generate_key();
                    let encoded = base64::engine::general_purpose::STANDARD.encode(key);
                    let _ = key_manager.set("master_key", &encoded);
                    encoded
                });

            Some(Encryptor::from_password(&master_key))
        } else {
            None
        };

        let rate_limiter = RateLimiter::from_config(&config.rate_limit);

        Self {
            key_manager,
            encryptor,
            sanitizer: Sanitizer::new(),
            rate_limiter,
        }
    }

    pub fn validate_and_sanitize(&self, input: &str) -> sena_core::Result<String> {
        self.sanitizer.sanitize_input(input)
    }

    pub fn check_rate_limit(&self, key: &str) -> sena_core::Result<()> {
        self.rate_limiter.try_acquire(key)
    }

    pub fn get_api_key(&self, provider: &str) -> sena_core::Result<Option<String>> {
        self.key_manager.get(provider)
    }

    pub fn set_api_key(&self, provider: &str, key: &str) -> sena_core::Result<()> {
        let sanitized = self.sanitizer.sanitize_api_key(key)?;
        self.key_manager.set(provider, &sanitized)
    }

    pub fn encrypt(&self, data: &str) -> sena_core::Result<String> {
        match &self.encryptor {
            Some(e) => e.encrypt_string(data),
            None => Err(sena_core::Error::security("encryption not enabled")),
        }
    }

    pub fn decrypt(&self, data: &str) -> sena_core::Result<String> {
        match &self.encryptor {
            Some(e) => e.decrypt_string(data),
            None => Err(sena_core::Error::security("encryption not enabled")),
        }
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new(&sena_core::config::SecurityConfig::default())
    }
}
