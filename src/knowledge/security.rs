use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VulnerabilityType {
    SqlInjection,
    Xss,
    CommandInjection,
    PathTraversal,
    BrokenAuth,
    DataExposure,
    Misconfiguration,
    InsecureDeserialization,
    Csrf,
    Other,
}

impl std::fmt::Display for VulnerabilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VulnerabilityType::SqlInjection => write!(f, "SQL Injection"),
            VulnerabilityType::Xss => write!(f, "Cross-Site Scripting (XSS)"),
            VulnerabilityType::CommandInjection => write!(f, "Command Injection"),
            VulnerabilityType::PathTraversal => write!(f, "Path Traversal"),
            VulnerabilityType::BrokenAuth => write!(f, "Broken Authentication"),
            VulnerabilityType::DataExposure => write!(f, "Sensitive Data Exposure"),
            VulnerabilityType::Misconfiguration => write!(f, "Security Misconfiguration"),
            VulnerabilityType::InsecureDeserialization => write!(f, "Insecure Deserialization"),
            VulnerabilityType::Csrf => write!(f, "CSRF"),
            VulnerabilityType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPattern {
    pub name: String,
    pub description: String,
    pub vulnerability_type: VulnerabilityType,
    pub severity: u8,
    pub secure_example: String,
    pub insecure_example: String,
    pub prevention: Vec<String>,
}

impl SecurityPattern {
    pub fn new(name: &str, description: &str, vuln_type: VulnerabilityType) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            vulnerability_type: vuln_type,
            severity: 5,
            secure_example: String::new(),
            insecure_example: String::new(),
            prevention: Vec::new(),
        }
    }

    pub fn with_severity(mut self, severity: u8) -> Self {
        self.severity = severity.min(10);
        self
    }

    pub fn with_secure_example(mut self, example: &str) -> Self {
        self.secure_example = example.to_string();
        self
    }

    pub fn with_insecure_example(mut self, example: &str) -> Self {
        self.insecure_example = example.to_string();
        self
    }

    pub fn with_prevention(mut self, tip: &str) -> Self {
        self.prevention.push(tip.to_string());
        self
    }

    pub fn with_preventions(mut self, tips: &[&str]) -> Self {
        for tip in tips {
            self.prevention.push(tip.to_string());
        }
        self
    }
}

impl std::fmt::Display for SecurityPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════")?;
        writeln!(f, "  🔒 {}", self.name)?;
        writeln!(f, "═══════════════════════════════════════")?;
        writeln!(f)?;
        writeln!(f, "{}", self.description)?;
        writeln!(f)?;
        writeln!(f, "Vulnerability: {}", self.vulnerability_type)?;
        writeln!(f, "Severity: {}/10", self.severity)?;
        writeln!(f)?;

        if !self.secure_example.is_empty() {
            writeln!(f, "✅ SECURE:")?;
            writeln!(f, "{}", self.secure_example)?;
            writeln!(f)?;
        }

        if !self.insecure_example.is_empty() {
            writeln!(f, "❌ INSECURE:")?;
            writeln!(f, "{}", self.insecure_example)?;
            writeln!(f)?;
        }

        if !self.prevention.is_empty() {
            writeln!(f, "Prevention:")?;
            for tip in &self.prevention {
                writeln!(f, "  • {}", tip)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAudit {
    pub name: String,
    pub files_checked: Vec<String>,
    pub vulnerabilities: Vec<AuditFinding>,
    pub score: u8,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub vulnerability_type: VulnerabilityType,
    pub file: String,
    pub line: Option<usize>,
    pub description: String,
    pub severity: u8,
    pub code_snippet: Option<String>,
}

pub fn default_patterns() -> Vec<SecurityPattern> {
    vec![
        SecurityPattern::new(
            "SQL Injection Prevention",
            "Prevent SQL injection by using parameterized queries instead of string concatenation.",
            VulnerabilityType::SqlInjection,
        )
        .with_severity(9)
        .with_secure_example(
            "// Parameterized query\n\
            const user = await db.query(\n\
                'SELECT * FROM users WHERE email = $1',\n\
                [email]\n\
            );\n\n\
            // ORM with parameter binding\n\
            const user = await User.findOne({ where: { email } });",
        )
        .with_insecure_example(
            "// String concatenation - VULNERABLE!\n\
            const user = await db.query(\n\
                `SELECT * FROM users WHERE email = '${email}'`\n\
            );",
        )
        .with_preventions(&[
            "Always use parameterized queries",
            "Use ORM with proper parameter binding",
            "Validate and sanitize all user input",
            "Apply principle of least privilege to database accounts",
        ]),
        SecurityPattern::new(
            "XSS Prevention",
            "Prevent Cross-Site Scripting by encoding output and sanitizing HTML.",
            VulnerabilityType::Xss,
        )
        .with_severity(8)
        .with_secure_example(
            "// DOMPurify for HTML\n\
            import DOMPurify from 'dompurify';\n\
            const safeHTML = DOMPurify.sanitize(userInput);\n\n\
            // React auto-escapes\n\
            <div>{userInput}</div>  // Safe!\n\n\
            // Attribute encoding\n\
            import { escape } from 'html-escaper';\n\
            const safeAttr = escape(userInput);",
        )
        .with_insecure_example(
            "// Raw HTML injection - VULNERABLE!\n\
            element.innerHTML = userInput;\n\n\
            // dangerouslySetInnerHTML without sanitization\n\
            <div dangerouslySetInnerHTML={{ __html: userInput }} />",
        )
        .with_preventions(&[
            "Use DOMPurify or similar for HTML sanitization",
            "Let React/Vue handle escaping automatically",
            "Set Content-Security-Policy headers",
            "Use httpOnly cookies to protect tokens",
        ]),
        SecurityPattern::new(
            "Command Injection Prevention",
            "Prevent command injection by avoiding shell execution with user input.",
            VulnerabilityType::CommandInjection,
        )
        .with_severity(10)
        .with_secure_example(
            "// Use execFile instead of exec\n\
            import { execFile } from 'child_process';\n\n\
            execFile('convert', [inputFile, outputFile], (error, stdout) => {\n\
                // Safe: No shell interpretation\n\
            });",
        )
        .with_insecure_example(
            "// Shell execution - VULNERABLE!\n\
            import { exec } from 'child_process';\n\n\
            exec(`convert ${userFile} output.png`, (error, stdout) => {\n\
                // Vulnerable if userFile = 'input.png; rm -rf /'\n\
            });",
        )
        .with_preventions(&[
            "Use execFile instead of exec",
            "Avoid passing user input to shell commands",
            "Whitelist allowed commands and arguments",
            "Use libraries instead of shell commands when possible",
        ]),
        SecurityPattern::new(
            "Path Traversal Prevention",
            "Prevent path traversal by validating and normalizing file paths.",
            VulnerabilityType::PathTraversal,
        )
        .with_severity(8)
        .with_secure_example(
            "import path from 'path';\n\n\
            function serveFile(filename: string) {\n\
                const baseDir = '/var/www/uploads';\n\
                const fullPath = path.normalize(path.join(baseDir, filename));\n\n\
                // Ensure path is within baseDir\n\
                if (!fullPath.startsWith(baseDir)) {\n\
                    throw new Error('Invalid file path');\n\
                }\n\n\
                return fs.readFile(fullPath);\n\
            }",
        )
        .with_insecure_example(
            "function serveFile(filename: string) {\n\
                // VULNERABLE: filename = '../../../etc/passwd'\n\
                return fs.readFile(`/var/www/uploads/${filename}`);\n\
            }",
        )
        .with_preventions(&[
            "Normalize paths with path.normalize()",
            "Validate that final path is within allowed directory",
            "Use a whitelist of allowed file names when possible",
            "Avoid including user input in file paths",
        ]),
        SecurityPattern::new(
            "JWT Security",
            "Secure JWT implementation with short-lived tokens and proper validation.",
            VulnerabilityType::BrokenAuth,
        )
        .with_severity(8)
        .with_secure_example(
            "// Short-lived access token + refresh token\n\
            const accessToken = jwt.sign(\n\
                { userId: user.id, role: user.role },\n\
                process.env.JWT_SECRET,\n\
                { expiresIn: '15m', algorithm: 'HS256' }\n\
            );\n\n\
            const refreshToken = jwt.sign(\n\
                { userId: user.id, tokenFamily: uuidv4() },\n\
                process.env.REFRESH_SECRET,\n\
                { expiresIn: '7d', algorithm: 'HS256' }\n\
            );",
        )
        .with_insecure_example(
            "// VULNERABLE!\n\
            const token = jwt.sign(\n\
                { userId: user.id },\n\
                'hardcoded-secret',  // Hardcoded secret!\n\
                { expiresIn: '30d' } // Too long!\n\
            );",
        )
        .with_preventions(&[
            "Use short-lived access tokens (15 minutes)",
            "Store secrets in environment variables",
            "Always specify algorithm explicitly",
            "Implement refresh token rotation",
        ]),
        SecurityPattern::new(
            "Password Hashing",
            "Use modern password hashing algorithms like bcrypt or Argon2.",
            VulnerabilityType::DataExposure,
        )
        .with_severity(9)
        .with_secure_example(
            "// bcrypt\n\
            import bcrypt from 'bcrypt';\n\
            const hash = await bcrypt.hash(password, 12);\n\
            const valid = await bcrypt.compare(password, hash);\n\n\
            // Argon2 (even better)\n\
            import argon2 from 'argon2';\n\
            const hash = await argon2.hash(password);\n\
            const valid = await argon2.verify(hash, password);",
        )
        .with_insecure_example(
            "// VULNERABLE!\n\
            const hash = crypto.createHash('md5').update(password).digest('hex');\n\
            // Problems:\n\
            // - MD5 is broken\n\
            // - No salt\n\
            // - Too fast (enables brute force)",
        )
        .with_preventions(&[
            "Use bcrypt with at least 12 rounds",
            "Or use Argon2 (winner of Password Hashing Competition)",
            "Never use MD5, SHA1, or SHA256 for passwords",
            "Always use a salt (bcrypt/Argon2 handle this automatically)",
        ]),
        SecurityPattern::new(
            "Session Security",
            "Configure sessions with secure cookie settings.",
            VulnerabilityType::BrokenAuth,
        )
        .with_severity(7)
        .with_secure_example(
            "app.use(session({\n\
                secret: process.env.SESSION_SECRET,\n\
                resave: false,\n\
                saveUninitialized: false,\n\
                cookie: {\n\
                    secure: true,        // HTTPS only\n\
                    httpOnly: true,      // No JavaScript access\n\
                    sameSite: 'strict',  // CSRF protection\n\
                    maxAge: 3600000      // 1 hour\n\
                }\n\
            }));",
        )
        .with_insecure_example(
            "// VULNERABLE!\n\
            app.use(session({\n\
                secret: 'my-secret',  // Hardcoded!\n\
                cookie: {\n\
                    secure: false,    // Works on HTTP!\n\
                    httpOnly: false   // XSS vulnerable!\n\
                }\n\
            }));",
        )
        .with_preventions(&[
            "Set secure: true to require HTTPS",
            "Set httpOnly: true to prevent XSS token theft",
            "Set sameSite: 'strict' for CSRF protection",
            "Use short session lifetimes",
        ]),
        SecurityPattern::new(
            "CORS Configuration",
            "Configure CORS restrictively to prevent unauthorized cross-origin requests.",
            VulnerabilityType::Misconfiguration,
        )
        .with_severity(6)
        .with_secure_example(
            "app.use(cors({\n\
                origin: ['https://example.com', 'https://app.example.com'],\n\
                credentials: true,\n\
                methods: ['GET', 'POST', 'PUT', 'DELETE'],\n\
                allowedHeaders: ['Content-Type', 'Authorization'],\n\
                maxAge: 600\n\
            }));",
        )
        .with_insecure_example(
            "// VULNERABLE: Allows all origins!\n\
            app.use(cors());\n\n\
            // Also vulnerable:\n\
            app.use(cors({ origin: '*' }));",
        )
        .with_preventions(&[
            "Explicitly list allowed origins",
            "Don't use wildcard (*) with credentials",
            "Limit allowed methods and headers",
            "Set appropriate maxAge for preflight caching",
        ]),
        SecurityPattern::new(
            "Rate Limiting",
            "Implement rate limiting to prevent brute force and DoS attacks.",
            VulnerabilityType::Other,
        )
        .with_severity(7)
        .with_secure_example(
            "import rateLimit from 'express-rate-limit';\n\n\
            // General API limiter\n\
            const limiter = rateLimit({\n\
                windowMs: 15 * 60 * 1000,  // 15 minutes\n\
                max: 100,                   // 100 requests per window\n\
                standardHeaders: true,\n\
            });\n\n\
            // Stricter for auth endpoints\n\
            const authLimiter = rateLimit({\n\
                windowMs: 15 * 60 * 1000,\n\
                max: 5,  // Only 5 login attempts\n\
                skipSuccessfulRequests: true\n\
            });\n\n\
            app.use('/api/', limiter);\n\
            app.post('/api/login', authLimiter, login);",
        )
        .with_insecure_example(
            "// No rate limiting - VULNERABLE to:\n\
            // - Brute force attacks\n\
            // - DoS attacks\n\
            // - Credential stuffing\n\
            app.post('/api/login', login);",
        )
        .with_preventions(&[
            "Apply rate limiting to all API endpoints",
            "Use stricter limits for authentication endpoints",
            "Consider using a distributed rate limiter for clustered apps",
            "Implement exponential backoff for repeated failures",
        ]),
        SecurityPattern::new(
            "API Key Security",
            "Properly validate and manage API keys with constant-time comparison.",
            VulnerabilityType::BrokenAuth,
        )
        .with_severity(7)
        .with_secure_example(
            "function validateApiKey(req, res, next) {\n\
                const apiKey = req.headers['x-api-key'];\n\n\
                if (!apiKey) {\n\
                    return res.status(401).json({ error: 'API key required' });\n\
                }\n\n\
                // Hash before comparison (constant-time)\n\
                const hashedKey = crypto.createHash('sha256').update(apiKey).digest('hex');\n\n\
                if (!crypto.timingSafeEqual(\n\
                    Buffer.from(hashedKey),\n\
                    Buffer.from(process.env.API_KEY_HASH)\n\
                )) {\n\
                    return res.status(401).json({ error: 'Invalid API key' });\n\
                }\n\n\
                next();\n\
            }",
        )
        .with_insecure_example(
            "// VULNERABLE!\n\
            function validateApiKey(req, res, next) {\n\
                // Hardcoded key + timing attack vulnerable\n\
                if (req.headers['x-api-key'] !== 'hardcoded-key') {\n\
                    return res.status(401).send('Invalid key');\n\
                }\n\
                next();\n\
            }",
        )
        .with_preventions(&[
            "Store API key hashes, not plaintext keys",
            "Use crypto.timingSafeEqual for comparison",
            "Rotate API keys regularly",
            "Never commit API keys to version control",
        ]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_patterns() {
        let patterns = default_patterns();
        assert!(patterns.len() >= 5);
    }

    #[test]
    fn test_pattern_display() {
        let pattern = &default_patterns()[0];
        let display = format!("{}", pattern);
        assert!(display.contains("SQL Injection"));
    }

    #[test]
    fn test_vulnerability_type_display() {
        assert_eq!(
            format!("{}", VulnerabilityType::SqlInjection),
            "SQL Injection"
        );
    }
}
