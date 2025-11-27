pub mod agent;
pub mod consensus;
pub mod error;
pub mod message;
pub mod orchestrator;
pub mod permission;
pub mod routing;
pub mod session;

pub use agent::{AgentCapability, AgentInfo, AgentRegistry, CollabAgent};
pub use consensus::{
    ConsensusManager, ConsensusResult, ConsensusStrategy, Proposal, ProposalState, Vote, VoteChoice,
};
pub use error::{CollabError, Result};
pub use message::{
    AgentStatus, CollabMessage, ContextOperation, ContextPayload, MessageContent, MessageMetadata,
    MessagePriority, MessageType, RequestPayload, RequestType, ResponsePayload, StatusPayload,
    ToolCallPayload, ToolResultPayload,
};
pub use orchestrator::{CollabOrchestrator, ParticipantSummary, SessionSummary};
pub use permission::{
    ApprovalStatus, Permission, PermissionApproval, PermissionRequest, PermissionSet,
};
pub use routing::{
    create_default_profiles, RoutingDecision, RoutingStrategy, SpecialistProfile, SpecialistRouter,
    TaskDomain,
};
pub use session::{
    CollabSession, Participant, SessionConfig, SessionManager, SessionState, SessionType,
};
