use async_trait::async_trait;
use chrono::{DateTime, Utc};
use flowforge_common::{
    AuditLog, DeadLetterTask, Organization, Project, Result, SystemStats, TaskAttempt, TaskLease,
    TaskRun, TaskState, User, WorkerRegistration, WorkerStatus, Workflow, WorkflowRun,
    WorkflowState, WorkflowVersion,
};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutboxRecord {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub topic: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
#[async_trait]
pub trait Repository: Send + Sync {
    // Identity & Tenancy
    async fn get_or_create_default_org(&self) -> Result<(Organization, Project)>;
    async fn list_organizations(&self) -> Result<Vec<Organization>>;
    async fn create_organization(&self, name: &str, slug: &str) -> Result<Organization>;
    async fn list_projects(&self, org_id: Uuid) -> Result<Vec<Project>>;
    async fn create_project(
        &self,
        org_id: Uuid,
        name: &str,
        slug: &str,
        desc: Option<&str>,
    ) -> Result<Project>;
    async fn list_users(&self, org_id: Uuid) -> Result<Vec<User>>;
    async fn create_user(
        &self,
        org_id: Uuid,
        email: &str,
        full_name: &str,
        role: &str,
    ) -> Result<User>;

    // Workflows & Versions
    async fn list_workflows(&self, project_id: Uuid) -> Result<Vec<Workflow>>;
    async fn get_workflow(&self, id: Uuid) -> Result<Workflow>;
    async fn get_workflow_by_name(&self, project_id: Uuid, name: &str) -> Result<Option<Workflow>>;
    async fn save_workflow(&self, wf: Workflow) -> Result<Workflow>;
    async fn save_workflow_version(&self, ver: WorkflowVersion) -> Result<WorkflowVersion>;
    async fn get_latest_version(&self, workflow_id: Uuid) -> Result<WorkflowVersion>;
    async fn get_version(&self, version_id: Uuid) -> Result<WorkflowVersion>;
    async fn list_versions(&self, workflow_id: Uuid) -> Result<Vec<WorkflowVersion>>;

    // Executions
    async fn create_workflow_run(&self, run: WorkflowRun) -> Result<WorkflowRun>;
    async fn get_workflow_run(&self, id: Uuid) -> Result<WorkflowRun>;
    async fn get_workflow_run_by_idempotency_key(
        &self,
        project_id: Uuid,
        key: &str,
    ) -> Result<Option<WorkflowRun>>;
    async fn update_workflow_run_status(
        &self,
        id: Uuid,
        status: WorkflowState,
        error_summary: Option<String>,
    ) -> Result<()>;
    async fn list_workflow_runs(&self, project_id: Uuid, limit: usize) -> Result<Vec<WorkflowRun>>;

    // Task Runs & Attempts
    async fn create_task_run(&self, task_run: TaskRun) -> Result<TaskRun>;
    async fn get_task_run(&self, id: Uuid) -> Result<TaskRun>;
    async fn get_task_runs_for_workflow_run(&self, run_id: Uuid) -> Result<Vec<TaskRun>>;
    async fn update_task_run_status(
        &self,
        id: Uuid,
        status: TaskState,
        worker_id: Option<String>,
        output: Option<String>,
        error: Option<String>,
    ) -> Result<()>;
    async fn create_task_attempt(&self, attempt: TaskAttempt) -> Result<TaskAttempt>;
    async fn list_task_attempts(&self, task_run_id: Uuid) -> Result<Vec<TaskAttempt>>;

    // Leases & Heartbeats
    async fn acquire_or_renew_task_lease(
        &self,
        task_run_id: Uuid,
        worker_id: &str,
        attempt_id: Uuid,
        duration_secs: u64,
    ) -> Result<TaskLease>;
    async fn release_task_lease(&self, task_run_id: Uuid) -> Result<()>;
    async fn find_stale_task_leases(&self, cutoff: DateTime<Utc>) -> Result<Vec<TaskLease>>;

    // Scheduler Leader Election
    async fn try_acquire_scheduler_leader(
        &self,
        service_name: &str,
        leader_id: &str,
        duration_secs: u64,
    ) -> Result<bool>;
    async fn step_down_scheduler_leader(&self, service_name: &str, leader_id: &str) -> Result<()>;

    // Worker Registrations
    async fn register_worker(&self, reg: WorkerRegistration) -> Result<()>;
    async fn worker_heartbeat(&self, worker_id: &str, current_load: u32) -> Result<()>;
    async fn list_workers(&self) -> Result<Vec<WorkerRegistration>>;
    async fn set_worker_status(&self, worker_id: &str, status: WorkerStatus) -> Result<()>;

    // Outbox & Events
    async fn insert_outbox_message(
        &self,
        org_id: Option<Uuid>,
        proj_id: Option<Uuid>,
        topic: &str,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<Uuid>;
    async fn fetch_pending_outbox(&self, limit: usize) -> Result<Vec<OutboxRecord>>;
    async fn mark_outbox_published(&self, id: Uuid) -> Result<()>;

    // Dead Letter Queue
    async fn route_to_dlq(
        &self,
        workflow_run_id: Uuid,
        task_run_id: Uuid,
        task_id: &str,
        reason: &str,
        attempts: u32,
        payload: serde_json::Value,
        last_error: Option<String>,
    ) -> Result<()>;
    async fn list_dlq(&self) -> Result<Vec<DeadLetterTask>>;
    async fn resolve_dlq(&self, id: Uuid, resolved_by: &str) -> Result<()>;

    // Audit Logging
    async fn insert_audit_log(&self, log: AuditLog) -> Result<()>;
    async fn query_audit_logs(&self, org_id: Option<Uuid>, limit: usize) -> Result<Vec<AuditLog>>;

    // Stats
    async fn get_system_stats(&self) -> Result<SystemStats>;
}
