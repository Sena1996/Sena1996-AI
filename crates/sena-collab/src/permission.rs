use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    ReadMessages,
    SendMessages,
    CreateSession,
    JoinSession,
    InviteAgent,
    ModifyContext,
    ExecuteTools,
    ShareFiles,
    AccessHistory,
    ModerateSession,
}

impl Permission {
    pub fn all() -> HashSet<Permission> {
        HashSet::from([
            Permission::ReadMessages,
            Permission::SendMessages,
            Permission::CreateSession,
            Permission::JoinSession,
            Permission::InviteAgent,
            Permission::ModifyContext,
            Permission::ExecuteTools,
            Permission::ShareFiles,
            Permission::AccessHistory,
            Permission::ModerateSession,
        ])
    }

    pub fn basic_participant() -> HashSet<Permission> {
        HashSet::from([
            Permission::ReadMessages,
            Permission::SendMessages,
            Permission::JoinSession,
        ])
    }

    pub fn standard_agent() -> HashSet<Permission> {
        HashSet::from([
            Permission::ReadMessages,
            Permission::SendMessages,
            Permission::JoinSession,
            Permission::ModifyContext,
            Permission::ExecuteTools,
            Permission::AccessHistory,
        ])
    }

    pub fn session_host() -> HashSet<Permission> {
        Permission::all()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: HashSet<Permission>,
    granted_by: Option<String>,
    granted_at: chrono::DateTime<chrono::Utc>,
}

impl PermissionSet {
    pub fn new(permissions: HashSet<Permission>) -> Self {
        Self {
            permissions,
            granted_by: None,
            granted_at: chrono::Utc::now(),
        }
    }

    pub fn with_granter(mut self, granter_id: &str) -> Self {
        self.granted_by = Some(granter_id.to_string());
        self
    }

    pub fn has(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    pub fn has_all(&self, required: &[Permission]) -> bool {
        required.iter().all(|p| self.permissions.contains(p))
    }

    pub fn has_any(&self, required: &[Permission]) -> bool {
        required.iter().any(|p| self.permissions.contains(p))
    }

    pub fn grant(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn revoke(&mut self, permission: Permission) {
        self.permissions.remove(&permission);
    }

    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new(Permission::basic_participant())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub requester_id: String,
    pub requested_permissions: Vec<Permission>,
    pub reason: Option<String>,
    pub requested_at: chrono::DateTime<chrono::Utc>,
}

impl PermissionRequest {
    pub fn new(requester_id: &str, permissions: Vec<Permission>) -> Self {
        Self {
            requester_id: requester_id.to_string(),
            requested_permissions: permissions,
            reason: None,
            requested_at: chrono::Utc::now(),
        }
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionApproval {
    pub request: PermissionRequest,
    pub status: ApprovalStatus,
    pub decided_by: Option<String>,
    pub decided_at: Option<chrono::DateTime<chrono::Utc>>,
    pub denial_reason: Option<String>,
}

impl PermissionApproval {
    pub fn pending(request: PermissionRequest) -> Self {
        Self {
            request,
            status: ApprovalStatus::Pending,
            decided_by: None,
            decided_at: None,
            denial_reason: None,
        }
    }

    pub fn approve(&mut self, approver_id: &str) {
        self.status = ApprovalStatus::Approved;
        self.decided_by = Some(approver_id.to_string());
        self.decided_at = Some(chrono::Utc::now());
    }

    pub fn deny(&mut self, approver_id: &str, reason: Option<String>) {
        self.status = ApprovalStatus::Denied;
        self.decided_by = Some(approver_id.to_string());
        self.decided_at = Some(chrono::Utc::now());
        self.denial_reason = reason;
    }

    pub fn is_pending(&self) -> bool {
        self.status == ApprovalStatus::Pending
    }

    pub fn is_approved(&self) -> bool {
        self.status == ApprovalStatus::Approved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_sets() {
        let basic = Permission::basic_participant();
        assert!(basic.contains(&Permission::ReadMessages));
        assert!(!basic.contains(&Permission::ModerateSession));

        let all = Permission::all();
        assert!(all.contains(&Permission::ModerateSession));
    }

    #[test]
    fn test_permission_set_operations() {
        let mut set = PermissionSet::default();
        assert!(set.has(Permission::ReadMessages));
        assert!(!set.has(Permission::ModerateSession));

        set.grant(Permission::ModerateSession);
        assert!(set.has(Permission::ModerateSession));

        set.revoke(Permission::ModerateSession);
        assert!(!set.has(Permission::ModerateSession));
    }

    #[test]
    fn test_permission_approval() {
        let request = PermissionRequest::new("agent-1", vec![Permission::ExecuteTools]);
        let mut approval = PermissionApproval::pending(request);

        assert!(approval.is_pending());
        assert!(!approval.is_approved());

        approval.approve("user-1");
        assert!(approval.is_approved());
        assert_eq!(approval.decided_by, Some("user-1".to_string()));
    }
}
