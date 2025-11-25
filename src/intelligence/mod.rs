mod thinking;
mod agents;
mod routing;
mod skills;

pub use thinking::{ThinkingEngine, ThinkingDepth, ThinkingResult};
pub use agents::{Agent, AgentType, AgentPool, AgentResult};
pub use routing::{ModelRouter, ModelType, RoutingDecision};
pub use skills::{Skill, SkillRegistry, SkillExecution};

use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct IntelligenceSystem {
    pub thinking: ThinkingEngine,
    pub agents: AgentPool,
    pub router: ModelRouter,
    pub skills: SkillRegistry,
}

impl IntelligenceSystem {
    pub fn new() -> Self {
        Self {
            thinking: ThinkingEngine::new(),
            agents: AgentPool::new(),
            router: ModelRouter::new(),
            skills: SkillRegistry::new(),
        }
    }

    pub fn analyze(&self, problem: &str, depth: ThinkingDepth) -> ThinkingResult {
        self.thinking.analyze(problem, depth)
    }

    pub fn dispatch(&self, task: &str, agent_type: AgentType) -> AgentResult {
        self.agents.dispatch(task, agent_type)
    }

    pub fn route(&self, task: &str) -> RoutingDecision {
        self.router.route(task)
    }

    pub fn execute_skill(&self, skill_name: &str, context: &str) -> Option<SkillExecution> {
        self.skills.execute(skill_name, context)
    }

    pub fn status(&self) -> IntelligenceStatus {
        IntelligenceStatus {
            thinking_available: true,
            agent_count: self.agents.count(),
            skill_count: self.skills.count(),
            router_mode: self.router.current_mode(),
        }
    }
}

impl Default for IntelligenceSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceStatus {
    pub thinking_available: bool,
    pub agent_count: usize,
    pub skill_count: usize,
    pub router_mode: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intelligence_system_creation() {
        let system = IntelligenceSystem::new();
        let status = system.status();
        assert!(status.thinking_available);
        assert!(status.agent_count > 0);
    }
}
