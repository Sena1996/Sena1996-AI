use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubMember {
    pub id: Uuid,
    pub role: String,
    pub name: String,
    pub joined_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubMessage {
    pub id: Uuid,
    pub from: String,
    pub from_id: Uuid,
    pub to: Option<String>,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub read: bool,
    pub broadcast: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct HubState {
    members: HashMap<Uuid, HubMember>,
    role_index: HashMap<String, Uuid>,
    messages: HashMap<Uuid, Vec<HubMessage>>,
    current_member: Option<Uuid>,
}

impl HubMessage {
    pub fn direct(from: &str, from_id: Uuid, to: &str, content: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            from: from.to_string(),
            from_id,
            to: Some(to.to_string()),
            content: content.to_string(),
            timestamp: Utc::now(),
            read: false,
            broadcast: false,
        }
    }

    pub fn broadcast(from: &str, from_id: Uuid, content: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            from: from.to_string(),
            from_id,
            to: None,
            content: content.to_string(),
            timestamp: Utc::now(),
            read: false,
            broadcast: true,
        }
    }
}

pub struct Hub {
    state: RwLock<HubState>,
    state_file: PathBuf,
}

impl Hub {
    pub fn new() -> Self {
        let state_file = Self::default_state_file();
        let state = Self::load_state(&state_file).unwrap_or_default();
        Self {
            state: RwLock::new(state),
            state_file,
        }
    }

    pub fn with_state_file(path: PathBuf) -> Self {
        let state = Self::load_state(&path).unwrap_or_default();
        Self {
            state: RwLock::new(state),
            state_file: path,
        }
    }

    fn default_state_file() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sena")
            .join("hub_state.json")
    }

    fn load_state(path: &PathBuf) -> Option<HubState> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn save_state(&self) {
        let state = self.state.read();
        if let Some(parent) = self.state_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(&*state) {
            let _ = std::fs::write(&self.state_file, content);
        }
    }

    pub fn join(&self, role: &str, name: Option<&str>) -> Uuid {
        let id = Uuid::new_v4();
        let display_name = name.unwrap_or(role).to_string();
        let now = Utc::now();

        let member = HubMember {
            id,
            role: role.to_string(),
            name: display_name,
            joined_at: now,
            last_seen: now,
        };

        {
            let mut state = self.state.write();
            state.members.insert(id, member);
            state.role_index.insert(role.to_string(), id);
            state.messages.insert(id, Vec::new());
            state.current_member = Some(id);
        }

        self.save_state();
        id
    }

    pub fn leave(&self) -> bool {
        let result = {
            let mut state = self.state.write();
            if let Some(id) = state.current_member {
                if let Some(member) = state.members.remove(&id) {
                    state.role_index.remove(&member.role);
                }
                state.current_member = None;
                true
            } else {
                false
            }
        };

        if result {
            self.save_state();
        }
        result
    }

    pub fn current_member_id(&self) -> Option<Uuid> {
        self.state.read().current_member
    }

    pub fn current_member(&self) -> Option<HubMember> {
        let state = self.state.read();
        let current_id = state.current_member?;
        state.members.get(&current_id).cloned()
    }

    pub fn get_member_by_role(&self, role: &str) -> Option<HubMember> {
        let state = self.state.read();
        let id = state.role_index.get(role).cloned()?;
        state.members.get(&id).cloned()
    }

    pub fn list_members(&self) -> Vec<HubMember> {
        self.state.read().members.values().cloned().collect()
    }

    pub fn tell(&self, target_role: &str, message: &str) -> Result<Uuid, String> {
        let from = self.current_member()
            .ok_or_else(|| "Not joined to hub. Use 'sena hub join <role>' first.".to_string())?;

        let target = self.get_member_by_role(target_role)
            .ok_or_else(|| format!("No member with role '{}' found in hub.", target_role))?;

        let hub_msg = HubMessage::direct(&from.role, from.id, target_role, message);
        let msg_id = hub_msg.id;

        {
            let mut state = self.state.write();
            state.messages
                .entry(target.id)
                .or_default()
                .push(hub_msg);
        }

        self.save_state();
        Ok(msg_id)
    }

    pub fn broadcast(&self, message: &str) -> Result<usize, String> {
        let from = self.current_member()
            .ok_or_else(|| "Not joined to hub. Use 'sena hub join <role>' first.".to_string())?;

        let member_ids: Vec<Uuid> = {
            let state = self.state.read();
            state.members
                .keys()
                .filter(|&id| *id != from.id)
                .cloned()
                .collect()
        };

        let count = member_ids.len();

        {
            let mut state = self.state.write();
            for member_id in member_ids {
                let hub_msg = HubMessage::broadcast(&from.role, from.id, message);
                state.messages
                    .entry(member_id)
                    .or_default()
                    .push(hub_msg);
            }
        }

        self.save_state();
        Ok(count)
    }

    pub fn inbox(&self, include_read: bool) -> Vec<HubMessage> {
        let state = self.state.read();
        let current_id = match state.current_member {
            Some(id) => id,
            None => return Vec::new(),
        };

        let my_messages = match state.messages.get(&current_id) {
            Some(msgs) => msgs,
            None => return Vec::new(),
        };

        if include_read {
            my_messages.clone()
        } else {
            my_messages.iter().filter(|m| !m.read).cloned().collect()
        }
    }

    pub fn mark_read(&self, message_id: Uuid) -> bool {
        let result = {
            let mut state = self.state.write();
            let current_id = match state.current_member {
                Some(id) => id,
                None => return false,
            };

            if let Some(my_messages) = state.messages.get_mut(&current_id) {
                for msg in my_messages.iter_mut() {
                    if msg.id == message_id {
                        msg.read = true;
                        return true;
                    }
                }
            }
            false
        };

        if result {
            self.save_state();
        }
        result
    }

    pub fn mark_all_read(&self) -> usize {
        let count = {
            let mut state = self.state.write();
            let current_id = match state.current_member {
                Some(id) => id,
                None => return 0,
            };

            let mut count = 0;
            if let Some(my_messages) = state.messages.get_mut(&current_id) {
                for msg in my_messages.iter_mut() {
                    if !msg.read {
                        msg.read = true;
                        count += 1;
                    }
                }
            }
            count
        };

        if count > 0 {
            self.save_state();
        }
        count
    }

    pub fn unread_count(&self) -> usize {
        let state = self.state.read();
        let current_id = match state.current_member {
            Some(id) => id,
            None => return 0,
        };

        state.messages.get(&current_id)
            .map(|msgs| msgs.iter().filter(|m| !m.read).count())
            .unwrap_or(0)
    }

    pub fn update_last_seen(&self) {
        {
            let mut state = self.state.write();
            if let Some(id) = state.current_member {
                if let Some(member) = state.members.get_mut(&id) {
                    member.last_seen = Utc::now();
                }
            }
        }
        self.save_state();
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_hub() -> Hub {
        let temp_file = PathBuf::from("/tmp/sena_hub_test.json");
        let _ = std::fs::remove_file(&temp_file);
        Hub::with_state_file(temp_file)
    }

    #[test]
    fn test_join_and_leave() {
        let hub = test_hub();

        let _id = hub.join("AndroidDev", Some("Alice"));
        assert!(hub.current_member_id().is_some());
        assert_eq!(hub.current_member().unwrap().role, "AndroidDev");

        assert!(hub.leave());
        assert!(hub.current_member_id().is_none());
    }

    #[test]
    fn test_tell() {
        let hub = test_hub();

        hub.join("AndroidDev", Some("Alice"));
        let alice_id = hub.current_member_id().unwrap();
        hub.leave();

        hub.join("BackendDev", Some("Bob"));

        {
            let mut state = hub.state.write();
            state.members.insert(alice_id, HubMember {
                id: alice_id,
                role: "AndroidDev".to_string(),
                name: "Alice".to_string(),
                joined_at: Utc::now(),
                last_seen: Utc::now(),
            });
            state.role_index.insert("AndroidDev".to_string(), alice_id);
            state.messages.insert(alice_id, Vec::new());
        }

        let result = hub.tell("AndroidDev", "Hello Alice!");
        assert!(result.is_ok());

        {
            let mut state = hub.state.write();
            state.current_member = Some(alice_id);
        }
        let inbox = hub.inbox(false);
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].content, "Hello Alice!");
    }

    #[test]
    fn test_broadcast() {
        let hub = test_hub();

        hub.join("AndroidDev", None);
        hub.leave();
        hub.join("WebDev", None);
        hub.leave();
        hub.join("BackendDev", None);

        {
            let mut state = hub.state.write();
            let id = Uuid::new_v4();
            state.members.insert(id, HubMember {
                id,
                role: "OtherDev".to_string(),
                name: "OtherDev".to_string(),
                joined_at: Utc::now(),
                last_seen: Utc::now(),
            });
        }

        let result = hub.broadcast("API updated!");
        assert!(result.is_ok());
    }
}
