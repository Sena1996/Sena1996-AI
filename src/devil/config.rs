use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SynthesisMethod {
    MajorityVoting,
    WeightedMerge,
    LongestCommonSubsequence,
    BestOfN,
    MetaLLM,
    #[default]
    CrossVerification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WaitMode {
    #[default]
    WaitForAll,
    EarlyReturn,
    Configurable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevilConfig {
    pub enabled: bool,
    pub timeout_secs: u64,
    pub min_providers: usize,
    pub synthesis_method: SynthesisMethod,
    pub consensus_threshold: f64,
    pub include_failed_in_output: bool,
    pub parallel_limit: Option<usize>,
    pub wait_mode: WaitMode,
    pub verification_enabled: bool,
    pub max_facts_per_response: usize,
}

impl Default for DevilConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_secs: 30,
            min_providers: 2,
            synthesis_method: SynthesisMethod::CrossVerification,
            consensus_threshold: 0.6,
            include_failed_in_output: false,
            parallel_limit: None,
            wait_mode: WaitMode::WaitForAll,
            verification_enabled: true,
            max_facts_per_response: 20,
        }
    }
}

impl DevilConfig {
    pub fn fast() -> Self {
        Self {
            timeout_secs: 15,
            synthesis_method: SynthesisMethod::BestOfN,
            wait_mode: WaitMode::EarlyReturn,
            verification_enabled: false,
            ..Default::default()
        }
    }

    pub fn thorough() -> Self {
        Self {
            timeout_secs: 60,
            synthesis_method: SynthesisMethod::CrossVerification,
            consensus_threshold: 0.7,
            wait_mode: WaitMode::WaitForAll,
            verification_enabled: true,
            max_facts_per_response: 30,
            ..Default::default()
        }
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn with_synthesis(mut self, method: SynthesisMethod) -> Self {
        self.synthesis_method = method;
        self
    }

    pub fn with_consensus_threshold(mut self, threshold: f64) -> Self {
        self.consensus_threshold = threshold;
        self
    }
}
