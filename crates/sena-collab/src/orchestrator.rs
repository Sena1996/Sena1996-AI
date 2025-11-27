use std::sync::Arc;
use tokio::sync::RwLock;

use sena_providers::{AIProvider, ChatRequest, Message};

use crate::{
    agent::AgentInfo,
    error::{CollabError, Result},
    message::{AgentStatus, CollabMessage, MessageContent, RequestPayload, ResponsePayload},
    permission::{Permission, PermissionSet},
    session::{CollabSession, SessionManager, SessionState},
};

pub struct CollabOrchestrator {
    session_manager: Arc<RwLock<SessionManager>>,
    providers: std::collections::HashMap<String, Arc<dyn AIProvider>>,
}

impl CollabOrchestrator {
    pub fn new(max_sessions: usize) -> Self {
        Self {
            session_manager: Arc::new(RwLock::new(SessionManager::new(max_sessions))),
            providers: std::collections::HashMap::new(),
        }
    }

    pub fn register_provider(&mut self, provider: Arc<dyn AIProvider>) {
        let id = provider.provider_id().to_string();
        self.providers.insert(id, provider);
    }

    pub async fn create_session(&self, name: &str, host_provider: &str) -> Result<String> {
        let provider = self
            .providers
            .get(host_provider)
            .ok_or_else(|| CollabError::AgentNotFound(host_provider.into()))?;

        let host = AgentInfo::new(provider.provider_id(), provider.default_model());

        let mut manager = self.session_manager.write().await;
        let session = manager.create_session(name, host)?;
        let session_id = session.id.clone();

        Ok(session_id)
    }

    pub async fn join_session(&self, session_id: &str, provider_id: &str) -> Result<String> {
        let provider = self
            .providers
            .get(provider_id)
            .ok_or_else(|| CollabError::AgentNotFound(provider_id.into()))?;

        let agent = AgentInfo::new(provider.provider_id(), provider.default_model());
        let agent_id = agent.id.clone();
        let permissions = PermissionSet::new(Permission::standard_agent());

        let mut manager = self.session_manager.write().await;
        let session = manager
            .get_session_mut(session_id)
            .ok_or_else(|| CollabError::SessionNotFound(session_id.into()))?;

        session.add_participant(agent, permissions)?;

        Ok(agent_id)
    }

    pub async fn start_session(&self, session_id: &str) -> Result<()> {
        let mut manager = self.session_manager.write().await;
        let session = manager
            .get_session_mut(session_id)
            .ok_or_else(|| CollabError::SessionNotFound(session_id.into()))?;

        session.start()
    }

    pub async fn send_message(
        &self,
        session_id: &str,
        sender_id: &str,
        content: &str,
    ) -> Result<()> {
        let mut manager = self.session_manager.write().await;
        let session = manager
            .get_session_mut(session_id)
            .ok_or_else(|| CollabError::SessionNotFound(session_id.into()))?;

        if !session.is_active() {
            return Err(CollabError::InvalidState("Session is not active".into()));
        }

        if !session.has_permission(sender_id, Permission::SendMessages) {
            return Err(CollabError::PermissionDenied(
                "Agent cannot send messages".into(),
            ));
        }

        let message = CollabMessage::chat(session_id, sender_id, content);
        session.add_message(message);

        Ok(())
    }

    pub async fn broadcast_to_agents(
        &self,
        session_id: &str,
        sender_id: &str,
        content: &str,
    ) -> Result<Vec<CollabMessage>> {
        let session_clone;
        let providers_to_query: Vec<(String, String, Arc<dyn AIProvider>)>;

        {
            let manager = self.session_manager.read().await;
            let session = manager
                .get_session(session_id)
                .ok_or_else(|| CollabError::SessionNotFound(session_id.into()))?;

            if !session.is_active() {
                return Err(CollabError::InvalidState("Session is not active".into()));
            }

            session_clone = session.clone();

            providers_to_query = session
                .participants()
                .iter()
                .filter(|p| p.agent.id != sender_id && p.agent.is_available())
                .filter_map(|p| {
                    self.providers.get(&p.agent.provider).map(|provider| {
                        (p.agent.id.clone(), p.agent.model.clone(), provider.clone())
                    })
                })
                .collect();
        }

        let context = self.build_context(&session_clone, content);

        let mut responses = Vec::new();

        for (agent_id, model, provider) in providers_to_query {
            match self.get_agent_response(&provider, &model, &context).await {
                Ok(response_text) => {
                    let response = CollabMessage::chat(session_id, &agent_id, &response_text);
                    responses.push(response);
                }
                Err(e) => {
                    tracing::warn!("Failed to get response from {}: {}", agent_id, e);
                }
            }
        }

        {
            let mut manager = self.session_manager.write().await;
            if let Some(session) = manager.get_session_mut(session_id) {
                for response in &responses {
                    session.add_message(response.clone());
                }
            }
        }

        Ok(responses)
    }

    pub async fn request_analysis(
        &self,
        session_id: &str,
        requester_id: &str,
        target_provider: &str,
        request: RequestPayload,
    ) -> Result<CollabMessage> {
        let provider = self
            .providers
            .get(target_provider)
            .ok_or_else(|| CollabError::AgentNotFound(target_provider.into()))?;

        let session;
        {
            let manager = self.session_manager.read().await;
            session = manager
                .get_session(session_id)
                .ok_or_else(|| CollabError::SessionNotFound(session_id.into()))?
                .clone();
        }

        if !session.is_active() {
            return Err(CollabError::InvalidState("Session is not active".into()));
        }

        let request_msg = CollabMessage::request(session_id, requester_id, request.clone());

        let prompt = format!(
            "You are participating in a collaborative AI session. \
             Another AI has requested your analysis.\n\n\
             Request type: {:?}\n\
             Description: {}\n\
             Parameters: {}\n\n\
             Please provide your analysis.",
            request.request_type, request.description, request.parameters
        );

        let chat_request = ChatRequest::new(vec![Message::user(&prompt)]);

        let chat_response = provider.chat(chat_request).await?;

        let response_payload = ResponsePayload::success(&chat_response.content);
        let response_msg = CollabMessage::response(
            session_id,
            target_provider,
            request_msg.id,
            response_payload,
        );

        {
            let mut manager = self.session_manager.write().await;
            if let Some(session) = manager.get_session_mut(session_id) {
                session.add_message(request_msg);
                session.add_message(response_msg.clone());
            }
        }

        Ok(response_msg)
    }

    pub async fn get_session_summary(&self, session_id: &str) -> Result<SessionSummary> {
        let manager = self.session_manager.read().await;
        let session = manager
            .get_session(session_id)
            .ok_or_else(|| CollabError::SessionNotFound(session_id.into()))?;

        let participant_summaries: Vec<ParticipantSummary> = session
            .participants()
            .iter()
            .map(|p| ParticipantSummary {
                agent_id: p.agent.id.clone(),
                provider: p.agent.provider.clone(),
                model: p.agent.model.clone(),
                is_host: p.is_host,
                status: p.agent.status,
                message_count: session.messages_from(&p.agent.id).len(),
            })
            .collect();

        Ok(SessionSummary {
            session_id: session.id.clone(),
            name: session.name.clone(),
            state: session.state,
            created_at: session.created_at,
            message_count: session.messages().len(),
            participants: participant_summaries,
        })
    }

    pub async fn list_active_sessions(&self) -> Vec<SessionSummary> {
        let manager = self.session_manager.read().await;
        let mut summaries = Vec::new();

        for session in manager.active_sessions() {
            let participant_summaries: Vec<ParticipantSummary> = session
                .participants()
                .iter()
                .map(|p| ParticipantSummary {
                    agent_id: p.agent.id.clone(),
                    provider: p.agent.provider.clone(),
                    model: p.agent.model.clone(),
                    is_host: p.is_host,
                    status: p.agent.status,
                    message_count: session.messages_from(&p.agent.id).len(),
                })
                .collect();

            summaries.push(SessionSummary {
                session_id: session.id.clone(),
                name: session.name.clone(),
                state: session.state,
                created_at: session.created_at,
                message_count: session.messages().len(),
                participants: participant_summaries,
            });
        }

        summaries
    }

    fn build_context(&self, session: &CollabSession, new_message: &str) -> String {
        let mut context = String::new();

        context.push_str("=== Collaboration Session Context ===\n\n");
        context.push_str(&format!("Session: {}\n", session.name));
        context.push_str(&format!(
            "Participants: {}\n\n",
            session.participant_count()
        ));

        context.push_str("Recent conversation:\n");
        for msg in session.recent_messages(10).iter().rev() {
            if let MessageContent::Text(text) = &msg.content {
                context.push_str(&format!("[{}]: {}\n", msg.sender_id, text));
            }
        }

        context.push_str(&format!("\nNew message: {}\n", new_message));
        context
            .push_str("\nPlease respond to this conversation as a collaborative AI participant.");

        context
    }

    async fn get_agent_response(
        &self,
        provider: &Arc<dyn AIProvider>,
        model: &str,
        context: &str,
    ) -> Result<String> {
        let request = ChatRequest::new(vec![Message::user(context)]).with_model(model);

        let response = provider.chat(request).await?;
        Ok(response.content)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub name: String,
    pub state: SessionState,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
    pub participants: Vec<ParticipantSummary>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ParticipantSummary {
    pub agent_id: String,
    pub provider: String,
    pub model: String,
    pub is_host: bool,
    pub status: AgentStatus,
    pub message_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let orchestrator = CollabOrchestrator::new(10);
        assert!(orchestrator.providers.is_empty());
    }
}
