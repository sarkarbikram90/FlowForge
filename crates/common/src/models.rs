use crate::retry::RetryPolicy;
use crate::state::{TaskState, WorkflowState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Identity & Multi-Tenancy ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub project_id: Option<Uuid>,
    pub name: String,
    pub key_prefix: String,
    pub role: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ─── Workflow Definition (YAML/JSON Schema) ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    pub metadata: WorkflowMetadata,
    pub spec: WorkflowBody,
}

fn default_api_version() -> String {
    "flowforge.io/v1".to_string()
}
fn default_kind() -> String {
    "Workflow".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowBody {
    #[serde(default)]
    pub schedule: Option<ScheduleSpec>,
    #[serde(default)]
    pub concurrency: Option<ConcurrencySpec>,
    #[serde(default)]
    pub retries: Option<RetryPolicy>,
    #[serde(default)]
    pub sla: Option<SlaSpec>,
    #[serde(default)]
    pub parameters: HashMap<String, ParameterSpec>,
    pub tasks: Vec<TaskSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSpec {
    #[serde(default)]
    pub cron: Option<String>,
    #[serde(default)]
    pub interval_secs: Option<u64>,
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencySpec {
    #[serde(default = "default_max_runs")]
    pub max_runs: u32,
}

fn default_max_runs() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaSpec {
    pub completion_time: String, // e.g. "30m", "2h"
    #[serde(default = "default_sla_severity")]
    pub severity: String,
}

fn default_sla_severity() -> String {
    "high".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSpec {
    pub param_type: String,
    pub default: Option<serde_json::Value>,
    pub description: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub task_type: String, // shell, container, http, script, wait, condition
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(rename = "dependsOn", default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub retries: Option<RetryPolicy>,
    #[serde(rename = "timeoutSecs", default = "default_task_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(rename = "waitSecs", default)]
    pub wait_secs: Option<u64>,
}

fn default_task_timeout() -> u64 {
    300
}

// ─── Core Workflow Entities ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub concurrency_limit: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVersion {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub version_number: u32,
    pub definition_yaml: String,
    pub definition_json: serde_json::Value,
    pub hash_sha256: String,
    pub is_latest: bool,
    pub change_summary: Option<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub project_id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_version_id: Uuid,
    pub idempotency_key: Option<String>,
    pub status: WorkflowState,
    pub triggered_by: String,
    pub trigger_metadata: serde_json::Value,
    pub variables: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub error_summary: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRun {
    pub id: Uuid,
    pub workflow_run_id: Uuid,
    pub task_id: String,
    pub task_type: String,
    pub status: TaskState,
    pub attempt_count: u32,
    pub max_attempts: u32,
    pub current_worker_id: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub output_data: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAttempt {
    pub id: Uuid,
    pub task_run_id: Uuid,
    pub attempt_number: u32,
    pub worker_id: String,
    pub status: TaskState,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub stdout_log_path: Option<String>,
    pub stderr_log_path: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLease {
    pub task_run_id: Uuid,
    pub worker_id: String,
    pub attempt_id: Uuid,
    pub lease_token: String,
    pub lease_version: i64,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub heartbeat_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRegistration {
    pub worker_id: String,
    pub hostname: String,
    pub os: String,
    pub architecture: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub labels: HashMap<String, String>,
    pub max_concurrency: u32,
    pub current_load: u32,
    pub status: WorkerStatus,
    pub first_registered_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkerStatus {
    Online,
    Degraded,
    Draining,
    Offline,
    Lost,
}

impl std::fmt::Display for WorkerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerStatus::Online => write!(f, "ONLINE"),
            WorkerStatus::Degraded => write!(f, "DEGRADED"),
            WorkerStatus::Draining => write!(f, "DRAINING"),
            WorkerStatus::Offline => write!(f, "OFFLINE"),
            WorkerStatus::Lost => write!(f, "LOST"),
        }
    }
}

// ─── Messaging / Dispatch ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDispatchMessage {
    pub organization_id: Uuid,
    pub project_id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_run_id: Uuid,
    pub task_run_id: Uuid,
    pub task_id: String,
    pub task_type: String,
    pub attempt_number: u32,
    pub max_attempts: u32,
    pub timeout_secs: u64,
    pub command: Option<String>,
    pub script: Option<String>,
    pub image: Option<String>,
    pub url: Option<String>,
    pub method: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub env: HashMap<String, String>,
    pub wait_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletionMessage {
    pub task_run_id: Uuid,
    pub attempt_id: Uuid,
    pub worker_id: String,
    pub status: TaskState,
    pub exit_code: Option<i32>,
    pub output_data: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: i64,
}

// ─── Reliability, DLQ, Audit & Health ───

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterTask {
    pub id: Uuid,
    pub workflow_run_id: Uuid,
    pub task_run_id: Uuid,
    pub task_id: String,
    pub failure_reason: String,
    pub total_attempts: u32,
    pub payload: serde_json::Value,
    pub last_error: Option<String>,
    pub is_resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub organization_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub actor: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub result: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub active_workflows: u64,
    pub total_runs: u64,
    pub running_runs: u64,
    pub succeeded_runs: u64,
    pub failed_runs: u64,
    pub queued_tasks: u64,
    pub running_tasks: u64,
    pub active_workers: u64,
    pub dlq_count: u64,
    pub scheduler_leader_id: Option<String>,
    pub scheduler_healthy: bool,
    pub success_rate: f64,
    pub average_duration_ms: f64,
}
