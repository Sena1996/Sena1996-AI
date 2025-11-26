pub mod backend;
pub mod iot;
pub mod ios;
pub mod android;
pub mod web;

use serde::{Deserialize, Serialize};

pub use backend::BackendAgent;
pub use iot::IoTAgent;
pub use ios::IOSAgent;
pub use android::AndroidAgent;
pub use web::WebAgent;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainAgentType {
    Backend,
    IoT,
    IOS,
    Android,
    Web,
}

impl std::fmt::Display for DomainAgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainAgentType::Backend => write!(f, "🔧 Backend Agent"),
            DomainAgentType::IoT => write!(f, "🔌 IoT Agent"),
            DomainAgentType::IOS => write!(f, "🍎 iOS Agent"),
            DomainAgentType::Android => write!(f, "🤖 Android Agent"),
            DomainAgentType::Web => write!(f, "🌐 Web Agent"),
        }
    }
}

impl DomainAgentType {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "backend" | "api" | "server" => Some(DomainAgentType::Backend),
            "iot" | "embedded" | "device" => Some(DomainAgentType::IoT),
            "ios" | "swift" | "apple" => Some(DomainAgentType::IOS),
            "android" | "kotlin" | "java" => Some(DomainAgentType::Android),
            "web" | "frontend" | "react" | "vue" => Some(DomainAgentType::Web),
            _ => None,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DomainAgentType::Backend => "API mapping, data flow analysis, auth audit, security scanning",
            DomainAgentType::IoT => "Protocol analysis, device debugging, power optimization, connectivity",
            DomainAgentType::IOS => "HIG compliance, UI/UX validation, performance, accessibility",
            DomainAgentType::Android => "Material Design, ANR prevention, lifecycle, device compatibility",
            DomainAgentType::Web => "Core Web Vitals, accessibility, SEO, performance optimization",
        }
    }

    pub fn commands(&self) -> Vec<&'static str> {
        match self {
            DomainAgentType::Backend => vec!["map", "flow", "auth", "secrets", "security", "endpoints"],
            DomainAgentType::IoT => vec!["protocol", "debug", "power", "connect", "sensor", "firmware"],
            DomainAgentType::IOS => vec!["ui", "hig", "perf", "a11y", "device", "memory"],
            DomainAgentType::Android => vec!["ui", "material", "perf", "lifecycle", "compat", "a11y"],
            DomainAgentType::Web => vec!["vitals", "a11y", "seo", "bundle", "perf", "audit"],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAnalysis {
    pub agent: DomainAgentType,
    pub category: String,
    pub findings: Vec<Finding>,
    pub recommendations: Vec<String>,
    pub score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    Warning,
    Info,
    Success,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "🔴"),
            Severity::Warning => write!(f, "🟡"),
            Severity::Info => write!(f, "🔵"),
            Severity::Success => write!(f, "✅"),
        }
    }
}

impl DomainAnalysis {
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str("╔══════════════════════════════════════════════════════════════╗\n");
        output.push_str(&format!("║  {} - {}                          \n", self.agent, self.category));
        output.push_str("╚══════════════════════════════════════════════════════════════╝\n\n");

        output.push_str("════════════════════════════════════════════════════════════════\n");
        output.push_str("  FINDINGS\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        for finding in &self.findings {
            output.push_str(&format!("{} {}\n", finding.severity, finding.title));
            output.push_str(&format!("   {}\n", finding.description));
            if let Some(loc) = &finding.location {
                output.push_str(&format!("   📍 {}\n", loc));
            }
            if let Some(sug) = &finding.suggestion {
                output.push_str(&format!("   💡 {}\n", sug));
            }
            output.push('\n');
        }

        output.push_str("════════════════════════════════════════════════════════════════\n");
        output.push_str("  RECOMMENDATIONS\n");
        output.push_str("════════════════════════════════════════════════════════════════\n\n");

        for (i, rec) in self.recommendations.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, rec));
        }

        output.push_str(&format!("\n📊 Score: {}/100\n", self.score));

        output
    }
}

pub struct DomainAgentPool {
    backend: BackendAgent,
    iot: IoTAgent,
    ios: IOSAgent,
    android: AndroidAgent,
    web: WebAgent,
}

impl DomainAgentPool {
    pub fn new() -> Self {
        Self {
            backend: BackendAgent::new(),
            iot: IoTAgent::new(),
            ios: IOSAgent::new(),
            android: AndroidAgent::new(),
            web: WebAgent::new(),
        }
    }

    pub fn analyze(&self, agent_type: DomainAgentType, command: &str, input: &str) -> DomainAnalysis {
        match agent_type {
            DomainAgentType::Backend => self.backend.analyze(command, input),
            DomainAgentType::IoT => self.iot.analyze(command, input),
            DomainAgentType::IOS => self.ios.analyze(command, input),
            DomainAgentType::Android => self.android.analyze(command, input),
            DomainAgentType::Web => self.web.analyze(command, input),
        }
    }

    pub fn list_agents(&self) -> Vec<DomainAgentType> {
        vec![
            DomainAgentType::Backend,
            DomainAgentType::IoT,
            DomainAgentType::IOS,
            DomainAgentType::Android,
            DomainAgentType::Web,
        ]
    }
}

impl Default for DomainAgentPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_agent_type_parse() {
        assert_eq!(DomainAgentType::parse("backend"), Some(DomainAgentType::Backend));
        assert_eq!(DomainAgentType::parse("ios"), Some(DomainAgentType::IOS));
        assert_eq!(DomainAgentType::parse("android"), Some(DomainAgentType::Android));
        assert_eq!(DomainAgentType::parse("iot"), Some(DomainAgentType::IoT));
        assert_eq!(DomainAgentType::parse("web"), Some(DomainAgentType::Web));
    }

    #[test]
    fn test_domain_agent_pool() {
        let pool = DomainAgentPool::new();
        assert_eq!(pool.list_agents().len(), 5);
    }
}
