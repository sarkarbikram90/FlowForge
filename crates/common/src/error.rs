use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlowForgeError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Workflow error: {0}")]
    Workflow(String),

    #[error("Cycle detected in DAG: {0}")]
    CycleDetected(String),

    #[error("Invalid state transition: from {from} to {to} on {entity_type} {id}")]
    InvalidStateTransition {
        entity_type: String,
        id: String,
        from: String,
        to: String,
    },

    #[error("Resource not found: {entity_type} with ID '{id}'")]
    NotFound { entity_type: String, id: String },

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Lease error: {0}")]
    LeaseError(String),

    #[error("Fencing violation: lease version {current} is stale (active version is {active})")]
    FencingViolation { current: i64, active: i64 },

    #[error("Execution failed for task {task_id}: {reason}")]
    ExecutionFailed { task_id: String, reason: String },

    #[error("Execution timed out after {timeout_secs}s")]
    ExecutionTimeout { timeout_secs: u64 },

    #[error("Database error: {0}")]
    Database(String),

    #[error("Messaging error: {0}")]
    Messaging(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, FlowForgeError>;
