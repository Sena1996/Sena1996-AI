use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::agent::AgentInfo;
use crate::error::Result;

type SelectionResult = Option<(String, f32, Vec<(String, f32)>)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskDomain {
    CodeGeneration,
    CodeReview,
    Documentation,
    Testing,
    Security,
    Performance,
    Architecture,
    DataAnalysis,
    NaturalLanguage,
    Mathematics,
    Research,
    Creative,
    General,
}

impl TaskDomain {
    pub fn from_keywords(text: &str) -> Self {
        let lower = text.to_lowercase();

        if lower.contains("security")
            || lower.contains("vulnerab")
            || lower.contains("exploit")
            || lower.contains("cve")
        {
            return TaskDomain::Security;
        }

        if lower.contains("performance")
            || lower.contains("optimize")
            || lower.contains("benchmark")
            || lower.contains("latency")
        {
            return TaskDomain::Performance;
        }

        if lower.contains("architecture")
            || lower.contains("design pattern")
            || lower.contains("system design")
        {
            return TaskDomain::Architecture;
        }

        if lower.contains("test")
            || lower.contains("spec")
            || lower.contains("coverage")
            || lower.contains("mock")
        {
            return TaskDomain::Testing;
        }

        if lower.contains("document")
            || lower.contains("readme")
            || lower.contains("api doc")
            || lower.contains("comment")
        {
            return TaskDomain::Documentation;
        }

        if lower.contains("review") || lower.contains("refactor") || lower.contains("code quality")
        {
            return TaskDomain::CodeReview;
        }

        if lower.contains("implement")
            || lower.contains("create function")
            || lower.contains("write code")
            || lower.contains("generate")
        {
            return TaskDomain::CodeGeneration;
        }

        if lower.contains("data")
            || lower.contains("analyz")
            || lower.contains("statistic")
            || lower.contains("chart")
        {
            return TaskDomain::DataAnalysis;
        }

        if lower.contains("math")
            || lower.contains("equation")
            || lower.contains("calcul")
            || lower.contains("proof")
        {
            return TaskDomain::Mathematics;
        }

        if lower.contains("research")
            || lower.contains("investigate")
            || lower.contains("find information")
        {
            return TaskDomain::Research;
        }

        if lower.contains("creative")
            || lower.contains("story")
            || lower.contains("design")
            || lower.contains("brainstorm")
        {
            return TaskDomain::Creative;
        }

        if lower.contains("translate")
            || lower.contains("summarize")
            || lower.contains("explain")
            || lower.contains("write")
        {
            return TaskDomain::NaturalLanguage;
        }

        TaskDomain::General
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistProfile {
    pub agent_id: String,
    pub provider: String,
    pub model: String,
    pub specialties: Vec<TaskDomain>,
    pub expertise_scores: HashMap<TaskDomain, f32>,
    pub load: f32,
    pub available: bool,
}

impl SpecialistProfile {
    pub fn new(agent: &AgentInfo) -> Self {
        Self {
            agent_id: agent.id.clone(),
            provider: agent.provider.clone(),
            model: agent.model.clone(),
            specialties: Vec::new(),
            expertise_scores: HashMap::new(),
            load: 0.0,
            available: true,
        }
    }

    pub fn with_specialty(mut self, domain: TaskDomain, score: f32) -> Self {
        self.specialties.push(domain);
        self.expertise_scores.insert(domain, score.clamp(0.0, 1.0));
        self
    }

    pub fn expertise_for(&self, domain: TaskDomain) -> f32 {
        self.expertise_scores.get(&domain).copied().unwrap_or(0.5)
    }

    pub fn effective_score(&self, domain: TaskDomain) -> f32 {
        let base_score = self.expertise_for(domain);
        let load_penalty = self.load * 0.3;
        (base_score - load_penalty).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingStrategy {
    BestMatch,
    RoundRobin,
    LeastLoaded,
    Random,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub task_description: String,
    pub detected_domain: TaskDomain,
    pub selected_agent: String,
    pub score: f32,
    pub alternatives: Vec<(String, f32)>,
    pub reasoning: String,
}

pub struct SpecialistRouter {
    specialists: HashMap<String, SpecialistProfile>,
    strategy: RoutingStrategy,
    round_robin_index: usize,
}

impl SpecialistRouter {
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self {
            specialists: HashMap::new(),
            strategy,
            round_robin_index: 0,
        }
    }

    pub fn register_specialist(&mut self, profile: SpecialistProfile) {
        self.specialists.insert(profile.agent_id.clone(), profile);
    }

    pub fn update_load(&mut self, agent_id: &str, load: f32) {
        if let Some(profile) = self.specialists.get_mut(agent_id) {
            profile.load = load.clamp(0.0, 1.0);
        }
    }

    pub fn set_availability(&mut self, agent_id: &str, available: bool) {
        if let Some(profile) = self.specialists.get_mut(agent_id) {
            profile.available = available;
        }
    }

    pub fn route_task(&mut self, task_description: &str) -> Result<RoutingDecision> {
        let domain = TaskDomain::from_keywords(task_description);

        let available_ids: Vec<String> = self
            .specialists
            .values()
            .filter(|s| s.available)
            .map(|s| s.agent_id.clone())
            .collect();

        if available_ids.is_empty() {
            return Err(crate::error::CollabError::AgentUnavailable(
                "No specialists available".into(),
            ));
        }

        let selection_result = match self.strategy {
            RoutingStrategy::BestMatch => self.select_best_match(&available_ids, domain),
            RoutingStrategy::RoundRobin => self.select_round_robin(&available_ids, domain),
            RoutingStrategy::LeastLoaded => self.select_least_loaded(&available_ids, domain),
            RoutingStrategy::Random => self.select_random(&available_ids, domain),
        };

        let (selected_id, score, alternatives) = selection_result.ok_or_else(|| {
            crate::error::CollabError::AgentUnavailable(
                "Failed to select specialist from available pool".into(),
            )
        })?;

        let reasoning = format!(
            "Selected {} (score: {:.2}) for {} task using {:?} strategy",
            selected_id,
            score,
            format!("{:?}", domain).to_lowercase(),
            self.strategy
        );

        Ok(RoutingDecision {
            task_description: task_description.to_string(),
            detected_domain: domain,
            selected_agent: selected_id,
            score,
            alternatives,
            reasoning,
        })
    }

    fn select_best_match(&self, available_ids: &[String], domain: TaskDomain) -> SelectionResult {
        let mut scored: Vec<(String, f32)> = available_ids
            .iter()
            .filter_map(|id| {
                self.specialists
                    .get(id)
                    .map(|s| (id.clone(), s.effective_score(domain)))
            })
            .collect();

        if scored.is_empty() {
            return None;
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let selected = scored.first()?.clone();
        let alternatives: Vec<(String, f32)> = scored.into_iter().skip(1).take(3).collect();

        Some((selected.0, selected.1, alternatives))
    }

    fn select_round_robin(
        &mut self,
        available_ids: &[String],
        domain: TaskDomain,
    ) -> SelectionResult {
        if available_ids.is_empty() {
            return None;
        }

        self.round_robin_index = (self.round_robin_index + 1) % available_ids.len();
        let selected_id = available_ids.get(self.round_robin_index)?.clone();

        let score = self
            .specialists
            .get(&selected_id)
            .map(|s| s.effective_score(domain))
            .unwrap_or(0.5);

        let alternatives: Vec<(String, f32)> = available_ids
            .iter()
            .filter(|id| **id != selected_id)
            .take(3)
            .filter_map(|id| {
                self.specialists
                    .get(id)
                    .map(|s| (id.clone(), s.effective_score(domain)))
            })
            .collect();

        Some((selected_id, score, alternatives))
    }

    fn select_least_loaded(&self, available_ids: &[String], domain: TaskDomain) -> SelectionResult {
        let mut with_loads: Vec<(String, f32, f32)> = available_ids
            .iter()
            .filter_map(|id| {
                self.specialists
                    .get(id)
                    .map(|s| (id.clone(), s.load, s.effective_score(domain)))
            })
            .collect();

        if with_loads.is_empty() {
            return None;
        }

        with_loads.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let (selected_id, _, score) = with_loads.first()?.clone();
        let alternatives: Vec<(String, f32)> = with_loads
            .into_iter()
            .skip(1)
            .take(3)
            .map(|(id, _, s)| (id, s))
            .collect();

        Some((selected_id, score, alternatives))
    }

    fn select_random(&self, available_ids: &[String], domain: TaskDomain) -> SelectionResult {
        if available_ids.is_empty() {
            return None;
        }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        let timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        timestamp.hash(&mut hasher);
        let index = (hasher.finish() as usize) % available_ids.len();

        let selected_id = available_ids.get(index)?.clone();
        let score = self
            .specialists
            .get(&selected_id)
            .map(|s| s.effective_score(domain))
            .unwrap_or(0.5);

        let alternatives: Vec<(String, f32)> = available_ids
            .iter()
            .filter(|id| **id != selected_id)
            .take(3)
            .filter_map(|id| {
                self.specialists
                    .get(id)
                    .map(|s| (id.clone(), s.effective_score(domain)))
            })
            .collect();

        Some((selected_id, score, alternatives))
    }

    pub fn specialists(&self) -> Vec<&SpecialistProfile> {
        self.specialists.values().collect()
    }

    pub fn available_count(&self) -> usize {
        self.specialists.values().filter(|s| s.available).count()
    }
}

impl Default for SpecialistRouter {
    fn default() -> Self {
        Self::new(RoutingStrategy::BestMatch)
    }
}

pub fn create_default_profiles() -> Vec<SpecialistProfile> {
    vec![
        SpecialistProfile {
            agent_id: "claude-specialist".into(),
            provider: "claude".into(),
            model: "claude-sonnet-4-5".into(),
            specialties: vec![
                TaskDomain::CodeGeneration,
                TaskDomain::CodeReview,
                TaskDomain::Architecture,
            ],
            expertise_scores: [
                (TaskDomain::CodeGeneration, 0.95),
                (TaskDomain::CodeReview, 0.92),
                (TaskDomain::Architecture, 0.90),
                (TaskDomain::Documentation, 0.88),
                (TaskDomain::General, 0.85),
            ]
            .into_iter()
            .collect(),
            load: 0.0,
            available: true,
        },
        SpecialistProfile {
            agent_id: "gpt4-specialist".into(),
            provider: "openai".into(),
            model: "gpt-4.1".into(),
            specialties: vec![
                TaskDomain::NaturalLanguage,
                TaskDomain::Creative,
                TaskDomain::Research,
            ],
            expertise_scores: [
                (TaskDomain::NaturalLanguage, 0.94),
                (TaskDomain::Creative, 0.92),
                (TaskDomain::Research, 0.90),
                (TaskDomain::Documentation, 0.88),
                (TaskDomain::General, 0.85),
            ]
            .into_iter()
            .collect(),
            load: 0.0,
            available: true,
        },
        SpecialistProfile {
            agent_id: "gemini-specialist".into(),
            provider: "gemini".into(),
            model: "gemini-pro".into(),
            specialties: vec![
                TaskDomain::DataAnalysis,
                TaskDomain::Mathematics,
                TaskDomain::Research,
            ],
            expertise_scores: [
                (TaskDomain::DataAnalysis, 0.93),
                (TaskDomain::Mathematics, 0.91),
                (TaskDomain::Research, 0.89),
                (TaskDomain::General, 0.82),
            ]
            .into_iter()
            .collect(),
            load: 0.0,
            available: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_detection() {
        assert_eq!(
            TaskDomain::from_keywords("review this code for security vulnerabilities"),
            TaskDomain::Security
        );
        assert_eq!(
            TaskDomain::from_keywords("optimize the database queries for performance"),
            TaskDomain::Performance
        );
        assert_eq!(
            TaskDomain::from_keywords("write unit tests for the auth module"),
            TaskDomain::Testing
        );
        assert_eq!(
            TaskDomain::from_keywords("implement a new feature"),
            TaskDomain::CodeGeneration
        );
    }

    #[test]
    fn test_specialist_profile() {
        let agent = AgentInfo::new("claude", "claude-sonnet-4-5");
        let profile = SpecialistProfile::new(&agent)
            .with_specialty(TaskDomain::CodeGeneration, 0.95)
            .with_specialty(TaskDomain::Security, 0.80);

        assert_eq!(profile.expertise_for(TaskDomain::CodeGeneration), 0.95);
        assert_eq!(profile.expertise_for(TaskDomain::Security), 0.80);
        assert_eq!(profile.expertise_for(TaskDomain::Testing), 0.5);
    }

    #[test]
    fn test_effective_score_with_load() {
        let agent = AgentInfo::new("claude", "claude-sonnet-4-5");
        let mut profile =
            SpecialistProfile::new(&agent).with_specialty(TaskDomain::CodeGeneration, 0.90);

        profile.load = 0.5;
        let effective = profile.effective_score(TaskDomain::CodeGeneration);
        assert!(effective < 0.90);
        assert!(effective > 0.70);
    }

    #[test]
    fn test_router_best_match() {
        let mut router = SpecialistRouter::new(RoutingStrategy::BestMatch);

        let agent1 = AgentInfo::new("claude", "claude-sonnet-4-5");
        let profile1 =
            SpecialistProfile::new(&agent1).with_specialty(TaskDomain::CodeGeneration, 0.95);

        let agent2 = AgentInfo::new("openai", "gpt-4.1");
        let profile2 =
            SpecialistProfile::new(&agent2).with_specialty(TaskDomain::CodeGeneration, 0.70);

        router.register_specialist(profile1);
        router.register_specialist(profile2);

        let decision = router.route_task("implement a new function").unwrap();
        assert_eq!(decision.selected_agent, agent1.id);
    }

    #[test]
    fn test_router_least_loaded() {
        let mut router = SpecialistRouter::new(RoutingStrategy::LeastLoaded);

        let agent1 = AgentInfo::new("claude", "claude-sonnet-4-5");
        let mut profile1 =
            SpecialistProfile::new(&agent1).with_specialty(TaskDomain::General, 0.90);
        profile1.load = 0.8;

        let agent2 = AgentInfo::new("openai", "gpt-4.1");
        let mut profile2 =
            SpecialistProfile::new(&agent2).with_specialty(TaskDomain::General, 0.85);
        profile2.load = 0.2;

        router.register_specialist(profile1);
        router.register_specialist(profile2);

        let decision = router.route_task("general task").unwrap();
        assert_eq!(decision.selected_agent, agent2.id);
    }

    #[test]
    fn test_availability_filter() {
        let mut router = SpecialistRouter::new(RoutingStrategy::BestMatch);

        let agent1 = AgentInfo::new("claude", "claude-sonnet-4-5");
        let profile1 = SpecialistProfile::new(&agent1).with_specialty(TaskDomain::General, 0.95);

        let agent2 = AgentInfo::new("openai", "gpt-4.1");
        let profile2 = SpecialistProfile::new(&agent2).with_specialty(TaskDomain::General, 0.70);

        router.register_specialist(profile1);
        router.register_specialist(profile2);

        router.set_availability(&agent1.id, false);

        let decision = router.route_task("any task").unwrap();
        assert_eq!(decision.selected_agent, agent2.id);
    }

    #[test]
    fn test_default_profiles() {
        let profiles = create_default_profiles();
        assert_eq!(profiles.len(), 3);

        let claude = &profiles[0];
        assert!(claude.specialties.contains(&TaskDomain::CodeGeneration));
    }
}
