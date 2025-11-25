# SENA Security Patterns & Best Practices

This document contains security knowledge, patterns, and best practices for persistent application across all SENA sessions.

---

## 1. AUTHENTICATION PATTERNS

### Multi-Factor Authentication (MFA)
```typescript
// SECURE: MFA with TOTP
async function authenticateUser(username: string, password: string, totpCode: string) {
    // Step 1: Verify password
    const user = await verifyPassword(username, password);
    if (!user) return null;

    // Step 2: Verify TOTP
    const totpValid = verifyTOTP(user.totpSecret, totpCode);
    if (!totpValid) return null;

    // Step 3: Generate session
    return createSession(user);
}

// INSECURE: Password only
async function authenticateUser(username: string, password: string) {
    return await verifyPassword(username, password); // ❌ No second factor
}
```

### JWT Best Practices
```typescript
// SECURE: Short-lived access token + refresh token
const accessToken = jwt.sign(
    { userId: user.id, role: user.role },
    process.env.JWT_SECRET,
    { expiresIn: '15m', algorithm: 'HS256' } // Short expiry
);

const refreshToken = jwt.sign(
    { userId: user.id, tokenFamily: uuidv4() },
    process.env.REFRESH_SECRET,
    { expiresIn: '7d', algorithm: 'HS256' }
);

// INSECURE: Long-lived token, no refresh
const token = jwt.sign(
    { userId: user.id },
    'hardcoded-secret', // ❌ Hardcoded secret
    { expiresIn: '30d' } // ❌ Too long, ❌ No algorithm specified
);
```

### Session Management
```typescript
// SECURE: Secure session configuration
app.use(session({
    secret: process.env.SESSION_SECRET,
    resave: false,
    saveUninitialized: false,
    cookie: {
        secure: true,        // HTTPS only
        httpOnly: true,      // No JavaScript access
        sameSite: 'strict',  // CSRF protection
        maxAge: 3600000      // 1 hour
    }
}));

// INSECURE:
app.use(session({
    secret: 'my-secret',     // ❌ Hardcoded
    cookie: {
        secure: false,       // ❌ Works on HTTP
        httpOnly: false      // ❌ XSS vulnerable
    }
}));
```

---

## 2. AUTHORIZATION PATTERNS

### Role-Based Access Control (RBAC)
```typescript
// SECURE: Role-based authorization middleware
function authorize(...roles: string[]) {
    return (req, res, next) => {
        if (!req.user) {
            return res.status(401).json({ error: 'Not authenticated' });
        }

        if (!roles.includes(req.user.role)) {
            return res.status(403).json({ error: 'Forbidden' });
        }

        next();
    };
}

// Usage
app.delete('/api/users/:id', authorize('admin'), deleteUser);
app.get('/api/users', authorize('admin', 'manager'), listUsers);

// INSECURE: Client-controlled authorization
app.delete('/api/users/:id', (req, res) => {
    if (req.body.isAdmin) { // ❌ Client can set this!
        deleteUser(req.params.id);
    }
});
```

### Attribute-Based Access Control (ABAC)
```typescript
// SECURE: Fine-grained access control
function canEditDocument(user: User, document: Document): boolean {
    // Owner can edit
    if (document.ownerId === user.id) return true;

    // Admin can edit
    if (user.role === 'admin') return true;

    // Collaborators with edit permission can edit
    if (document.collaborators.some(c =>
        c.userId === user.id && c.permission === 'edit'
    )) return true;

    return false;
}
```

---

## 3. INPUT VALIDATION & SANITIZATION

### SQL Injection Prevention
```typescript
// SECURE: Parameterized queries
const user = await db.query(
    'SELECT * FROM users WHERE email = $1',
    [email] // Parameterized
);

// SECURE: ORM with parameter binding
const user = await User.findOne({ where: { email } });

// INSECURE: String concatenation
const user = await db.query(
    `SELECT * FROM users WHERE email = '${email}'` // ❌ SQL injection!
);
```

### XSS Prevention
```typescript
// SECURE: Output encoding
import DOMPurify from 'dompurify';
import { escape } from 'html-escaper';

// For HTML context
const safeHTML = DOMPurify.sanitize(userInput);

// For attributes
const safeAttr = escape(userInput);

// React automatically escapes
<div>{userInput}</div> // ✅ Safe by default

// INSECURE: Raw HTML injection
element.innerHTML = userInput; // ❌ XSS vulnerability
```

### Command Injection Prevention
```typescript
// SECURE: Avoid shell execution, use libraries
import { execFile } from 'child_process';

execFile('convert', [inputFile, outputFile], (error, stdout) => {
    // Safe: No shell interpretation
});

// INSECURE: Shell execution with user input
import { exec } from 'child_process';

exec(`convert ${userFile} output.png`, (error, stdout) => {
    // ❌ Command injection if userFile = "input.png; rm -rf /"
});
```

### Path Traversal Prevention
```typescript
// SECURE: Validate and normalize paths
import path from 'path';

function serveFile(filename: string) {
    const baseDir = '/var/www/uploads';
    const fullPath = path.normalize(path.join(baseDir, filename));

    // Ensure path is within baseDir
    if (!fullPath.startsWith(baseDir)) {
        throw new Error('Invalid file path');
    }

    return fs.readFile(fullPath);
}

// INSECURE:
function serveFile(filename: string) {
    return fs.readFile(`/var/www/uploads/${filename}`);
    // ❌ filename = "../../../etc/passwd" would expose system files
}
```

---

## 4. CRYPTOGRAPHY PATTERNS

### Password Hashing
```typescript
// SECURE: Modern password hashing
import bcrypt from 'bcrypt';
import argon2 from 'argon2';

// bcrypt (good)
const hash = await bcrypt.hash(password, 12); // 12 rounds
const valid = await bcrypt.compare(password, hash);

// Argon2 (better, won Password Hashing Competition)
const hash = await argon2.hash(password);
const valid = await argon2.verify(hash, password);

// INSECURE:
const hash = crypto.createHash('md5').update(password).digest('hex');
// ❌ MD5 is broken
// ❌ No salt
// ❌ Too fast (enables brute force)
```

### Data Encryption
```typescript
// SECURE: AES-256-GCM with proper key management
import crypto from 'crypto';

function encrypt(plaintext: string, key: Buffer): EncryptedData {
    const iv = crypto.randomBytes(16);
    const cipher = crypto.createCipheriv('aes-256-gcm', key, iv);

    let ciphertext = cipher.update(plaintext, 'utf8', 'hex');
    ciphertext += cipher.final('hex');
    const authTag = cipher.getAuthTag();

    return { ciphertext, iv: iv.toString('hex'), authTag: authTag.toString('hex') };
}

// INSECURE:
function encrypt(plaintext: string): string {
    const cipher = crypto.createCipher('des', 'my-password');
    // ❌ DES is broken
    // ❌ Password as key (use proper key derivation)
    // ❌ No IV
    // ❌ No authentication (use GCM or similar)
    return cipher.update(plaintext, 'utf8', 'hex') + cipher.final('hex');
}
```

### Secure Random Generation
```typescript
// SECURE: Cryptographically secure random
import crypto from 'crypto';

const token = crypto.randomBytes(32).toString('hex'); // 256 bits
const uuid = crypto.randomUUID(); // RFC 4122 v4

// INSECURE:
const token = Math.random().toString(36);
// ❌ Math.random() is NOT cryptographically secure
```

---

## 5. API SECURITY PATTERNS

### Rate Limiting
```typescript
// SECURE: Rate limiting to prevent abuse
import rateLimit from 'express-rate-limit';

const limiter = rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: 100, // Limit each IP to 100 requests per windowMs
    standardHeaders: true,
    legacyHeaders: false,
});

app.use('/api/', limiter);

// Stricter for auth endpoints
const authLimiter = rateLimit({
    windowMs: 15 * 60 * 1000,
    max: 5, // Only 5 attempts
    skipSuccessfulRequests: true
});

app.post('/api/login', authLimiter, login);
```

### CORS Configuration
```typescript
// SECURE: Restrictive CORS
app.use(cors({
    origin: ['https://example.com', 'https://app.example.com'],
    credentials: true,
    methods: ['GET', 'POST', 'PUT', 'DELETE'],
    allowedHeaders: ['Content-Type', 'Authorization'],
    exposedHeaders: ['X-Total-Count'],
    maxAge: 600 // 10 minutes
}));

// INSECURE:
app.use(cors());
// ❌ Allows all origins
// ❌ Default configuration too permissive
```

### API Key Management
```typescript
// SECURE: API key with proper validation
function validateApiKey(req, res, next) {
    const apiKey = req.headers['x-api-key'];

    if (!apiKey) {
        return res.status(401).json({ error: 'API key required' });
    }

    // Hash before comparison (constant-time)
    const hashedKey = crypto.createHash('sha256').update(apiKey).digest('hex');

    if (!crypto.timingSafeEqual(
        Buffer.from(hashedKey),
        Buffer.from(process.env.API_KEY_HASH)
    )) {
        return res.status(401).json({ error: 'Invalid API key' });
    }

    next();
}

// INSECURE:
function validateApiKey(req, res, next) {
    if (req.headers['x-api-key'] !== 'hardcoded-key') {
        // ❌ Hardcoded key
        // ❌ Not constant-time comparison (timing attack)
        return res.status(401).send('Invalid key');
    }
    next();
}
```

---

## 6. SECURE CONFIGURATION

### Environment Variables
```typescript
// SECURE: Dotenv with validation
import dotenv from 'dotenv';
import { z } from 'zod';

dotenv.config();

const EnvSchema = z.object({
    DATABASE_URL: z.string().url(),
    JWT_SECRET: z.string().min(32),
    NODE_ENV: z.enum(['development', 'production', 'test']),
    PORT: z.string().transform(Number).pipe(z.number().int().positive())
});

const env = EnvSchema.parse(process.env);
// Throws if validation fails

// INSECURE:
const dbUrl = process.env.DATABASE_URL || 'mongodb://localhost';
// ❌ No validation
// ❌ Default value might not be safe
```

### Secrets Management
```
✅ SECURE:
- Store in environment variables (12-factor app)
- Use secrets management (AWS Secrets Manager, HashiCorp Vault)
- Rotate regularly
- Never commit to version control
- Use .env.example for documentation

❌ INSECURE:
- Hardcoding in source code
- Committing .env files
- Storing in plain text
- Same secrets across environments
```

---

## 7. LOGGING & MONITORING

### Secure Logging
```typescript
// SECURE: Sanitized logging
import winston from 'winston';
import { redactPII } from './utils';

const logger = winston.createLogger({
    format: winston.format.combine(
        winston.format.timestamp(),
        winston.format.json()
    ),
    transports: [
        new winston.transports.File({ filename: 'app.log' })
    ]
});

function logRequest(req) {
    logger.info('API request', {
        method: req.method,
        path: req.path,
        userId: req.user?.id,
        ip: req.ip,
        // ✅ Redact sensitive data
        body: redactPII(req.body, ['password', 'ssn', 'creditCard'])
    });
}

// INSECURE:
console.log('User login:', req.body);
// ❌ Logs passwords in plain text
// ❌ No structure
// ❌ No timestamp
// ❌ No log levels
```

---

## 8. COMMON VULNERABILITY CHECKLIST

### Before Deployment:
- [ ] All user inputs validated and sanitized
- [ ] SQL queries use parameterized statements
- [ ] Passwords hashed with bcrypt/Argon2 (≥12 rounds)
- [ ] Secrets in environment variables (not code)
- [ ] HTTPS enforced (HSTS headers)
- [ ] CORS properly configured
- [ ] CSP headers set
- [ ] Rate limiting on API endpoints
- [ ] Authentication on all protected routes
- [ ] Authorization checks before sensitive operations
- [ ] Error messages don't leak implementation details
- [ ] Dependencies updated (no known CVEs)
- [ ] Logging doesn't include sensitive data
- [ ] File uploads validated and scanned
- [ ] Session cookies: secure, httpOnly, sameSite
- [ ] API keys rotated and not in version control

---

**Updated:** November 23, 2025
**Version:** 3.3.1
**Part of:** SENA Multi-Level Memory System
