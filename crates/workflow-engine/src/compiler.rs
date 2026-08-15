use crate::validator::WorkflowValidator;
use flowforge_common::{Result, WorkflowVersion};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct WorkflowCompiler;

impl WorkflowCompiler {
    pub fn compile_version(
        workflow_id: Uuid,
        version_number: u32,
        yaml_content: &str,
        created_by: &str,
    ) -> Result<WorkflowVersion> {
        let (spec, _dag) = WorkflowValidator::parse_and_validate_yaml(yaml_content)?;

        // Compute deterministic SHA-256 hash of the canonical JSON representation
        let json_value = serde_json::to_value(&spec)
            .map_err(|e| flowforge_common::FlowForgeError::Internal(e.to_string()))?;
        let canonical_json_bytes = serde_json::to_vec(&json_value)
            .map_err(|e| flowforge_common::FlowForgeError::Internal(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(&canonical_json_bytes);
        let hash_sha256 = format!("{:x}", hasher.finalize());

        Ok(WorkflowVersion {
            id: Uuid::new_v4(),
            workflow_id,
            version_number,
            definition_yaml: yaml_content.to_string(),
            definition_json: json_value,
            hash_sha256,
            is_latest: true,
            change_summary: Some(format!("Workflow version {} compiled", version_number)),
            created_by: created_by.to_string(),
            created_at: chrono::Utc::now(),
        })
    }
}
