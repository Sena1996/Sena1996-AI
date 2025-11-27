use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{CollabError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteChoice {
    Approve,
    Reject,
    Abstain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusStrategy {
    Unanimous,
    Majority,
    SuperMajority,
    WeightedMajority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalState {
    Pending,
    Voting,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter_id: String,
    pub choice: VoteChoice,
    pub weight: f32,
    pub reasoning: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Vote {
    pub fn new(voter_id: &str, choice: VoteChoice) -> Self {
        Self {
            voter_id: voter_id.to_string(),
            choice,
            weight: 1.0,
            reasoning: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 10.0);
        self
    }

    pub fn with_reasoning(mut self, reasoning: &str) -> Self {
        self.reasoning = Some(reasoning.to_string());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub session_id: String,
    pub proposer_id: String,
    pub title: String,
    pub description: String,
    pub state: ProposalState,
    pub strategy: ConsensusStrategy,
    pub required_voters: Vec<String>,
    pub votes: HashMap<String, Vote>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

impl Proposal {
    pub fn new(session_id: &str, proposer_id: &str, title: &str, description: &str) -> Self {
        let proposal_id = format!(
            "prop_{}",
            Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );

        Self {
            id: proposal_id,
            session_id: session_id.to_string(),
            proposer_id: proposer_id.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            state: ProposalState::Pending,
            strategy: ConsensusStrategy::Majority,
            required_voters: Vec::new(),
            votes: HashMap::new(),
            created_at: chrono::Utc::now(),
            deadline: None,
        }
    }

    pub fn with_strategy(mut self, strategy: ConsensusStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_voters(mut self, voters: Vec<String>) -> Self {
        self.required_voters = voters;
        self
    }

    pub fn with_deadline(mut self, deadline: chrono::DateTime<chrono::Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn start_voting(&mut self) -> Result<()> {
        if self.state != ProposalState::Pending {
            return Err(CollabError::InvalidState(
                "Proposal must be pending to start voting".into(),
            ));
        }
        self.state = ProposalState::Voting;
        Ok(())
    }

    pub fn cast_vote(&mut self, vote: Vote) -> Result<()> {
        if self.state != ProposalState::Voting {
            return Err(CollabError::InvalidState(
                "Voting is not open for this proposal".into(),
            ));
        }

        if self.votes.contains_key(&vote.voter_id) {
            return Err(CollabError::PermissionDenied(
                "Agent has already voted".into(),
            ));
        }

        if !self.required_voters.is_empty() && !self.required_voters.contains(&vote.voter_id) {
            return Err(CollabError::PermissionDenied(
                "Agent is not authorized to vote".into(),
            ));
        }

        self.votes.insert(vote.voter_id.clone(), vote);

        if self.all_votes_received() {
            self.finalize_voting();
        }

        Ok(())
    }

    pub fn all_votes_received(&self) -> bool {
        if self.required_voters.is_empty() {
            return false;
        }
        self.required_voters
            .iter()
            .all(|v| self.votes.contains_key(v))
    }

    pub fn finalize_voting(&mut self) {
        if self.state != ProposalState::Voting {
            return;
        }

        let result = self.calculate_result();
        self.state = if result.approved {
            ProposalState::Approved
        } else {
            ProposalState::Rejected
        };
    }

    pub fn calculate_result(&self) -> ConsensusResult {
        let total_votes = self.votes.len();
        let mut approve_weight = 0.0;
        let mut reject_weight = 0.0;
        let mut abstain_count = 0;

        for vote in self.votes.values() {
            match vote.choice {
                VoteChoice::Approve => approve_weight += vote.weight,
                VoteChoice::Reject => reject_weight += vote.weight,
                VoteChoice::Abstain => abstain_count += 1,
            }
        }

        let total_weight = approve_weight + reject_weight;
        let approval_ratio = if total_weight > 0.0 {
            approve_weight / total_weight
        } else {
            0.0
        };

        let threshold = match self.strategy {
            ConsensusStrategy::Unanimous => 1.0,
            ConsensusStrategy::Majority => 0.5,
            ConsensusStrategy::SuperMajority => 0.67,
            ConsensusStrategy::WeightedMajority => 0.5,
        };

        let approved = match self.strategy {
            ConsensusStrategy::Unanimous => self
                .votes
                .values()
                .filter(|v| v.choice != VoteChoice::Abstain)
                .all(|v| v.choice == VoteChoice::Approve),
            _ => approval_ratio > threshold,
        };

        ConsensusResult {
            proposal_id: self.id.clone(),
            approved,
            total_votes,
            approve_weight,
            reject_weight,
            abstain_count,
            approval_ratio,
            threshold,
        }
    }

    pub fn check_expiration(&mut self) {
        if let Some(deadline) = self.deadline {
            if chrono::Utc::now() > deadline && self.state == ProposalState::Voting {
                self.state = ProposalState::Expired;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    pub proposal_id: String,
    pub approved: bool,
    pub total_votes: usize,
    pub approve_weight: f32,
    pub reject_weight: f32,
    pub abstain_count: usize,
    pub approval_ratio: f32,
    pub threshold: f32,
}

#[derive(Debug, Default)]
pub struct ConsensusManager {
    proposals: HashMap<String, Proposal>,
}

impl ConsensusManager {
    pub fn new() -> Self {
        Self {
            proposals: HashMap::new(),
        }
    }

    pub fn create_proposal(&mut self, proposal: Proposal) -> Option<&Proposal> {
        let proposal_id = proposal.id.clone();
        self.proposals.insert(proposal_id.clone(), proposal);
        self.proposals.get(&proposal_id)
    }

    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    pub fn get_proposal_mut(&mut self, proposal_id: &str) -> Option<&mut Proposal> {
        self.proposals.get_mut(proposal_id)
    }

    pub fn session_proposals(&self, session_id: &str) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.session_id == session_id)
            .collect()
    }

    pub fn pending_proposals(&self, session_id: &str) -> Vec<&Proposal> {
        self.proposals
            .values()
            .filter(|p| p.session_id == session_id && p.state == ProposalState::Voting)
            .collect()
    }

    pub fn check_expirations(&mut self) {
        for proposal in self.proposals.values_mut() {
            proposal.check_expiration();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_creation() {
        let proposal = Proposal::new("session_1", "agent_1", "Test Proposal", "A test proposal");

        assert_eq!(proposal.state, ProposalState::Pending);
        assert!(proposal.id.starts_with("prop_"));
    }

    #[test]
    fn test_vote_casting() {
        let mut proposal = Proposal::new("session_1", "agent_1", "Test", "Test proposal")
            .with_voters(vec!["agent_1".into(), "agent_2".into()]);

        proposal.start_voting().expect("should start voting");

        let vote = Vote::new("agent_1", VoteChoice::Approve);
        proposal.cast_vote(vote).expect("should cast vote");

        assert_eq!(proposal.votes.len(), 1);
    }

    #[test]
    fn test_majority_consensus() {
        let mut proposal = Proposal::new("session_1", "agent_1", "Test", "Test")
            .with_strategy(ConsensusStrategy::Majority)
            .with_voters(vec!["agent_1".into(), "agent_2".into(), "agent_3".into()]);

        proposal.start_voting().expect("should start");

        proposal
            .cast_vote(Vote::new("agent_1", VoteChoice::Approve))
            .expect("vote 1");
        proposal
            .cast_vote(Vote::new("agent_2", VoteChoice::Approve))
            .expect("vote 2");
        proposal
            .cast_vote(Vote::new("agent_3", VoteChoice::Reject))
            .expect("vote 3");

        let result = proposal.calculate_result();
        assert!(result.approved);
        assert!(result.approval_ratio > 0.5);
    }

    #[test]
    fn test_unanimous_consensus() {
        let mut proposal = Proposal::new("session_1", "agent_1", "Test", "Test")
            .with_strategy(ConsensusStrategy::Unanimous)
            .with_voters(vec!["agent_1".into(), "agent_2".into()]);

        proposal.start_voting().expect("should start");

        proposal
            .cast_vote(Vote::new("agent_1", VoteChoice::Approve))
            .expect("vote 1");
        proposal
            .cast_vote(Vote::new("agent_2", VoteChoice::Reject))
            .expect("vote 2");

        let result = proposal.calculate_result();
        assert!(!result.approved);
    }

    #[test]
    fn test_weighted_voting() {
        let mut proposal = Proposal::new("session_1", "agent_1", "Test", "Test")
            .with_strategy(ConsensusStrategy::WeightedMajority)
            .with_voters(vec!["expert".into(), "novice".into()]);

        proposal.start_voting().expect("should start");

        let expert_vote = Vote::new("expert", VoteChoice::Approve).with_weight(2.0);
        let novice_vote = Vote::new("novice", VoteChoice::Reject).with_weight(1.0);

        proposal.cast_vote(expert_vote).expect("expert vote");
        proposal.cast_vote(novice_vote).expect("novice vote");

        let result = proposal.calculate_result();
        assert!(result.approved);
        assert!(result.approve_weight > result.reject_weight);
    }

    #[test]
    fn test_double_vote_prevention() {
        let mut proposal = Proposal::new("session_1", "agent_1", "Test", "Test")
            .with_voters(vec!["agent_1".into()]);

        proposal.start_voting().expect("should start");

        proposal
            .cast_vote(Vote::new("agent_1", VoteChoice::Approve))
            .expect("first vote");

        let result = proposal.cast_vote(Vote::new("agent_1", VoteChoice::Reject));
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_finalize() {
        let mut proposal = Proposal::new("session_1", "agent_1", "Test", "Test")
            .with_voters(vec!["agent_1".into()]);

        proposal.start_voting().expect("should start");

        proposal
            .cast_vote(Vote::new("agent_1", VoteChoice::Approve))
            .expect("vote");

        assert_eq!(proposal.state, ProposalState::Approved);
    }

    #[test]
    fn test_consensus_manager() {
        let mut manager = ConsensusManager::new();

        let proposal = Proposal::new("session_1", "agent_1", "Test", "Test");
        manager.create_proposal(proposal);

        assert_eq!(manager.session_proposals("session_1").len(), 1);
    }
}
