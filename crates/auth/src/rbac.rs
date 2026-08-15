use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    PlatformAdmin,
    OrgAdmin,
    ProjectAdmin,
    WorkflowEditor,
    WorkflowOperator,
    Viewer,
    Auditor,
}

impl FromStr for Role {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "platformadmin" | "platform_admin" => Role::PlatformAdmin,
            "orgadmin" | "org_admin" => Role::OrgAdmin,
            "projectadmin" | "project_admin" => Role::ProjectAdmin,
            "workfloweditor" | "workflow_editor" | "editor" => Role::WorkflowEditor,
            "workflowoperator" | "workflow_operator" | "operator" => Role::WorkflowOperator,
            "auditor" => Role::Auditor,
            _ => Role::Viewer,
        })
    }
}

impl Role {
    pub fn parse(s: &str) -> Self {
        Role::from_str(s).unwrap_or(Role::Viewer)
    }

    pub fn permissions(&self) -> HashSet<&'static str> {
        let mut perms = HashSet::new();

        // Base viewer permissions
        perms.insert("workflow:read");
        perms.insert("run:read");
        perms.insert("worker:read");
        perms.insert("queue:read");

        match self {
            Role::Viewer => {}
            Role::Auditor => {
                perms.insert("audit:read");
                perms.insert("metrics:read");
            }
            Role::WorkflowOperator => {
                perms.insert("workflow:execute");
                perms.insert("workflow:cancel");
                perms.insert("run:retry");
                perms.insert("worker:drain");
                perms.insert("audit:read");
            }
            Role::WorkflowEditor => {
                perms.insert("workflow:create");
                perms.insert("workflow:update");
                perms.insert("workflow:delete");
                perms.insert("workflow:execute");
                perms.insert("workflow:cancel");
                perms.insert("run:retry");
                perms.insert("worker:drain");
                perms.insert("secret:read");
                perms.insert("audit:read");
            }
            Role::ProjectAdmin => {
                perms.insert("workflow:create");
                perms.insert("workflow:update");
                perms.insert("workflow:delete");
                perms.insert("workflow:execute");
                perms.insert("workflow:cancel");
                perms.insert("run:retry");
                perms.insert("worker:admin");
                perms.insert("worker:drain");
                perms.insert("secret:read");
                perms.insert("secret:write");
                perms.insert("project:admin");
                perms.insert("audit:read");
            }
            Role::OrgAdmin => {
                perms.insert("workflow:create");
                perms.insert("workflow:update");
                perms.insert("workflow:delete");
                perms.insert("workflow:execute");
                perms.insert("workflow:cancel");
                perms.insert("run:retry");
                perms.insert("worker:admin");
                perms.insert("worker:drain");
                perms.insert("secret:read");
                perms.insert("secret:write");
                perms.insert("project:admin");
                perms.insert("organization:admin");
                perms.insert("audit:read");
            }
            Role::PlatformAdmin => {
                // All permissions
                perms.insert("workflow:create");
                perms.insert("workflow:update");
                perms.insert("workflow:delete");
                perms.insert("workflow:execute");
                perms.insert("workflow:cancel");
                perms.insert("run:retry");
                perms.insert("worker:admin");
                perms.insert("worker:drain");
                perms.insert("secret:read");
                perms.insert("secret:write");
                perms.insert("project:admin");
                perms.insert("organization:admin");
                perms.insert("platform:admin");
                perms.insert("audit:read");
            }
        }

        perms
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions().contains(permission)
    }
}
