use chrono::{DateTime, Duration, Utc};
use sena_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub provider: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub pkce_required: bool,
}

impl OAuthConfig {
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_string(),
            client_id: String::new(),
            client_secret: None,
            auth_url: String::new(),
            token_url: String::new(),
            redirect_uri: "http://localhost:19960/callback".to_string(),
            scopes: Vec::new(),
            pkce_required: true,
        }
    }

    pub fn with_client_id(mut self, id: &str) -> Self {
        self.client_id = id.to_string();
        self
    }

    pub fn with_client_secret(mut self, secret: &str) -> Self {
        self.client_secret = Some(secret.to_string());
        self
    }

    pub fn with_auth_url(mut self, url: &str) -> Self {
        self.auth_url = url.to_string();
        self
    }

    pub fn with_token_url(mut self, url: &str) -> Self {
        self.token_url = url.to_string();
        self
    }

    pub fn with_scopes(mut self, scopes: Vec<&str>) -> Self {
        self.scopes = scopes.into_iter().map(String::from).collect();
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

impl OAuthToken {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() >= expires_at - Duration::minutes(5)
        } else {
            false
        }
    }

    pub fn from_response(response: TokenResponse) -> Self {
        let expires_at = response.expires_in.map(|secs| {
            Utc::now() + Duration::seconds(secs as i64)
        });

        Self {
            access_token: response.access_token,
            token_type: response.token_type.unwrap_or_else(|| "Bearer".to_string()),
            expires_at,
            refresh_token: response.refresh_token,
            scope: response.scope,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

pub struct OAuthClient {
    http_client: reqwest::Client,
}

impl OAuthClient {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn authenticate(&self, config: &OAuthConfig) -> Result<OAuthToken> {
        let (code_verifier, code_challenge) = if config.pkce_required {
            let verifier = Self::generate_code_verifier();
            let challenge = Self::generate_code_challenge(&verifier);
            (Some(verifier), Some(challenge))
        } else {
            (None, None)
        };

        let state = Self::generate_state();

        let auth_url = self.build_auth_url(config, &state, code_challenge.as_deref())?;

        println!("\n🔐 Opening browser for authentication...");
        println!("If browser doesn't open, visit:\n{}\n", auth_url);

        if let Err(e) = open::that(&auth_url) {
            eprintln!("Failed to open browser: {}", e);
        }

        let code = self.wait_for_callback(&state)?;

        let token = self.exchange_code(config, &code, code_verifier.as_deref()).await?;

        println!("✅ Authentication successful!");

        Ok(token)
    }

    fn build_auth_url(
        &self,
        config: &OAuthConfig,
        state: &str,
        code_challenge: Option<&str>,
    ) -> Result<String> {
        let mut params = vec![
            ("client_id", config.client_id.as_str()),
            ("redirect_uri", config.redirect_uri.as_str()),
            ("response_type", "code"),
            ("state", state),
        ];

        let scopes = config.scopes.join(" ");
        if !scopes.is_empty() {
            params.push(("scope", &scopes));
        }

        if let Some(challenge) = code_challenge {
            params.push(("code_challenge", challenge));
            params.push(("code_challenge_method", "S256"));
        }

        let query = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        Ok(format!("{}?{}", config.auth_url, query))
    }

    fn wait_for_callback(&self, expected_state: &str) -> Result<String> {
        let listener = TcpListener::bind("127.0.0.1:19960")
            .map_err(|e| Error::network(format!("Failed to start callback server: {}", e)))?;

        listener.set_nonblocking(false)
            .map_err(|e| Error::network(format!("Failed to configure listener: {}", e)))?;

        println!("⏳ Waiting for authentication callback...");

        let (mut stream, _) = listener.accept()
            .map_err(|e| Error::network(format!("Failed to accept connection: {}", e)))?;

        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)
            .map_err(|e| Error::network(format!("Failed to read request: {}", e)))?;

        let (code, state) = self.parse_callback(&request_line)?;

        if state != expected_state {
            let response = "HTTP/1.1 400 Bad Request\r\n\r\n<html><body><h1>Authentication Failed</h1><p>Invalid state parameter.</p></body></html>";
            let _ = stream.write_all(response.as_bytes());
            return Err(Error::auth("State mismatch - possible CSRF attack"));
        }

        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>✅ Authentication Successful!</h1><p>You can close this window and return to the terminal.</p><script>window.close();</script></body></html>";
        let _ = stream.write_all(response.as_bytes());

        Ok(code)
    }

    fn parse_callback(&self, request: &str) -> Result<(String, String)> {
        let path = request
            .split_whitespace()
            .nth(1)
            .ok_or_else(|| Error::auth("Invalid callback request"))?;

        let query = path
            .split('?')
            .nth(1)
            .ok_or_else(|| Error::auth("No query parameters in callback"))?;

        let params: HashMap<_, _> = query
            .split('&')
            .filter_map(|p| {
                let mut parts = p.split('=');
                Some((parts.next()?, parts.next()?))
            })
            .collect();

        if let Some(error) = params.get("error") {
            let desc = params.get("error_description").unwrap_or(&"Unknown error");
            return Err(Error::auth(format!("OAuth error: {} - {}", error, desc)));
        }

        let code = params
            .get("code")
            .ok_or_else(|| Error::auth("No authorization code in callback"))?
            .to_string();

        let state = params
            .get("state")
            .ok_or_else(|| Error::auth("No state in callback"))?
            .to_string();

        Ok((code, state))
    }

    async fn exchange_code(
        &self,
        config: &OAuthConfig,
        code: &str,
        code_verifier: Option<&str>,
    ) -> Result<OAuthToken> {
        let mut params = vec![
            ("client_id", config.client_id.as_str()),
            ("code", code),
            ("redirect_uri", config.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ];

        if let Some(secret) = &config.client_secret {
            params.push(("client_secret", secret.as_str()));
        }

        if let Some(verifier) = code_verifier {
            params.push(("code_verifier", verifier));
        }

        let response = self.http_client
            .post(&config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::network(format!("Token exchange failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::auth(format!("Token exchange failed: {}", error_text)));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| Error::auth(format!("Failed to parse token response: {}", e)))?;

        Ok(OAuthToken::from_response(token_response))
    }

    pub async fn refresh_token(&self, config: &OAuthConfig, refresh_token: &str) -> Result<OAuthToken> {
        let mut params = vec![
            ("client_id", config.client_id.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ];

        if let Some(secret) = &config.client_secret {
            params.push(("client_secret", secret.as_str()));
        }

        let response = self.http_client
            .post(&config.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| Error::network(format!("Token refresh failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::auth(format!("Token refresh failed: {}", error_text)));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| Error::auth(format!("Failed to parse token response: {}", e)))?;

        Ok(OAuthToken::from_response(token_response))
    }

    fn generate_code_verifier() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        base64_url_encode(&bytes)
    }

    fn generate_code_challenge(verifier: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        base64_url_encode(&hash)
    }

    fn generate_state() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        base64_url_encode(&bytes)
    }
}

impl Default for OAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

fn base64_url_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(data)
}
