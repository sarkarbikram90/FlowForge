use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlowForgeError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("DAG validation error: {0}")]
    DagValidation(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("DAG not found: {0}")]
    DagNotFound(String),

    #[error("Run not found: {0}")]
    RunNotFound(String),

    #[error("Cycle detected in DAG: {0}")]
    CycleDetected(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, FlowForgeError>;
