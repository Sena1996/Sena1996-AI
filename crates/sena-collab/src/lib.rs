// Existing Sena1996-AI modules
pub mod agent;
pub mod consensus;
pub mod error;
pub mod message;
pub mod orchestrator;
pub mod permission;
pub mod routing;
pub mod session;

// New modules from 1996AI
pub mod collaboration;
pub mod crdt;
pub mod detection;
pub mod document;
pub mod hub;
pub mod roles;
pub mod sync;
pub mod taskboard;
pub mod websocket;

// Existing exports
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

// New exports from 1996AI
pub use collaboration::{
    CollaborationMode, CollaborationSession, DriverNavigatorState,
    MessageType as CollabMessageType, NavigatorSuggestion, QueuedMessage,
    SessionState as CollabSessionState, SuggestionStatus, SuggestionType, TurnManager,
};
pub use crdt::{
    GCounter, LWWMap, LWWRegister, NodeId, ORSet, PNCounter, Timestamp, VectorClock,
};
pub use detection::{
    ColorSupport, EnvironmentDetector, EnvironmentFeatures, EnvironmentInfo, EnvironmentType,
    IdeType, JetBrainsProduct, ShellType,
};
pub use document::{CollaborativeSession, DocumentState, DocumentSynchronizer};
pub use hub::{Hub, HubMember, HubMessage};
pub use roles::{
    Action, Participant as RoleParticipant, ParticipantRole, PermissionSet as RolePermissionSet,
    RoleEnforcer,
};
pub use sync::{SyncClient, SyncMessage, SyncPeer, SyncServer};
pub use taskboard::{Task, TaskBoard, TaskComment, TaskPriority, TaskStatus};
pub use websocket::{WebSocketServer, WsMessage, WsOperation};
