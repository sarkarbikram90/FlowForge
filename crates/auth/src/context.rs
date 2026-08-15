use crate::rbac::Role;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub project_id: Uuid,
    pub email: String,
    pub role: Role,
}

impl Default for AuthContext {
    fn default() -> Self {
        Self {
            user_id: Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            organization_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            project_id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
            email: "admin@flowforge.internal".to_string(),
            role: Role::PlatformAdmin,
        }
    }
}

impl AuthContext {
    pub fn new(
        user_id: Uuid,
        organization_id: Uuid,
        project_id: Uuid,
        email: String,
        role: Role,
    ) -> Self {
        Self {
            user_id,
            organization_id,
            project_id,
            email,
            role,
        }
    }

    pub fn require_permission(&self, perm: &str) -> flowforge_common::Result<()> {
        if self.role.has_permission(perm) {
            Ok(())
        } else {
            Err(flowforge_common::FlowForgeError::Forbidden(format!(
                "Role '{:?}' lacks required permission '{}'",
                self.role, perm
            )))
        }
    }
}
