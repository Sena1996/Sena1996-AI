mod config;
mod error;
mod executor;
mod hallucination;
mod interceptor;
mod validator;

pub use config::{GuardianConfig, HallucinationMode, SandboxLevel};
pub use error::{GuardianError, GuardianResult};
pub use executor::{DirectExecutor, InlineExecutable};
pub use hallucination::{HallucinationDetector, HallucinationResult, HallucinationResponse};
pub use interceptor::{InterceptedOutput, StreamInterceptor};
pub use validator::{CommandValidator, ValidationResult};

use std::sync::{Arc, RwLock};

use crate::ancient::{HarmonyValidationEngine, NegativeSpaceArchitecture};

pub struct GuardianMiddleware {
    #[allow(dead_code)]
    negative_space: Arc<RwLock<NegativeSpaceArchitecture>>,
    #[allow(dead_code)]
    harmony_validator: Arc<RwLock<HarmonyValidationEngine>>,
    command_validator: CommandValidator,
    hallucination_detector: HallucinationDetector,
    direct_executor: DirectExecutor,
    config: GuardianConfig,
}

impl GuardianMiddleware {
    pub fn new() -> Self {
        let negative_space = Arc::new(RwLock::new(NegativeSpaceArchitecture::new()));
        let harmony_validator = Arc::new(RwLock::new(HarmonyValidationEngine::new()));

        Self {
            negative_space: Arc::clone(&negative_space),
            harmony_validator: Arc::clone(&harmony_validator),
            command_validator: CommandValidator::new(Arc::clone(&negative_space)),
            hallucination_detector: HallucinationDetector::new(
                Arc::clone(&negative_space),
                Arc::clone(&harmony_validator),
            ),
            direct_executor: DirectExecutor::new(),
            config: GuardianConfig::default(),
        }
    }

    pub fn with_config(config: GuardianConfig) -> Self {
        let negative_space = Arc::new(RwLock::new(NegativeSpaceArchitecture::new()));
        let harmony_validator = Arc::new(RwLock::new(HarmonyValidationEngine::new()));

        Self {
            negative_space: Arc::clone(&negative_space),
            harmony_validator: Arc::clone(&harmony_validator),
            command_validator: CommandValidator::new(Arc::clone(&negative_space)),
            hallucination_detector: HallucinationDetector::with_threshold(
                Arc::clone(&negative_space),
                Arc::clone(&harmony_validator),
                config.hallucination_threshold,
            ),
            direct_executor: DirectExecutor::new(),
            config,
        }
    }

    pub fn validate_command(&self, command: &str) -> ValidationResult {
        self.command_validator.validate(command)
    }

    pub fn check_hallucination(&self, content: &str) -> HallucinationResult {
        self.hallucination_detector.check(content)
    }

    pub fn execute(&self, command: &str, args: &[&str]) -> GuardianResult<std::process::Output> {
        let validation = self.validate_command(command);
        if !validation.allowed {
            return Err(GuardianError::ExecutionBlocked(
                validation.reason.unwrap_or_else(|| "Blocked by policy".to_string()),
            ));
        }

        self.direct_executor.execute(command, args)
    }

    pub fn intercept_output(&self, output: &str) -> InterceptedOutput {
        let hallucination_check = self.check_hallucination(output);

        let response = match hallucination_check.response {
            HallucinationResponse::Block => {
                return InterceptedOutput {
                    content: "[BLOCKED: Potential hallucination detected]".to_string(),
                    original: output.to_string(),
                    was_blocked: true,
                    hallucination_score: hallucination_check.risk_score,
                    warnings: vec![format!(
                        "Content blocked due to high hallucination risk: {:.2}",
                        hallucination_check.risk_score
                    )],
                };
            }
            HallucinationResponse::Warn => {
                let mut warnings = vec![format!(
                    "Warning: Possible hallucination detected (score: {:.2})",
                    hallucination_check.risk_score
                )];
                warnings.extend(hallucination_check.warnings);
                InterceptedOutput {
                    content: output.to_string(),
                    original: output.to_string(),
                    was_blocked: false,
                    hallucination_score: hallucination_check.risk_score,
                    warnings,
                }
            }
            HallucinationResponse::Log | HallucinationResponse::Pass => InterceptedOutput {
                content: output.to_string(),
                original: output.to_string(),
                was_blocked: false,
                hallucination_score: hallucination_check.risk_score,
                warnings: hallucination_check.warnings,
            },
        };

        response
    }

    pub fn config(&self) -> &GuardianConfig {
        &self.config
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

impl Default for GuardianMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guardian_creation() {
        let guardian = GuardianMiddleware::new();
        assert!(guardian.is_enabled());
    }

    #[test]
    fn test_command_validation() {
        let guardian = GuardianMiddleware::new();

        let safe_result = guardian.validate_command("ls");
        assert!(safe_result.allowed);

        let dangerous_result = guardian.validate_command("rm -rf /");
        assert!(!dangerous_result.allowed);
    }

    #[test]
    fn test_output_interception() {
        let guardian = GuardianMiddleware::new();

        let output = "This is a normal response about Rust programming.";
        let intercepted = guardian.intercept_output(output);

        assert!(!intercepted.was_blocked);
        assert_eq!(intercepted.content, output);
    }
}
