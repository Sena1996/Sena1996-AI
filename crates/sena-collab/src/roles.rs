use sena_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantRole {
    Owner,
    Driver,
    Navigator,
    Observer,
}

impl ParticipantRole {
    pub fn default_permissions(&self) -> PermissionSet {
        match self {
            Self::Owner => PermissionSet::owner(),
            Self::Driver => PermissionSet::driver(),
            Self::Navigator => PermissionSet::navigator(),
            Self::Observer => PermissionSet::observer(),
        }
    }

    pub fn can_promote_to(&self, target: ParticipantRole) -> bool {
        match (self, target) {
            (Self::Owner, _) => true,
            (Self::Driver, Self::Navigator | Self::Observer) => false,
            (Self::Navigator, Self::Observer) => false,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    pub can_send_messages: bool,
    pub can_execute_commands: bool,
    pub can_modify_files: bool,
    pub can_invite_others: bool,
    pub can_change_provider: bool,
    pub can_view_history: bool,
    pub can_export_session: bool,
    pub can_change_roles: bool,
    pub can_terminate_session: bool,
}

impl PermissionSet {
    pub fn owner() -> Self {
        Self {
            can_send_messages: true,
            can_execute_commands: true,
            can_modify_files: true,
            can_invite_others: true,
            can_change_provider: true,
            can_view_history: true,
            can_export_session: true,
            can_change_roles: true,
            can_terminate_session: true,
        }
    }

    pub fn driver() -> Self {
        Self {
            can_send_messages: true,
            can_execute_commands: true,
            can_modify_files: true,
            can_invite_others: false,
            can_change_provider: false,
            can_view_history: true,
            can_export_session: false,
            can_change_roles: false,
            can_terminate_session: false,
        }
    }

    pub fn navigator() -> Self {
        Self {
            can_send_messages: true,
            can_execute_commands: false,
            can_modify_files: false,
            can_invite_others: false,
            can_change_provider: false,
            can_view_history: true,
            can_export_session: false,
            can_change_roles: false,
            can_terminate_session: false,
        }
    }

    pub fn observer() -> Self {
        Self {
            can_send_messages: false,
            can_execute_commands: false,
            can_modify_files: false,
            can_invite_others: false,
            can_change_provider: false,
            can_view_history: true,
            can_export_session: false,
            can_change_roles: false,
            can_terminate_session: false,
        }
    }

    pub fn has_permission(&self, action: &Action) -> bool {
        match action {
            Action::SendMessage => self.can_send_messages,
            Action::ExecuteCommand => self.can_execute_commands,
            Action::ModifyFile => self.can_modify_files,
            Action::InviteParticipant => self.can_invite_others,
            Action::ChangeProvider => self.can_change_provider,
            Action::ViewHistory => self.can_view_history,
            Action::ExportSession => self.can_export_session,
            Action::ChangeRoles => self.can_change_roles,
            Action::TerminateSession => self.can_terminate_session,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    SendMessage,
    ExecuteCommand,
    ModifyFile,
    InviteParticipant,
    ChangeProvider,
    ViewHistory,
    ExportSession,
    ChangeRoles,
    TerminateSession,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: uuid::Uuid,
    pub name: String,
    pub role: ParticipantRole,
    pub permissions: PermissionSet,
    pub joined_at: u64,
    pub last_active: u64,
    pub is_connected: bool,
}

impl Participant {
    pub fn new(name: impl Into<String>, role: ParticipantRole) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4(),
            name: name.into(),
            role,
            permissions: role.default_permissions(),
            joined_at: now,
            last_active: now,
            is_connected: true,
        }
    }

    pub fn with_custom_permissions(mut self, permissions: PermissionSet) -> Self {
        self.permissions = permissions;
        self
    }

    pub fn can(&self, action: &Action) -> bool {
        self.permissions.has_permission(action)
    }

    pub fn touch(&mut self) {
        self.last_active = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

#[derive(Clone)]
pub struct RoleEnforcer {
    participants: HashMap<uuid::Uuid, Participant>,
    owner_id: Option<uuid::Uuid>,
    driver_id: Option<uuid::Uuid>,
    max_participants: usize,
}

impl RoleEnforcer {
    pub fn new() -> Self {
        Self {
            participants: HashMap::new(),
            owner_id: None,
            driver_id: None,
            max_participants: 10,
        }
    }

    pub fn with_max_participants(mut self, max: usize) -> Self {
        self.max_participants = max;
        self
    }

    pub fn add_participant(&mut self, name: impl Into<String>, role: ParticipantRole) -> Result<uuid::Uuid> {
        if self.participants.len() >= self.max_participants {
            return Err(Error::validation("maximum participants reached"));
        }

        if role == ParticipantRole::Owner && self.owner_id.is_some() {
            return Err(Error::validation("session already has an owner"));
        }

        let participant = Participant::new(name, role);
        let id = participant.id;

        if role == ParticipantRole::Owner {
            self.owner_id = Some(id);
            if self.driver_id.is_none() {
                self.driver_id = Some(id);
            }
        } else if role == ParticipantRole::Driver && self.driver_id.is_none() {
            self.driver_id = Some(id);
        }

        self.participants.insert(id, participant);
        Ok(id)
    }

    pub fn remove_participant(&mut self, id: uuid::Uuid) -> Result<()> {
        if Some(id) == self.owner_id {
            return Err(Error::validation("cannot remove session owner"));
        }

        if Some(id) == self.driver_id {
            self.driver_id = self.owner_id;
        }

        self.participants.remove(&id);
        Ok(())
    }

    pub fn check_permission(&self, participant_id: uuid::Uuid, action: &Action) -> Result<()> {
        let participant = self.participants.get(&participant_id)
            .ok_or_else(|| Error::not_found("participant not found"))?;

        if !participant.can(action) {
            return Err(Error::security(format!(
                "participant {:?} does not have permission for {:?}",
                participant.role, action
            )));
        }

        Ok(())
    }

    pub fn transfer_driver(&mut self, from_id: uuid::Uuid, to_id: uuid::Uuid) -> Result<()> {
        if Some(from_id) != self.driver_id && Some(from_id) != self.owner_id {
            return Err(Error::security("only driver or owner can transfer driver role"));
        }

        let to_participant = self.participants.get(&to_id)
            .ok_or_else(|| Error::not_found("target participant not found"))?;

        if to_participant.role == ParticipantRole::Observer {
            return Err(Error::validation("cannot transfer driver role to observer"));
        }

        if let Some(old_driver) = self.driver_id {
            if old_driver != to_id {
                if let Some(participant) = self.participants.get_mut(&old_driver) {
                    if participant.role == ParticipantRole::Driver {
                        participant.role = ParticipantRole::Navigator;
                        participant.permissions = ParticipantRole::Navigator.default_permissions();
                    }
                }
            }
        }

        if let Some(participant) = self.participants.get_mut(&to_id) {
            participant.role = ParticipantRole::Driver;
            participant.permissions = ParticipantRole::Driver.default_permissions();
        }

        self.driver_id = Some(to_id);
        Ok(())
    }

    pub fn change_role(&mut self, actor_id: uuid::Uuid, target_id: uuid::Uuid, new_role: ParticipantRole) -> Result<()> {
        self.check_permission(actor_id, &Action::ChangeRoles)?;

        if target_id == self.owner_id.unwrap_or_default() && new_role != ParticipantRole::Owner {
            return Err(Error::validation("cannot demote session owner"));
        }

        if new_role == ParticipantRole::Owner {
            return Err(Error::validation("cannot promote to owner"));
        }

        let participant = self.participants.get_mut(&target_id)
            .ok_or_else(|| Error::not_found("participant not found"))?;

        participant.role = new_role;
        participant.permissions = new_role.default_permissions();

        if new_role == ParticipantRole::Driver {
            if let Some(old_driver) = self.driver_id {
                if old_driver != target_id {
                    if let Some(old) = self.participants.get_mut(&old_driver) {
                        if old.role == ParticipantRole::Driver {
                            old.role = ParticipantRole::Navigator;
                            old.permissions = ParticipantRole::Navigator.default_permissions();
                        }
                    }
                }
            }
            self.driver_id = Some(target_id);
        }

        Ok(())
    }

    pub fn get_participant(&self, id: uuid::Uuid) -> Option<&Participant> {
        self.participants.get(&id)
    }

    pub fn get_participant_mut(&mut self, id: uuid::Uuid) -> Option<&mut Participant> {
        self.participants.get_mut(&id)
    }

    pub fn owner(&self) -> Option<&Participant> {
        self.owner_id.and_then(|id| self.participants.get(&id))
    }

    pub fn driver(&self) -> Option<&Participant> {
        self.driver_id.and_then(|id| self.participants.get(&id))
    }

    pub fn participants(&self) -> impl Iterator<Item = &Participant> {
        self.participants.values()
    }

    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }
}

impl Default for RoleEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set() {
        let owner = PermissionSet::owner();
        assert!(owner.has_permission(&Action::TerminateSession));

        let observer = PermissionSet::observer();
        assert!(!observer.has_permission(&Action::SendMessage));
        assert!(observer.has_permission(&Action::ViewHistory));
    }

    #[test]
    fn test_role_enforcer() {
        let mut enforcer = RoleEnforcer::new();

        let owner_id = enforcer.add_participant("Alice", ParticipantRole::Owner).unwrap();
        let nav_id = enforcer.add_participant("Bob", ParticipantRole::Navigator).unwrap();

        assert!(enforcer.check_permission(owner_id, &Action::TerminateSession).is_ok());
        assert!(enforcer.check_permission(nav_id, &Action::TerminateSession).is_err());
    }

    #[test]
    fn test_driver_transfer() {
        let mut enforcer = RoleEnforcer::new();

        let owner_id = enforcer.add_participant("Alice", ParticipantRole::Owner).unwrap();
        let nav_id = enforcer.add_participant("Bob", ParticipantRole::Navigator).unwrap();

        enforcer.transfer_driver(owner_id, nav_id).unwrap();
        assert_eq!(enforcer.driver_id, Some(nav_id));
    }

    #[test]
    fn test_cannot_have_two_owners() {
        let mut enforcer = RoleEnforcer::new();

        enforcer.add_participant("Alice", ParticipantRole::Owner).unwrap();
        let result = enforcer.add_participant("Bob", ParticipantRole::Owner);

        assert!(result.is_err());
    }
}
