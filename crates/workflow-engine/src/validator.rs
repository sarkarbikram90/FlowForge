use crate::dag::DagGraph;
use flowforge_common::{FlowForgeError, Result, WorkflowSpec};

pub struct WorkflowValidator;

impl WorkflowValidator {
    pub fn validate_spec(spec: &WorkflowSpec) -> Result<DagGraph> {
        // 1. Metadata check
        if spec.metadata.name.trim().is_empty() {
            return Err(FlowForgeError::Validation(
                "Workflow metadata.name cannot be empty".to_string(),
            ));
        }

        // 2. Tasks check
        if spec.spec.tasks.is_empty() {
            return Err(FlowForgeError::Validation(
                "Workflow spec must contain at least one task".to_string(),
            ));
        }

        // 3. Task details check
        for task in &spec.spec.tasks {
            if task.id.trim().is_empty() {
                return Err(FlowForgeError::Validation(
                    "Task id cannot be empty".to_string(),
                ));
            }

            match task.task_type.as_str() {
                "shell" => {
                    if task.command.is_none() && task.script.is_none() {
                        return Err(FlowForgeError::Validation(format!(
                            "Shell task '{}' must specify 'command' or 'script'",
                            task.id
                        )));
                    }
                }
                "container" | "docker" | "kubernetes" => {
                    if task.image.is_none() {
                        return Err(FlowForgeError::Validation(format!(
                            "Container task '{}' must specify 'image'",
                            task.id
                        )));
                    }
                }
                "http" => {
                    if task.url.is_none() {
                        return Err(FlowForgeError::Validation(format!(
                            "HTTP task '{}' must specify 'url'",
                            task.id
                        )));
                    }
                }
                "script" | "python" => {
                    if task.script.is_none() && task.command.is_none() {
                        return Err(FlowForgeError::Validation(format!(
                            "Script task '{}' must specify 'script' or 'command'",
                            task.id
                        )));
                    }
                }
                "wait" => {
                    if task.wait_secs.is_none() {
                        return Err(FlowForgeError::Validation(format!(
                            "Wait task '{}' must specify 'waitSecs'",
                            task.id
                        )));
                    }
                }
                "condition" => {
                    if task.condition.is_none() {
                        return Err(FlowForgeError::Validation(format!(
                            "Condition task '{}' must specify 'condition' expression",
                            task.id
                        )));
                    }
                }
                unknown => {
                    return Err(FlowForgeError::Validation(format!(
                        "Task '{}' has unsupported task type '{}'",
                        task.id, unknown
                    )));
                }
            }

            if task.timeout_secs == 0 {
                return Err(FlowForgeError::Validation(format!(
                    "Task '{}' timeout must be greater than 0",
                    task.id
                )));
            }
        }

        // 4. Build and validate DAG dependencies and cycles
        DagGraph::build(&spec.spec.tasks)
    }

    pub fn parse_and_validate_yaml(yaml_str: &str) -> Result<(WorkflowSpec, DagGraph)> {
        let spec: WorkflowSpec = serde_yaml::from_str(yaml_str)
            .map_err(|e| FlowForgeError::Validation(format!("Invalid YAML syntax: {}", e)))?;
        let dag = Self::validate_spec(&spec)?;
        Ok((spec, dag))
    }

    pub fn parse_and_validate_json(json_str: &str) -> Result<(WorkflowSpec, DagGraph)> {
        let spec: WorkflowSpec = serde_json::from_str(json_str)
            .map_err(|e| FlowForgeError::Validation(format!("Invalid JSON syntax: {}", e)))?;
        let dag = Self::validate_spec(&spec)?;
        Ok((spec, dag))
    }
}
