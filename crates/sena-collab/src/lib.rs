pub mod error;
pub mod permission;
pub mod message;
pub mod agent;
pub mod session;
pub mod orchestrator;
pub mod consensus;
pub mod routing;

pub use error::{CollabError, Result};
pub use permission::{Permission, PermissionSet, PermissionRequest, PermissionApproval, ApprovalStatus};
pub use message::{
    CollabMessage, MessageType, MessagePriority, MessageContent,
    RequestPayload, RequestType, ResponsePayload,
    ToolCallPayload, ToolResultPayload,
    ContextPayload, ContextOperation,
    StatusPayload, AgentStatus, MessageMetadata,
};
pub use agent::{AgentInfo, AgentCapability, AgentRegistry, CollabAgent};
pub use session::{CollabSession, SessionConfig, SessionState, SessionType, SessionManager, Participant};
pub use orchestrator::{CollabOrchestrator, SessionSummary, ParticipantSummary};
pub use consensus::{
    Vote, VoteChoice, Proposal, ProposalState, ConsensusStrategy,
    ConsensusResult, ConsensusManager,
};
pub use routing::{
    TaskDomain, SpecialistProfile, SpecialistRouter, RoutingStrategy,
    RoutingDecision, create_default_profiles,
};
