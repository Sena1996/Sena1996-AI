use serde::{Deserialize, Serialize};

use super::OAuthToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    OAuth,
    ApiKey,
    None,
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OAuth => write!(f, "OAuth"),
            Self::ApiKey => write!(f, "API Key"),
            Self::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialStatus {
    Valid,
    Expired,
    Invalid,
    NotFound,
}

impl std::fmt::Display for CredentialStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valid => write!(f, "Valid"),
            Self::Expired => write!(f, "Expired"),
            Self::Invalid => write!(f, "Invalid"),
            Self::NotFound => write!(f, "Not Found"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub provider: String,
    pub method: AuthMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_token: Option<OAuthToken>,
    pub status: CredentialStatus,
}

impl AuthCredentials {
    pub fn get_token(&self) -> Option<&str> {
        if let Some(ref key) = self.api_key {
            return Some(key);
        }
        if let Some(ref token) = self.oauth_token {
            return Some(&token.access_token);
        }
        None
    }

    pub fn is_valid(&self) -> bool {
        self.status == CredentialStatus::Valid
    }

    pub fn needs_refresh(&self) -> bool {
        self.status == CredentialStatus::Expired
    }

    pub fn masked_token(&self) -> String {
        if let Some(token) = self.get_token() {
            let len = token.len();
            if len <= 8 {
                return "*".repeat(len);
            }
            format!("{}...{}", &token[..4], &token[len-4..])
        } else {
            "None".to_string()
        }
    }
}
