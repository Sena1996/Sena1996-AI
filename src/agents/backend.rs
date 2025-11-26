use super::{DomainAgentType, DomainAnalysis, Finding, Severity};
use once_cell::sync::Lazy;
use regex::Regex;

static ENDPOINT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(get|post|put|patch|delete|router\.(get|post|put|patch|delete))\s*\(\s*['"`]([^'"`]+)['"`]"#)
        .expect("invalid endpoint regex")
});

static SQL_INJECTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(`[^`]*(SELECT|INSERT|UPDATE|DELETE|FROM|WHERE)[^`]*\$\{[^}]+\}[^`]*`)"#)
        .expect("invalid sql injection regex")
});

static HARDCODED_SECRET_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(password|secret|api_key|apikey|token|auth)\s*[=:]\s*['"`][^'"`]{8,}['"`]"#)
        .expect("invalid secret regex")
});

static JWT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(jwt|jsonwebtoken|jose)"#).expect("invalid jwt regex")
});

static DB_CONNECTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(mongodb|postgres|mysql|redis|sqlite)://[^\s'"`]+"#)
        .expect("invalid db connection regex")
});

pub struct BackendAgent {
    #[allow(dead_code)]
    name: String,
}

impl BackendAgent {
    pub fn new() -> Self {
        Self {
            name: "Backend Agent".to_string(),
        }
    }

    pub fn analyze(&self, command: &str, input: &str) -> DomainAnalysis {
        match command {
            "map" | "endpoints" => self.map_endpoints(input),
            "flow" => self.analyze_data_flow(input),
            "auth" => self.audit_auth(input),
            "secrets" => self.scan_secrets(input),
            "security" => self.security_scan(input),
            _ => self.full_analysis(input),
        }
    }

    fn map_endpoints(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        let mut endpoints_found = Vec::new();
        for cap in ENDPOINT_REGEX.captures_iter(&input_lower) {
            if let Some(path) = cap.get(3) {
                endpoints_found.push(path.as_str().to_string());
            }
        }

        if !endpoints_found.is_empty() {
            for endpoint in &endpoints_found {
                let severity = if endpoint.contains("admin") || endpoint.contains("delete") {
                    Severity::Warning
                } else {
                    Severity::Info
                };
                findings.push(Finding {
                    severity,
                    title: format!("Endpoint: {}", endpoint),
                    description: "API endpoint detected".to_string(),
                    location: None,
                    suggestion: Some("Document this endpoint in OpenAPI spec".to_string()),
                });
            }
        }

        if input_lower.contains("/api/") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "REST API structure detected".to_string(),
                description: "Code follows REST API conventions with /api/ prefix".to_string(),
                location: None,
                suggestion: Some("Ensure consistent versioning (e.g., /api/v1/)".to_string()),
            });
        }

        if input_lower.contains("graphql") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "GraphQL API detected".to_string(),
                description: "GraphQL implementation found".to_string(),
                location: None,
                suggestion: Some("Implement query complexity limits and depth limiting".to_string()),
            });
        }

        let score = if endpoints_found.len() > 5 { 85 } else { 70 };

        DomainAnalysis {
            agent: DomainAgentType::Backend,
            category: "API Mapping".to_string(),
            findings,
            recommendations: vec![
                "Generate OpenAPI/Swagger documentation".to_string(),
                "Add request/response validation schemas".to_string(),
                "Implement API versioning strategy".to_string(),
                "Create endpoint dependency graph".to_string(),
            ],
            score,
        }
    }

    fn analyze_data_flow(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if input_lower.contains("req.body") || input_lower.contains("request.body") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "Direct request body access".to_string(),
                description: "Request body is accessed directly without validation".to_string(),
                location: None,
                suggestion: Some("Use validation middleware (Joi, Zod, class-validator)".to_string()),
            });
        }

        if input_lower.contains("req.params") || input_lower.contains("req.query") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "URL parameter access detected".to_string(),
                description: "Query/path parameters should be validated and sanitized".to_string(),
                location: None,
                suggestion: Some("Validate params with schema before use".to_string()),
            });
        }

        if input_lower.contains(".save(") || input_lower.contains(".create(") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Database write operation".to_string(),
                description: "Data is being persisted to database".to_string(),
                location: None,
                suggestion: Some("Ensure data is validated before persistence".to_string()),
            });
        }

        if input_lower.contains(".find(") || input_lower.contains(".query(") || input_lower.contains("select ") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Database read operation".to_string(),
                description: "Data is being queried from database".to_string(),
                location: None,
                suggestion: Some("Consider caching frequent queries".to_string()),
            });
        }

        if input_lower.contains("json") && input_lower.contains("res") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "JSON response format".to_string(),
                description: "API returns JSON responses".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let has_validation = input_lower.contains("validate") || input_lower.contains("schema");
        let score = if has_validation { 80 } else { 60 };

        DomainAnalysis {
            agent: DomainAgentType::Backend,
            category: "Data Flow Analysis".to_string(),
            findings,
            recommendations: vec![
                "Implement input validation at API boundary".to_string(),
                "Add data transformation layer (DTOs)".to_string(),
                "Use repository pattern for data access".to_string(),
                "Implement audit logging for data changes".to_string(),
                "Add request/response interceptors".to_string(),
            ],
            score,
        }
    }

    fn audit_auth(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if JWT_REGEX.is_match(&input_lower) {
            findings.push(Finding {
                severity: Severity::Info,
                title: "JWT authentication detected".to_string(),
                description: "JSON Web Tokens are used for authentication".to_string(),
                location: None,
                suggestion: Some("Ensure tokens have short expiry and use refresh tokens".to_string()),
            });

            if input_lower.contains("expiresIn") || input_lower.contains("exp") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Token expiration configured".to_string(),
                    description: "JWT tokens have expiration time set".to_string(),
                    location: None,
                    suggestion: None,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Critical,
                    title: "No token expiration detected".to_string(),
                    description: "JWT tokens may not have expiration configured".to_string(),
                    location: None,
                    suggestion: Some("Add expiresIn option to jwt.sign()".to_string()),
                });
            }
        }

        if input_lower.contains("bcrypt") || input_lower.contains("argon2") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Secure password hashing".to_string(),
                description: "Passwords are hashed with secure algorithm".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input_lower.contains("password") && (input_lower.contains("md5") || input_lower.contains("sha1")) {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "Weak password hashing".to_string(),
                description: "MD5/SHA1 should not be used for password hashing".to_string(),
                location: None,
                suggestion: Some("Use bcrypt or Argon2 for password hashing".to_string()),
            });
        }

        if input_lower.contains("oauth") || input_lower.contains("passport") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "OAuth/Social authentication".to_string(),
                description: "OAuth or Passport.js integration detected".to_string(),
                location: None,
                suggestion: Some("Validate OAuth state parameter to prevent CSRF".to_string()),
            });
        }

        if input_lower.contains("role") || input_lower.contains("permission") || input_lower.contains("authorize") {
            findings.push(Finding {
                severity: Severity::Info,
                title: "Authorization logic detected".to_string(),
                description: "Role-based or permission-based access control found".to_string(),
                location: None,
                suggestion: Some("Ensure authorization checks on all protected routes".to_string()),
            });
        }

        if input_lower.contains("session") {
            if input_lower.contains("httponly") && input_lower.contains("secure") {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "Secure session cookies".to_string(),
                    description: "Session cookies have httpOnly and secure flags".to_string(),
                    location: None,
                    suggestion: None,
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Warning,
                    title: "Session security check needed".to_string(),
                    description: "Verify session cookies have httpOnly and secure flags".to_string(),
                    location: None,
                    suggestion: Some("Set httpOnly: true, secure: true, sameSite: 'strict'".to_string()),
                });
            }
        }

        let critical_count = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let score = if critical_count > 0 { 40 } else { 75 };

        DomainAnalysis {
            agent: DomainAgentType::Backend,
            category: "Authentication Audit".to_string(),
            findings,
            recommendations: vec![
                "Implement multi-factor authentication (MFA)".to_string(),
                "Use short-lived access tokens with refresh rotation".to_string(),
                "Implement rate limiting on auth endpoints".to_string(),
                "Add brute force protection".to_string(),
                "Log authentication events for auditing".to_string(),
            ],
            score,
        }
    }

    fn scan_secrets(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if HARDCODED_SECRET_REGEX.is_match(input) {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "Hardcoded secret detected".to_string(),
                description: "Credentials or API keys appear to be hardcoded in source".to_string(),
                location: None,
                suggestion: Some("Move secrets to environment variables".to_string()),
            });
        }

        if DB_CONNECTION_REGEX.is_match(input) {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "Database connection string exposed".to_string(),
                description: "Database URL with credentials found in code".to_string(),
                location: None,
                suggestion: Some("Use DATABASE_URL environment variable".to_string()),
            });
        }

        if input_lower.contains("process.env") || input_lower.contains("env::var") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Environment variables used".to_string(),
                description: "Configuration is loaded from environment".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("aws_access_key") || input_lower.contains("aws_secret") {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "AWS credentials detected".to_string(),
                description: "AWS access keys found in code".to_string(),
                location: None,
                suggestion: Some("Use IAM roles or AWS Secrets Manager".to_string()),
            });
        }

        if input_lower.contains(".env") && input_lower.contains("gitignore") {
            findings.push(Finding {
                severity: Severity::Success,
                title: ".env in .gitignore".to_string(),
                description: "Environment file is excluded from version control".to_string(),
                location: None,
                suggestion: None,
            });
        }

        let critical_count = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let score = if critical_count > 0 { 30 } else { 90 };

        DomainAnalysis {
            agent: DomainAgentType::Backend,
            category: "Secrets Scan".to_string(),
            findings,
            recommendations: vec![
                "Use secrets manager (AWS Secrets Manager, HashiCorp Vault)".to_string(),
                "Implement secret rotation policy".to_string(),
                "Add pre-commit hooks to detect secrets".to_string(),
                "Use .env.example for documentation (no real values)".to_string(),
                "Audit access to production secrets".to_string(),
            ],
            score,
        }
    }

    fn security_scan(&self, input: &str) -> DomainAnalysis {
        let mut findings = Vec::new();
        let input_lower = input.to_lowercase();

        if SQL_INJECTION_REGEX.is_match(input) {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "Potential SQL injection".to_string(),
                description: "String interpolation in SQL query detected".to_string(),
                location: None,
                suggestion: Some("Use parameterized queries or ORM".to_string()),
            });
        }

        if input_lower.contains("innerhtml") || input_lower.contains("dangerouslysetinnerhtml") {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "XSS vulnerability risk".to_string(),
                description: "Direct HTML injection detected".to_string(),
                location: None,
                suggestion: Some("Use DOMPurify or escape HTML content".to_string()),
            });
        }

        if input_lower.contains("eval(") || input_lower.contains("function(") && input_lower.contains("return") {
            findings.push(Finding {
                severity: Severity::Critical,
                title: "Code injection risk".to_string(),
                description: "Dynamic code execution detected".to_string(),
                location: None,
                suggestion: Some("Avoid eval() and dynamic code execution".to_string()),
            });
        }

        if input_lower.contains("cors") {
            if input_lower.contains("origin: '*'") || input_lower.contains("origin: true") {
                findings.push(Finding {
                    severity: Severity::Warning,
                    title: "Permissive CORS policy".to_string(),
                    description: "CORS allows all origins".to_string(),
                    location: None,
                    suggestion: Some("Restrict CORS to specific trusted origins".to_string()),
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Success,
                    title: "CORS configured".to_string(),
                    description: "CORS policy is implemented".to_string(),
                    location: None,
                    suggestion: None,
                });
            }
        }

        if input_lower.contains("helmet") || input_lower.contains("content-security-policy") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Security headers configured".to_string(),
                description: "Helmet or CSP headers detected".to_string(),
                location: None,
                suggestion: None,
            });
        }

        if input_lower.contains("ratelimit") || input_lower.contains("rate_limit") {
            findings.push(Finding {
                severity: Severity::Success,
                title: "Rate limiting implemented".to_string(),
                description: "API has rate limiting protection".to_string(),
                location: None,
                suggestion: None,
            });
        } else if input_lower.contains("api") || input_lower.contains("endpoint") {
            findings.push(Finding {
                severity: Severity::Warning,
                title: "No rate limiting detected".to_string(),
                description: "API endpoints may be vulnerable to abuse".to_string(),
                location: None,
                suggestion: Some("Implement rate limiting middleware".to_string()),
            });
        }

        let critical_count = findings.iter().filter(|f| f.severity == Severity::Critical).count();
        let score = if critical_count > 0 { 35 } else if critical_count == 0 { 85 } else { 60 };

        DomainAnalysis {
            agent: DomainAgentType::Backend,
            category: "Security Scan".to_string(),
            findings,
            recommendations: vec![
                "Run OWASP dependency check".to_string(),
                "Implement input validation on all endpoints".to_string(),
                "Add security headers (Helmet.js)".to_string(),
                "Enable HTTPS everywhere".to_string(),
                "Implement proper error handling (no stack traces)".to_string(),
                "Add request logging and monitoring".to_string(),
            ],
            score,
        }
    }

    fn full_analysis(&self, input: &str) -> DomainAnalysis {
        let mut all_findings = Vec::new();

        let endpoint_analysis = self.map_endpoints(input);
        let flow_analysis = self.analyze_data_flow(input);
        let auth_analysis = self.audit_auth(input);
        let secrets_analysis = self.scan_secrets(input);
        let security_analysis = self.security_scan(input);

        all_findings.extend(endpoint_analysis.findings);
        all_findings.extend(flow_analysis.findings);
        all_findings.extend(auth_analysis.findings);
        all_findings.extend(secrets_analysis.findings);
        all_findings.extend(security_analysis.findings);

        let scores = [
            endpoint_analysis.score,
            flow_analysis.score,
            auth_analysis.score,
            secrets_analysis.score,
            security_analysis.score,
        ];
        let avg_score = scores.iter().sum::<u8>() / scores.len() as u8;

        DomainAnalysis {
            agent: DomainAgentType::Backend,
            category: "Full Backend Analysis".to_string(),
            findings: all_findings,
            recommendations: vec![
                "Address all critical security issues first".to_string(),
                "Implement comprehensive input validation".to_string(),
                "Set up CI/CD security scanning".to_string(),
                "Create API documentation".to_string(),
                "Implement proper logging and monitoring".to_string(),
                "Review authentication and authorization flows".to_string(),
            ],
            score: avg_score,
        }
    }
}

impl Default for BackendAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_agent_creation() {
        let agent = BackendAgent::new();
        assert_eq!(agent.name, "Backend Agent");
    }

    #[test]
    fn test_endpoint_mapping() {
        let agent = BackendAgent::new();
        let code = r#"
            app.get('/api/users', getUsers);
            app.post('/api/users', createUser);
            router.delete('/api/users/:id', deleteUser);
        "#;
        let result = agent.analyze("map", code);
        assert_eq!(result.category, "API Mapping");
    }

    #[test]
    fn test_security_scan_sql_injection() {
        let agent = BackendAgent::new();
        let code = r#"
            const query = `SELECT * FROM users WHERE id = ${userId}`;
            db.query(query);
        "#;
        let result = agent.analyze("security", code);
        assert!(result.findings.iter().any(|f| f.severity == Severity::Critical));
    }

    #[test]
    fn test_auth_audit_jwt() {
        let agent = BackendAgent::new();
        let code = r#"
            const jwt = require('jsonwebtoken');
            const token = jwt.sign({ userId }, secret, { expiresIn: '15m' });
        "#;
        let result = agent.analyze("auth", code);
        assert!(result.findings.iter().any(|f| f.title.contains("JWT")));
    }

    #[test]
    fn test_secrets_scan() {
        let agent = BackendAgent::new();
        let code = r#"
            const password = "supersecret123";
            const apiKey = "sk-1234567890abcdef";
        "#;
        let result = agent.analyze("secrets", code);
        assert!(result.findings.iter().any(|f| f.severity == Severity::Critical));
    }
}
