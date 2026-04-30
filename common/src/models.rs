use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ─── DAG Definition (what user submits as YAML) ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub schedule: Option<String>, // cron expression
    #[serde(default = "default_max_retries")]
    pub default_retries: u32,
    pub tasks: Vec<TaskDefinition>,
}

fn default_max_retries() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub retries: Option<u32>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

fn default_timeout() -> u64 {
    300 // 5 minutes
}

// ─── Database Models ───

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Dag {
    pub id: Uuid,
    pub dag_id: String,
    pub name: String,
    pub description: String,
    pub schedule: Option<String>,
    pub default_retries: i32,
    pub definition: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum RunStatus {
    #[serde(rename = "pending")]
    #[sqlx(rename = "pending")]
    Pending,
    #[serde(rename = "running")]
    #[sqlx(rename = "running")]
    Running,
    #[serde(rename = "success")]
    #[sqlx(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    #[sqlx(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    #[sqlx(rename = "cancelled")]
    Cancelled,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunStatus::Pending => write!(f, "pending"),
            RunStatus::Running => write!(f, "running"),
            RunStatus::Success => write!(f, "success"),
            RunStatus::Failed => write!(f, "failed"),
            RunStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DagRun {
    pub id: Uuid,
    pub dag_id: String,
    pub status: String,
    pub triggered_by: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[sqlx(rename_all = "snake_case")]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    #[sqlx(rename = "pending")]
    Pending,
    #[serde(rename = "queued")]
    #[sqlx(rename = "queued")]
    Queued,
    #[serde(rename = "running")]
    #[sqlx(rename = "running")]
    Running,
    #[serde(rename = "success")]
    #[sqlx(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    #[sqlx(rename = "failed")]
    Failed,
    #[serde(rename = "retrying")]
    #[sqlx(rename = "retrying")]
    Retrying,
    #[serde(rename = "skipped")]
    #[sqlx(rename = "skipped")]
    Skipped,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Queued => write!(f, "queued"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Success => write!(f, "success"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Retrying => write!(f, "retrying"),
            TaskStatus::Skipped => write!(f, "skipped"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskInstance {
    pub id: Uuid,
    pub run_id: Uuid,
    pub task_id: String,
    pub dag_id: String,
    pub status: String,
    pub attempt: i32,
    pub max_retries: i32,
    pub command: String,
    pub worker_id: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ─── Queue Messages ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    pub task_instance_id: Uuid,
    pub run_id: Uuid,
    pub dag_id: String,
    pub task_id: String,
    pub command: String,
    pub attempt: i32,
    pub max_retries: i32,
    pub timeout_secs: u64,
    pub env: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_instance_id: Uuid,
    pub run_id: Uuid,
    pub dag_id: String,
    pub task_id: String,
    pub status: String,
    pub attempt: i32,
    pub output: Option<String>,
    pub error: Option<String>,
    pub worker_id: String,
    pub duration_ms: u64,
}

// ─── API Response Types ───

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TriggerRunRequest {
    pub dag_id: String,
    #[serde(default = "default_triggered_by")]
    pub triggered_by: String,
}

fn default_triggered_by() -> String {
    "api".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DagSubmitRequest {
    pub yaml: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerHeartbeat {
    pub worker_id: String,
    pub timestamp: DateTime<Utc>,
    pub active_tasks: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub active_dags: i64,
    pub total_runs: i64,
    pub running_tasks: i64,
    pub active_workers: i64,
    pub scheduler_healthy: bool,
}
