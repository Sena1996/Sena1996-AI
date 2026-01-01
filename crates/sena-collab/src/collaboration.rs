use sena_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

use crate::roles::{Action, ParticipantRole, RoleEnforcer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollaborationMode {
    FreeForm,
    DriverNavigator,
    TurnBased,
    Moderated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    pub id: Uuid,
    pub mode: CollaborationMode,
    pub state: SessionState,
    pub turn_manager: Option<TurnManager>,
    pub driver_navigator: Option<DriverNavigatorState>,
    pub message_queue: VecDeque<QueuedMessage>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    Active,
    Paused,
    AwaitingTurn,
    Voting,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: u64,
    pub requires_driver: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Command,
    Suggestion,
    Comment,
    Question,
    Vote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverNavigatorState {
    pub driver_id: Uuid,
    pub navigator_ids: Vec<Uuid>,
    pub swap_interval_secs: Option<u64>,
    pub last_swap_at: u64,
    pub pending_suggestions: Vec<NavigatorSuggestion>,
    pub auto_rotate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigatorSuggestion {
    pub id: Uuid,
    pub navigator_id: Uuid,
    pub content: String,
    pub suggestion_type: SuggestionType,
    pub status: SuggestionStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionType {
    CodeChange,
    Direction,
    Review,
    Question,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionStatus {
    Pending,
    Accepted,
    Rejected,
    Implemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnManager {
    pub turn_order: Vec<Uuid>,
    pub current_turn_index: usize,
    pub turn_duration_secs: Option<u64>,
    pub turn_started_at: u64,
    pub skip_inactive: bool,
    pub allow_pass: bool,
}

impl CollaborationSession {
    pub fn new(mode: CollaborationMode) -> Self {
        let now = Self::now();

        Self {
            id: Uuid::new_v4(),
            mode,
            state: SessionState::Active,
            turn_manager: if mode == CollaborationMode::TurnBased {
                Some(TurnManager::new())
            } else {
                None
            },
            driver_navigator: if mode == CollaborationMode::DriverNavigator {
                Some(DriverNavigatorState::new())
            } else {
                None
            },
            message_queue: VecDeque::new(),
            created_at: now,
        }
    }

    pub fn with_driver(mut self, driver_id: Uuid) -> Self {
        if let Some(ref mut dn) = self.driver_navigator {
            dn.driver_id = driver_id;
            dn.last_swap_at = Self::now();
        }
        self
    }

    pub fn with_turn_order(mut self, order: Vec<Uuid>) -> Self {
        if let Some(ref mut tm) = self.turn_manager {
            tm.turn_order = order;
        }
        self
    }

    pub fn can_act(&self, participant_id: Uuid, action: &Action, enforcer: &RoleEnforcer) -> Result<bool> {
        enforcer.check_permission(participant_id, action)?;

        match self.mode {
            CollaborationMode::FreeForm => Ok(true),

            CollaborationMode::DriverNavigator => {
                if let Some(ref dn) = self.driver_navigator {
                    match action {
                        Action::ExecuteCommand | Action::ModifyFile => {
                            Ok(participant_id == dn.driver_id)
                        }
                        Action::SendMessage => Ok(true),
                        _ => Ok(participant_id == dn.driver_id)
                    }
                } else {
                    Ok(true)
                }
            }

            CollaborationMode::TurnBased => {
                if let Some(ref tm) = self.turn_manager {
                    Ok(tm.is_current_turn(participant_id))
                } else {
                    Ok(true)
                }
            }

            CollaborationMode::Moderated => {
                let participant = enforcer.get_participant(participant_id)
                    .ok_or_else(|| Error::not_found("participant not found"))?;
                Ok(participant.role == ParticipantRole::Owner || participant.role == ParticipantRole::Driver)
            }
        }
    }

    pub fn submit_suggestion(&mut self, navigator_id: Uuid, content: String, suggestion_type: SuggestionType) -> Result<Uuid> {
        if self.mode != CollaborationMode::DriverNavigator {
            return Err(Error::validation("suggestions only available in Driver-Navigator mode"));
        }

        let dn = self.driver_navigator.as_mut()
            .ok_or_else(|| Error::validation("driver-navigator state not initialized"))?;

        if navigator_id == dn.driver_id {
            return Err(Error::validation("driver cannot submit suggestions"));
        }

        let suggestion = NavigatorSuggestion {
            id: Uuid::new_v4(),
            navigator_id,
            content,
            suggestion_type,
            status: SuggestionStatus::Pending,
            created_at: Self::now(),
        };

        let id = suggestion.id;
        dn.pending_suggestions.push(suggestion);
        Ok(id)
    }

    pub fn respond_to_suggestion(&mut self, driver_id: Uuid, suggestion_id: Uuid, accept: bool) -> Result<()> {
        let dn = self.driver_navigator.as_mut()
            .ok_or_else(|| Error::validation("not in driver-navigator mode"))?;

        if driver_id != dn.driver_id {
            return Err(Error::security("only driver can respond to suggestions"));
        }

        if let Some(suggestion) = dn.pending_suggestions.iter_mut().find(|s| s.id == suggestion_id) {
            suggestion.status = if accept {
                SuggestionStatus::Accepted
            } else {
                SuggestionStatus::Rejected
            };
            Ok(())
        } else {
            Err(Error::not_found("suggestion not found"))
        }
    }

    pub fn swap_driver(&mut self, new_driver_id: Uuid, enforcer: &mut RoleEnforcer) -> Result<Uuid> {
        let dn = self.driver_navigator.as_mut()
            .ok_or_else(|| Error::validation("not in driver-navigator mode"))?;

        let old_driver_id = dn.driver_id;

        if !dn.navigator_ids.contains(&new_driver_id) && new_driver_id != old_driver_id {
            return Err(Error::validation("new driver must be a current navigator"));
        }

        enforcer.transfer_driver(old_driver_id, new_driver_id)?;

        dn.navigator_ids.retain(|id| *id != new_driver_id);
        if old_driver_id != new_driver_id {
            dn.navigator_ids.push(old_driver_id);
        }
        dn.driver_id = new_driver_id;
        dn.last_swap_at = Self::now();

        Ok(old_driver_id)
    }

    pub fn check_auto_swap(&mut self, enforcer: &mut RoleEnforcer) -> Result<Option<Uuid>> {
        let dn = self.driver_navigator.as_ref()
            .ok_or_else(|| Error::validation("not in driver-navigator mode"))?;

        if !dn.auto_rotate {
            return Ok(None);
        }

        if let Some(interval) = dn.swap_interval_secs {
            let now = Self::now();
            if now - dn.last_swap_at >= interval && !dn.navigator_ids.is_empty() {
                let next_driver = dn.navigator_ids[0];
                return self.swap_driver(next_driver, enforcer).map(Some);
            }
        }

        Ok(None)
    }

    pub fn advance_turn(&mut self) -> Result<Uuid> {
        let tm = self.turn_manager.as_mut()
            .ok_or_else(|| Error::validation("not in turn-based mode"))?;

        tm.advance();
        tm.current_participant()
            .ok_or_else(|| Error::validation("no participants in turn order"))
    }

    pub fn pass_turn(&mut self, participant_id: Uuid) -> Result<Uuid> {
        let tm = self.turn_manager.as_mut()
            .ok_or_else(|| Error::validation("not in turn-based mode"))?;

        if !tm.allow_pass {
            return Err(Error::validation("passing is not allowed in this session"));
        }

        if !tm.is_current_turn(participant_id) {
            return Err(Error::validation("not your turn"));
        }

        self.advance_turn()
    }

    pub fn current_turn(&self) -> Option<Uuid> {
        self.turn_manager.as_ref().and_then(|tm| tm.current_participant())
    }

    pub fn queue_message(&mut self, sender_id: Uuid, content: String, message_type: MessageType) {
        let requires_driver = matches!(message_type, MessageType::Command);

        self.message_queue.push_back(QueuedMessage {
            id: Uuid::new_v4(),
            sender_id,
            content,
            message_type,
            timestamp: Self::now(),
            requires_driver,
        });
    }

    pub fn process_queue(&mut self, enforcer: &RoleEnforcer) -> Vec<QueuedMessage> {
        let mut processed = Vec::new();

        while let Some(msg) = self.message_queue.front() {
            let can_process = if msg.requires_driver {
                if let Some(ref dn) = self.driver_navigator {
                    msg.sender_id == dn.driver_id
                } else {
                    true
                }
            } else {
                self.can_act(msg.sender_id, &Action::SendMessage, enforcer).unwrap_or(false)
            };

            if can_process {
                if let Some(msg) = self.message_queue.pop_front() {
                    processed.push(msg);
                }
            } else {
                break;
            }
        }

        processed
    }

    pub fn pause(&mut self) {
        self.state = SessionState::Paused;
    }

    pub fn resume(&mut self) {
        self.state = SessionState::Active;
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

impl DriverNavigatorState {
    pub fn new() -> Self {
        Self {
            driver_id: Uuid::nil(),
            navigator_ids: Vec::new(),
            swap_interval_secs: None,
            last_swap_at: 0,
            pending_suggestions: Vec::new(),
            auto_rotate: false,
        }
    }

    pub fn with_auto_rotate(mut self, interval_secs: u64) -> Self {
        self.auto_rotate = true;
        self.swap_interval_secs = Some(interval_secs);
        self
    }

    pub fn add_navigator(&mut self, id: Uuid) {
        if !self.navigator_ids.contains(&id) && id != self.driver_id {
            self.navigator_ids.push(id);
        }
    }

    pub fn remove_navigator(&mut self, id: Uuid) {
        self.navigator_ids.retain(|n| *n != id);
    }

    pub fn pending_suggestion_count(&self) -> usize {
        self.pending_suggestions.iter()
            .filter(|s| s.status == SuggestionStatus::Pending)
            .count()
    }
}

impl Default for DriverNavigatorState {
    fn default() -> Self {
        Self::new()
    }
}

impl TurnManager {
    pub fn new() -> Self {
        Self {
            turn_order: Vec::new(),
            current_turn_index: 0,
            turn_duration_secs: None,
            turn_started_at: 0,
            skip_inactive: true,
            allow_pass: true,
        }
    }

    pub fn with_duration(mut self, secs: u64) -> Self {
        self.turn_duration_secs = Some(secs);
        self
    }

    pub fn with_order(mut self, order: Vec<Uuid>) -> Self {
        self.turn_order = order;
        self
    }

    pub fn add_participant(&mut self, id: Uuid) {
        if !self.turn_order.contains(&id) {
            self.turn_order.push(id);
        }
    }

    pub fn remove_participant(&mut self, id: Uuid) {
        if let Some(pos) = self.turn_order.iter().position(|p| *p == id) {
            self.turn_order.remove(pos);
            if self.current_turn_index >= self.turn_order.len() && !self.turn_order.is_empty() {
                self.current_turn_index = 0;
            }
        }
    }

    pub fn current_participant(&self) -> Option<Uuid> {
        self.turn_order.get(self.current_turn_index).copied()
    }

    pub fn is_current_turn(&self, participant_id: Uuid) -> bool {
        self.current_participant() == Some(participant_id)
    }

    pub fn advance(&mut self) {
        if !self.turn_order.is_empty() {
            self.current_turn_index = (self.current_turn_index + 1) % self.turn_order.len();
            self.turn_started_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
        }
    }

    pub fn is_turn_expired(&self) -> bool {
        if let Some(duration) = self.turn_duration_secs {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            now - self.turn_started_at >= duration
        } else {
            false
        }
    }

    pub fn remaining_time(&self) -> Option<u64> {
        self.turn_duration_secs.map(|duration| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let elapsed = now - self.turn_started_at;
            duration.saturating_sub(elapsed)
        })
    }
}

impl Default for TurnManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_driver_navigator_mode() {
        let driver_id = Uuid::new_v4();
        let navigator_id = Uuid::new_v4();

        let mut session = CollaborationSession::new(CollaborationMode::DriverNavigator)
            .with_driver(driver_id);

        if let Some(ref mut dn) = session.driver_navigator {
            dn.add_navigator(navigator_id);
        }

        let suggestion_id = session.submit_suggestion(
            navigator_id,
            "Consider using a HashMap here".to_string(),
            SuggestionType::CodeChange,
        ).unwrap();

        assert!(session.driver_navigator.as_ref().unwrap().pending_suggestion_count() == 1);

        session.respond_to_suggestion(driver_id, suggestion_id, true).unwrap();

        let pending = session.driver_navigator.as_ref().unwrap().pending_suggestion_count();
        assert_eq!(pending, 0);
    }

    #[test]
    fn test_turn_based_mode() {
        let p1 = Uuid::new_v4();
        let p2 = Uuid::new_v4();
        let p3 = Uuid::new_v4();

        let mut session = CollaborationSession::new(CollaborationMode::TurnBased)
            .with_turn_order(vec![p1, p2, p3]);

        assert_eq!(session.current_turn(), Some(p1));

        session.advance_turn().unwrap();
        assert_eq!(session.current_turn(), Some(p2));

        session.advance_turn().unwrap();
        assert_eq!(session.current_turn(), Some(p3));

        session.advance_turn().unwrap();
        assert_eq!(session.current_turn(), Some(p1));
    }

    #[test]
    fn test_turn_manager_participant_removal() {
        let mut tm = TurnManager::new()
            .with_order(vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]);

        let to_remove = tm.turn_order[1];
        tm.current_turn_index = 2;

        tm.remove_participant(to_remove);

        assert_eq!(tm.turn_order.len(), 2);
        assert!(tm.current_turn_index < tm.turn_order.len());
    }

    #[test]
    fn test_message_queue() {
        let mut session = CollaborationSession::new(CollaborationMode::FreeForm);
        let sender = Uuid::new_v4();

        session.queue_message(sender, "Hello".to_string(), MessageType::Comment);
        session.queue_message(sender, "World".to_string(), MessageType::Comment);

        assert_eq!(session.message_queue.len(), 2);
    }
}
