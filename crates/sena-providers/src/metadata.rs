use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub icon: Option<String>,
    pub website: String,
    pub documentation_url: Option<String>,
    pub auth_schema: AuthSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSchema {
    pub auth_type: AuthType,
    pub fields: Vec<AuthField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    ApiKey,
    OAuth2,
    BasicAuth,
    Local,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthField {
    pub id: String,
    pub display_name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub sensitive: bool,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub default_value: Option<String>,
    pub env_var_name: Option<String>,
    pub validation_pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Text,
    Password,
    Url,
    Number,
    Toggle,
}

impl ProviderMetadata {
    pub fn new(id: &str, display_name: &str) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            description: String::new(),
            icon: None,
            website: String::new(),
            documentation_url: None,
            auth_schema: AuthSchema {
                auth_type: AuthType::None,
                fields: Vec::new(),
            },
        }
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    pub fn with_website(mut self, website: &str) -> Self {
        self.website = website.to_string();
        self
    }

    pub fn with_docs_url(mut self, url: &str) -> Self {
        self.documentation_url = Some(url.to_string());
        self
    }

    pub fn with_icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn with_auth_schema(mut self, auth_schema: AuthSchema) -> Self {
        self.auth_schema = auth_schema;
        self
    }
}

impl AuthSchema {
    pub fn api_key(fields: Vec<AuthField>) -> Self {
        Self {
            auth_type: AuthType::ApiKey,
            fields,
        }
    }

    pub fn local(fields: Vec<AuthField>) -> Self {
        Self {
            auth_type: AuthType::Local,
            fields,
        }
    }

    pub fn none() -> Self {
        Self {
            auth_type: AuthType::None,
            fields: Vec::new(),
        }
    }
}

impl AuthField {
    pub fn api_key(env_var: &str) -> Self {
        Self {
            id: "api_key".to_string(),
            display_name: "API Key".to_string(),
            field_type: FieldType::Password,
            required: true,
            sensitive: true,
            placeholder: None,
            help_text: None,
            default_value: None,
            env_var_name: Some(env_var.to_string()),
            validation_pattern: None,
        }
    }

    pub fn url(id: &str, display_name: &str, default: Option<&str>) -> Self {
        Self {
            id: id.to_string(),
            display_name: display_name.to_string(),
            field_type: FieldType::Url,
            required: true,
            sensitive: false,
            placeholder: None,
            help_text: None,
            default_value: default.map(|s| s.to_string()),
            env_var_name: None,
            validation_pattern: Some(r"^https?://".to_string()),
        }
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    pub fn with_help_text(mut self, help_text: &str) -> Self {
        self.help_text = Some(help_text.to_string());
        self
    }

    pub fn with_validation(mut self, pattern: &str) -> Self {
        self.validation_pattern = Some(pattern.to_string());
        self
    }
}

pub fn claude_metadata() -> ProviderMetadata {
    ProviderMetadata::new("claude", "Claude (Orchestrator)")
        .with_description(
            "Use Claude Desktop/Code as orchestrator via SENA MCP - no API key needed",
        )
        .with_website("https://claude.ai")
        .with_docs_url("https://docs.anthropic.com")
        .with_icon("anthropic")
        .with_auth_schema(AuthSchema::none())
}

pub fn openai_metadata() -> ProviderMetadata {
    ProviderMetadata::new("openai", "OpenAI")
        .with_description("GPT models including GPT-4.1 and o4-mini reasoning model")
        .with_website("https://openai.com")
        .with_docs_url("https://platform.openai.com/docs")
        .with_icon("openai")
        .with_auth_schema(AuthSchema::api_key(vec![AuthField::api_key(
            "OPENAI_API_KEY",
        )
        .with_placeholder("sk-...")
        .with_help_text("Get your API key from platform.openai.com/api-keys")
        .with_validation(r"^sk-")]))
}

pub fn gemini_metadata() -> ProviderMetadata {
    ProviderMetadata::new("gemini", "Google Gemini")
        .with_description("Google's multimodal AI with 1M+ token context window")
        .with_website("https://ai.google.dev")
        .with_docs_url("https://ai.google.dev/docs")
        .with_icon("google")
        .with_auth_schema(AuthSchema::api_key(vec![AuthField::api_key(
            "GOOGLE_API_KEY",
        )
        .with_placeholder("AI...")
        .with_help_text("Get your API key from aistudio.google.com/apikey")
        .with_validation(r"^AI")]))
}

pub fn ollama_metadata() -> ProviderMetadata {
    ProviderMetadata::new("ollama", "Ollama (Local)")
        .with_description("Run open-source models locally - no API key required")
        .with_website("https://ollama.com")
        .with_docs_url("https://github.com/ollama/ollama")
        .with_icon("ollama")
        .with_auth_schema(AuthSchema::local(vec![AuthField::url(
            "base_url",
            "Server URL",
            Some("http://localhost:11434"),
        )
        .with_placeholder("http://localhost:11434")
        .with_help_text("URL where Ollama server is running")]))
}

pub fn mistral_metadata() -> ProviderMetadata {
    ProviderMetadata::new("mistral", "Mistral AI")
        .with_description("European AI lab with efficient multilingual models")
        .with_website("https://mistral.ai")
        .with_docs_url("https://docs.mistral.ai")
        .with_icon("mistral")
        .with_auth_schema(AuthSchema::api_key(vec![AuthField::api_key(
            "MISTRAL_API_KEY",
        )
        .with_placeholder("...")
        .with_help_text("Get your API key from console.mistral.ai")]))
}

pub fn get_all_provider_metadata() -> Vec<ProviderMetadata> {
    vec![
        claude_metadata(),
        openai_metadata(),
        gemini_metadata(),
        ollama_metadata(),
        mistral_metadata(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_metadata() {
        let meta = claude_metadata();
        assert_eq!(meta.id, "claude");
        assert_eq!(meta.auth_schema.auth_type, AuthType::ApiKey);
        assert_eq!(meta.auth_schema.fields.len(), 1);
        assert_eq!(
            meta.auth_schema.fields[0].env_var_name,
            Some("ANTHROPIC_API_KEY".to_string())
        );
    }

    #[test]
    fn test_ollama_metadata() {
        let meta = ollama_metadata();
        assert_eq!(meta.id, "ollama");
        assert_eq!(meta.auth_schema.auth_type, AuthType::Local);
        assert_eq!(meta.auth_schema.fields[0].field_type, FieldType::Url);
    }

    #[test]
    fn test_all_providers() {
        let all = get_all_provider_metadata();
        assert_eq!(all.len(), 5);
    }
}
