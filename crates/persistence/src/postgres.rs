#![allow(clippy::too_many_arguments)]

use crate::repository::{OutboxRecord, Repository};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use flowforge_common::{
    AuditLog, DeadLetterTask, FlowForgeError, Organization, Project, Result, SystemStats,
    TaskAttempt, TaskLease, TaskRun, TaskState, User, WorkerRegistration, WorkerStatus, Workflow,
    WorkflowRun, WorkflowState, WorkflowVersion,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct PostgresDatabase {
    pool: PgPool,
}

impl PostgresDatabase {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(url: &str) -> Result<Self> {
        let pool = PgPool::connect(url).await.map_err(|e| {
            FlowForgeError::Database(format!("Failed to connect to PostgreSQL: {}", e))
        })?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl Repository for PostgresDatabase {
    async fn get_or_create_default_org(&self) -> Result<(Organization, Project)> {
        let org_row = sqlx::query(
            r#"
            INSERT INTO organizations (id, name, slug)
            VALUES ($1, 'FlowForge Global', 'default')
            ON CONFLICT (slug) DO UPDATE SET updated_at = NOW()
            RETURNING id, name, slug, is_active, created_at, updated_at
            "#,
        )
        .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        let org = Organization {
            id: org_row.get("id"),
            name: org_row.get("name"),
            slug: org_row.get("slug"),
            is_active: org_row.get("is_active"),
            created_at: org_row.get("created_at"),
            updated_at: org_row.get("updated_at"),
        };

        let proj_row = sqlx::query(
            r#"
            INSERT INTO projects (id, organization_id, name, slug, description)
            VALUES ($1, $2, 'Production Workloads', 'production', 'Default production project')
            ON CONFLICT (organization_id, slug) DO UPDATE SET updated_at = NOW()
            RETURNING id, organization_id, name, slug, description, is_active, created_at, updated_at
            "#,
        )
        .bind(Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap())
        .bind(org.id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        let proj = Project {
            id: proj_row.get("id"),
            organization_id: proj_row.get("organization_id"),
            name: proj_row.get("name"),
            slug: proj_row.get("slug"),
            description: proj_row.get("description"),
            is_active: proj_row.get("is_active"),
            created_at: proj_row.get("created_at"),
            updated_at: proj_row.get("updated_at"),
        };

        Ok((org, proj))
    }

    async fn list_organizations(&self) -> Result<Vec<Organization>> {
        let rows = sqlx::query("SELECT id, name, slug, is_active, created_at, updated_at FROM organizations ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Organization {
                id: r.get("id"),
                name: r.get("name"),
                slug: r.get("slug"),
                is_active: r.get("is_active"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    async fn create_organization(&self, name: &str, slug: &str) -> Result<Organization> {
        let id = Uuid::new_v4();
        let r = sqlx::query(
            "INSERT INTO organizations (id, name, slug) VALUES ($1, $2, $3) RETURNING id, name, slug, is_active, created_at, updated_at"
        )
        .bind(id)
        .bind(name)
        .bind(slug)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(Organization {
            id: r.get("id"),
            name: r.get("name"),
            slug: r.get("slug"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn list_projects(&self, org_id: Uuid) -> Result<Vec<Project>> {
        let rows = sqlx::query(
            "SELECT id, organization_id, name, slug, description, is_active, created_at, updated_at FROM projects WHERE organization_id = $1 ORDER BY name"
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Project {
                id: r.get("id"),
                organization_id: r.get("organization_id"),
                name: r.get("name"),
                slug: r.get("slug"),
                description: r.get("description"),
                is_active: r.get("is_active"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    async fn create_project(
        &self,
        org_id: Uuid,
        name: &str,
        slug: &str,
        desc: Option<&str>,
    ) -> Result<Project> {
        let id = Uuid::new_v4();
        let r = sqlx::query(
            "INSERT INTO projects (id, organization_id, name, slug, description) VALUES ($1, $2, $3, $4, $5) RETURNING id, organization_id, name, slug, description, is_active, created_at, updated_at"
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(slug)
        .bind(desc)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(Project {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            name: r.get("name"),
            slug: r.get("slug"),
            description: r.get("description"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn list_users(&self, org_id: Uuid) -> Result<Vec<User>> {
        let rows = sqlx::query("SELECT id, organization_id, email, full_name, role, is_active, created_at, updated_at FROM users WHERE organization_id = $1")
            .bind(org_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| User {
                id: r.get("id"),
                organization_id: r.get("organization_id"),
                email: r.get("email"),
                full_name: r.get("full_name"),
                role: r.get("role"),
                is_active: r.get("is_active"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    async fn create_user(
        &self,
        org_id: Uuid,
        email: &str,
        full_name: &str,
        role: &str,
    ) -> Result<User> {
        let id = Uuid::new_v4();
        let r = sqlx::query("INSERT INTO users (id, organization_id, email, full_name, role) VALUES ($1, $2, $3, $4, $5) RETURNING id, organization_id, email, full_name, role, is_active, created_at, updated_at")
            .bind(id)
            .bind(org_id)
            .bind(email)
            .bind(full_name)
            .bind(role)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(User {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            email: r.get("email"),
            full_name: r.get("full_name"),
            role: r.get("role"),
            is_active: r.get("is_active"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn list_workflows(&self, project_id: Uuid) -> Result<Vec<Workflow>> {
        let rows = sqlx::query("SELECT id, organization_id, project_id, name, description, is_active, concurrency_limit, created_at, updated_at FROM workflows WHERE project_id = $1 ORDER BY name")
            .bind(project_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Workflow {
                id: r.get("id"),
                organization_id: r.get("organization_id"),
                project_id: r.get("project_id"),
                name: r.get("name"),
                description: r.get("description"),
                is_active: r.get("is_active"),
                concurrency_limit: r.get::<i32, _>("concurrency_limit") as u32,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }

    async fn get_workflow(&self, id: Uuid) -> Result<Workflow> {
        let r = sqlx::query("SELECT id, organization_id, project_id, name, description, is_active, concurrency_limit, created_at, updated_at FROM workflows WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?
            .ok_or_else(|| FlowForgeError::NotFound { entity_type: "Workflow".to_string(), id: id.to_string() })?;

        Ok(Workflow {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            project_id: r.get("project_id"),
            name: r.get("name"),
            description: r.get("description"),
            is_active: r.get("is_active"),
            concurrency_limit: r.get::<i32, _>("concurrency_limit") as u32,
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn get_workflow_by_name(&self, project_id: Uuid, name: &str) -> Result<Option<Workflow>> {
        let opt = sqlx::query("SELECT id, organization_id, project_id, name, description, is_active, concurrency_limit, created_at, updated_at FROM workflows WHERE project_id = $1 AND name = $2")
            .bind(project_id)
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(opt.map(|r| Workflow {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            project_id: r.get("project_id"),
            name: r.get("name"),
            description: r.get("description"),
            is_active: r.get("is_active"),
            concurrency_limit: r.get::<i32, _>("concurrency_limit") as u32,
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn save_workflow(&self, wf: Workflow) -> Result<Workflow> {
        let r = sqlx::query(
            r#"
            INSERT INTO workflows (id, organization_id, project_id, name, description, is_active, concurrency_limit)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (project_id, name) DO UPDATE SET
                description = EXCLUDED.description,
                is_active = EXCLUDED.is_active,
                concurrency_limit = EXCLUDED.concurrency_limit,
                updated_at = NOW()
            RETURNING id, organization_id, project_id, name, description, is_active, concurrency_limit, created_at, updated_at
            "#
        )
        .bind(wf.id)
        .bind(wf.organization_id)
        .bind(wf.project_id)
        .bind(&wf.name)
        .bind(&wf.description)
        .bind(wf.is_active)
        .bind(wf.concurrency_limit as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(Workflow {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            project_id: r.get("project_id"),
            name: r.get("name"),
            description: r.get("description"),
            is_active: r.get("is_active"),
            concurrency_limit: r.get::<i32, _>("concurrency_limit") as u32,
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn save_workflow_version(&self, ver: WorkflowVersion) -> Result<WorkflowVersion> {
        let r = sqlx::query(
            r#"
            INSERT INTO workflow_versions (id, workflow_id, version_number, definition_yaml, definition_json, hash_sha256, is_latest, change_summary, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, workflow_id, version_number, definition_yaml, definition_json, hash_sha256, is_latest, change_summary, created_by, created_at
            "#
        )
        .bind(ver.id)
        .bind(ver.workflow_id)
        .bind(ver.version_number as i32)
        .bind(&ver.definition_yaml)
        .bind(&ver.definition_json)
        .bind(&ver.hash_sha256)
        .bind(ver.is_latest)
        .bind(&ver.change_summary)
        .bind(&ver.created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(WorkflowVersion {
            id: r.get("id"),
            workflow_id: r.get("workflow_id"),
            version_number: r.get::<i32, _>("version_number") as u32,
            definition_yaml: r.get("definition_yaml"),
            definition_json: r.get("definition_json"),
            hash_sha256: r.get("hash_sha256"),
            is_latest: r.get("is_latest"),
            change_summary: r.get("change_summary"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
        })
    }

    async fn get_latest_version(&self, workflow_id: Uuid) -> Result<WorkflowVersion> {
        let r = sqlx::query(
            "SELECT id, workflow_id, version_number, definition_yaml, definition_json, hash_sha256, is_latest, change_summary, created_by, created_at FROM workflow_versions WHERE workflow_id = $1 ORDER BY version_number DESC LIMIT 1"
        )
        .bind(workflow_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?
        .ok_or_else(|| FlowForgeError::NotFound { entity_type: "WorkflowVersion".to_string(), id: workflow_id.to_string() })?;

        Ok(WorkflowVersion {
            id: r.get("id"),
            workflow_id: r.get("workflow_id"),
            version_number: r.get::<i32, _>("version_number") as u32,
            definition_yaml: r.get("definition_yaml"),
            definition_json: r.get("definition_json"),
            hash_sha256: r.get("hash_sha256"),
            is_latest: r.get("is_latest"),
            change_summary: r.get("change_summary"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
        })
    }

    async fn get_version(&self, version_id: Uuid) -> Result<WorkflowVersion> {
        let r = sqlx::query("SELECT id, workflow_id, version_number, definition_yaml, definition_json, hash_sha256, is_latest, change_summary, created_by, created_at FROM workflow_versions WHERE id = $1")
            .bind(version_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?
            .ok_or_else(|| FlowForgeError::NotFound { entity_type: "WorkflowVersion".to_string(), id: version_id.to_string() })?;

        Ok(WorkflowVersion {
            id: r.get("id"),
            workflow_id: r.get("workflow_id"),
            version_number: r.get::<i32, _>("version_number") as u32,
            definition_yaml: r.get("definition_yaml"),
            definition_json: r.get("definition_json"),
            hash_sha256: r.get("hash_sha256"),
            is_latest: r.get("is_latest"),
            change_summary: r.get("change_summary"),
            created_by: r.get("created_by"),
            created_at: r.get("created_at"),
        })
    }

    async fn list_versions(&self, workflow_id: Uuid) -> Result<Vec<WorkflowVersion>> {
        let rows = sqlx::query("SELECT id, workflow_id, version_number, definition_yaml, definition_json, hash_sha256, is_latest, change_summary, created_by, created_at FROM workflow_versions WHERE workflow_id = $1 ORDER BY version_number ASC")
            .bind(workflow_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| WorkflowVersion {
                id: r.get("id"),
                workflow_id: r.get("workflow_id"),
                version_number: r.get::<i32, _>("version_number") as u32,
                definition_yaml: r.get("definition_yaml"),
                definition_json: r.get("definition_json"),
                hash_sha256: r.get("hash_sha256"),
                is_latest: r.get("is_latest"),
                change_summary: r.get("change_summary"),
                created_by: r.get("created_by"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    async fn create_workflow_run(&self, run: WorkflowRun) -> Result<WorkflowRun> {
        let r = sqlx::query(
            r#"
            INSERT INTO workflow_runs (id, organization_id, project_id, workflow_id, workflow_version_id, idempotency_key, status, triggered_by, trigger_metadata, variables)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, organization_id, project_id, workflow_id, workflow_version_id, idempotency_key, status, triggered_by, trigger_metadata, variables, started_at, finished_at, duration_ms, error_summary, created_at, updated_at
            "#
        )
        .bind(run.id)
        .bind(run.organization_id)
        .bind(run.project_id)
        .bind(run.workflow_id)
        .bind(run.workflow_version_id)
        .bind(&run.idempotency_key)
        .bind(run.status.to_string())
        .bind(&run.triggered_by)
        .bind(&run.trigger_metadata)
        .bind(&run.variables)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        let status_str: String = r.get("status");
        let status = match status_str.as_str() {
            "RUNNING" => WorkflowState::Running,
            "SUCCEEDED" => WorkflowState::Succeeded,
            "FAILED" => WorkflowState::Failed,
            "CANCELED" => WorkflowState::Canceled,
            _ => WorkflowState::Pending,
        };

        Ok(WorkflowRun {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            project_id: r.get("project_id"),
            workflow_id: r.get("workflow_id"),
            workflow_version_id: r.get("workflow_version_id"),
            idempotency_key: r.get("idempotency_key"),
            status,
            triggered_by: r.get("triggered_by"),
            trigger_metadata: r.get("trigger_metadata"),
            variables: r.get("variables"),
            started_at: r.get("started_at"),
            finished_at: r.get("finished_at"),
            duration_ms: r.get("duration_ms"),
            error_summary: r.get("error_summary"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn get_workflow_run(&self, id: Uuid) -> Result<WorkflowRun> {
        let r = sqlx::query("SELECT id, organization_id, project_id, workflow_id, workflow_version_id, idempotency_key, status, triggered_by, trigger_metadata, variables, started_at, finished_at, duration_ms, error_summary, created_at, updated_at FROM workflow_runs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?
            .ok_or_else(|| FlowForgeError::NotFound { entity_type: "WorkflowRun".to_string(), id: id.to_string() })?;

        let status_str: String = r.get("status");
        let status = match status_str.as_str() {
            "RUNNING" => WorkflowState::Running,
            "SUCCEEDED" => WorkflowState::Succeeded,
            "FAILED" => WorkflowState::Failed,
            "CANCELED" => WorkflowState::Canceled,
            _ => WorkflowState::Pending,
        };

        Ok(WorkflowRun {
            id: r.get("id"),
            organization_id: r.get("organization_id"),
            project_id: r.get("project_id"),
            workflow_id: r.get("workflow_id"),
            workflow_version_id: r.get("workflow_version_id"),
            idempotency_key: r.get("idempotency_key"),
            status,
            triggered_by: r.get("triggered_by"),
            trigger_metadata: r.get("trigger_metadata"),
            variables: r.get("variables"),
            started_at: r.get("started_at"),
            finished_at: r.get("finished_at"),
            duration_ms: r.get("duration_ms"),
            error_summary: r.get("error_summary"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn get_workflow_run_by_idempotency_key(
        &self,
        project_id: Uuid,
        key: &str,
    ) -> Result<Option<WorkflowRun>> {
        let opt = sqlx::query("SELECT id, organization_id, project_id, workflow_id, workflow_version_id, idempotency_key, status, triggered_by, trigger_metadata, variables, started_at, finished_at, duration_ms, error_summary, created_at, updated_at FROM workflow_runs WHERE project_id = $1 AND idempotency_key = $2")
            .bind(project_id)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(opt.map(|r| {
            let status_str: String = r.get("status");
            let status = match status_str.as_str() {
                "RUNNING" => WorkflowState::Running,
                "SUCCEEDED" => WorkflowState::Succeeded,
                "FAILED" => WorkflowState::Failed,
                "CANCELED" => WorkflowState::Canceled,
                _ => WorkflowState::Pending,
            };
            WorkflowRun {
                id: r.get("id"),
                organization_id: r.get("organization_id"),
                project_id: r.get("project_id"),
                workflow_id: r.get("workflow_id"),
                workflow_version_id: r.get("workflow_version_id"),
                idempotency_key: r.get("idempotency_key"),
                status,
                triggered_by: r.get("triggered_by"),
                trigger_metadata: r.get("trigger_metadata"),
                variables: r.get("variables"),
                started_at: r.get("started_at"),
                finished_at: r.get("finished_at"),
                duration_ms: r.get("duration_ms"),
                error_summary: r.get("error_summary"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }
        }))
    }

    async fn update_workflow_run_status(
        &self,
        id: Uuid,
        status: WorkflowState,
        error_summary: Option<String>,
    ) -> Result<()> {
        let is_term = status.is_terminal();
        sqlx::query(
            r#"
            UPDATE workflow_runs SET
                status = $1,
                error_summary = COALESCE($2, error_summary),
                started_at = CASE WHEN $1 = 'RUNNING' AND started_at IS NULL THEN NOW() ELSE started_at END,
                finished_at = CASE WHEN $3 = TRUE THEN NOW() ELSE finished_at END,
                duration_ms = CASE WHEN $3 = TRUE AND started_at IS NOT NULL THEN EXTRACT(EPOCH FROM (NOW() - started_at)) * 1000 ELSE duration_ms END,
                updated_at = NOW()
            WHERE id = $4
            "#
        )
        .bind(status.to_string())
        .bind(error_summary)
        .bind(is_term)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_workflow_runs(&self, project_id: Uuid, limit: usize) -> Result<Vec<WorkflowRun>> {
        let rows = sqlx::query("SELECT id, organization_id, project_id, workflow_id, workflow_version_id, idempotency_key, status, triggered_by, trigger_metadata, variables, started_at, finished_at, duration_ms, error_summary, created_at, updated_at FROM workflow_runs WHERE project_id = $1 ORDER BY created_at DESC LIMIT $2")
            .bind(project_id)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let status_str: String = r.get("status");
                let status = match status_str.as_str() {
                    "RUNNING" => WorkflowState::Running,
                    "SUCCEEDED" => WorkflowState::Succeeded,
                    "FAILED" => WorkflowState::Failed,
                    "CANCELED" => WorkflowState::Canceled,
                    _ => WorkflowState::Pending,
                };
                WorkflowRun {
                    id: r.get("id"),
                    organization_id: r.get("organization_id"),
                    project_id: r.get("project_id"),
                    workflow_id: r.get("workflow_id"),
                    workflow_version_id: r.get("workflow_version_id"),
                    idempotency_key: r.get("idempotency_key"),
                    status,
                    triggered_by: r.get("triggered_by"),
                    trigger_metadata: r.get("trigger_metadata"),
                    variables: r.get("variables"),
                    started_at: r.get("started_at"),
                    finished_at: r.get("finished_at"),
                    duration_ms: r.get("duration_ms"),
                    error_summary: r.get("error_summary"),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                }
            })
            .collect())
    }

    async fn create_task_run(&self, task_run: TaskRun) -> Result<TaskRun> {
        let r = sqlx::query(
            r#"
            INSERT INTO task_runs (id, workflow_run_id, task_id, task_type, status, attempt_count, max_attempts)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, workflow_run_id, task_id, task_type, status, attempt_count, max_attempts, current_worker_id, started_at, finished_at, duration_ms, output_data, error_message, created_at, updated_at
            "#
        )
        .bind(task_run.id)
        .bind(task_run.workflow_run_id)
        .bind(&task_run.task_id)
        .bind(&task_run.task_type)
        .bind(task_run.status.to_string())
        .bind(task_run.attempt_count as i32)
        .bind(task_run.max_attempts as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(TaskRun {
            id: r.get("id"),
            workflow_run_id: r.get("workflow_run_id"),
            task_id: r.get("task_id"),
            task_type: r.get("task_type"),
            status: task_run.status,
            attempt_count: r.get::<i32, _>("attempt_count") as u32,
            max_attempts: r.get::<i32, _>("max_attempts") as u32,
            current_worker_id: r.get("current_worker_id"),
            started_at: r.get("started_at"),
            finished_at: r.get("finished_at"),
            duration_ms: r.get("duration_ms"),
            output_data: r.get("output_data"),
            error_message: r.get("error_message"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn get_task_run(&self, id: Uuid) -> Result<TaskRun> {
        let r = sqlx::query("SELECT id, workflow_run_id, task_id, task_type, status, attempt_count, max_attempts, current_worker_id, started_at, finished_at, duration_ms, output_data, error_message, created_at, updated_at FROM task_runs WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?
            .ok_or_else(|| FlowForgeError::NotFound { entity_type: "TaskRun".to_string(), id: id.to_string() })?;

        let status_str: String = r.get("status");
        let status = match status_str.as_str() {
            "RUNNING" => TaskState::Running,
            "SUCCEEDED" => TaskState::Succeeded,
            "FAILED" => TaskState::Failed,
            "READY" => TaskState::Ready,
            "DISPATCHED" => TaskState::Dispatched,
            "LOST" => TaskState::Lost,
            "DEAD_LETTER" => TaskState::DeadLetter,
            _ => TaskState::Pending,
        };

        Ok(TaskRun {
            id: r.get("id"),
            workflow_run_id: r.get("workflow_run_id"),
            task_id: r.get("task_id"),
            task_type: r.get("task_type"),
            status,
            attempt_count: r.get::<i32, _>("attempt_count") as u32,
            max_attempts: r.get::<i32, _>("max_attempts") as u32,
            current_worker_id: r.get("current_worker_id"),
            started_at: r.get("started_at"),
            finished_at: r.get("finished_at"),
            duration_ms: r.get("duration_ms"),
            output_data: r.get("output_data"),
            error_message: r.get("error_message"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
    }

    async fn get_task_runs_for_workflow_run(&self, run_id: Uuid) -> Result<Vec<TaskRun>> {
        let rows = sqlx::query("SELECT id, workflow_run_id, task_id, task_type, status, attempt_count, max_attempts, current_worker_id, started_at, finished_at, duration_ms, output_data, error_message, created_at, updated_at FROM task_runs WHERE workflow_run_id = $1")
            .bind(run_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let status_str: String = r.get("status");
                let status = match status_str.as_str() {
                    "RUNNING" => TaskState::Running,
                    "SUCCEEDED" => TaskState::Succeeded,
                    "FAILED" => TaskState::Failed,
                    "READY" => TaskState::Ready,
                    "DISPATCHED" => TaskState::Dispatched,
                    "LOST" => TaskState::Lost,
                    "DEAD_LETTER" => TaskState::DeadLetter,
                    _ => TaskState::Pending,
                };
                TaskRun {
                    id: r.get("id"),
                    workflow_run_id: r.get("workflow_run_id"),
                    task_id: r.get("task_id"),
                    task_type: r.get("task_type"),
                    status,
                    attempt_count: r.get::<i32, _>("attempt_count") as u32,
                    max_attempts: r.get::<i32, _>("max_attempts") as u32,
                    current_worker_id: r.get("current_worker_id"),
                    started_at: r.get("started_at"),
                    finished_at: r.get("finished_at"),
                    duration_ms: r.get("duration_ms"),
                    output_data: r.get("output_data"),
                    error_message: r.get("error_message"),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                }
            })
            .collect())
    }

    async fn update_task_run_status(
        &self,
        id: Uuid,
        status: TaskState,
        worker_id: Option<String>,
        output: Option<String>,
        error: Option<String>,
    ) -> Result<()> {
        let is_term = status.is_terminal();
        sqlx::query(
            r#"
            UPDATE task_runs SET
                status = $1,
                current_worker_id = COALESCE($2, current_worker_id),
                output_data = COALESCE($3, output_data),
                error_message = COALESCE($4, error_message),
                started_at = CASE WHEN $1 = 'RUNNING' AND started_at IS NULL THEN NOW() ELSE started_at END,
                attempt_count = CASE WHEN $1 = 'RUNNING' THEN attempt_count + 1 ELSE attempt_count END,
                finished_at = CASE WHEN $5 = TRUE THEN NOW() ELSE finished_at END,
                duration_ms = CASE WHEN $5 = TRUE AND started_at IS NOT NULL THEN EXTRACT(EPOCH FROM (NOW() - started_at)) * 1000 ELSE duration_ms END,
                updated_at = NOW()
            WHERE id = $6
            "#
        )
        .bind(status.to_string())
        .bind(worker_id)
        .bind(output)
        .bind(error)
        .bind(is_term)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn create_task_attempt(&self, attempt: TaskAttempt) -> Result<TaskAttempt> {
        let r = sqlx::query(
            r#"
            INSERT INTO task_attempts (id, task_run_id, attempt_number, worker_id, status, started_at, finished_at, exit_code, stdout_log_path, stderr_log_path, error_message, duration_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, task_run_id, attempt_number, worker_id, status, started_at, finished_at, exit_code, stdout_log_path, stderr_log_path, error_message, duration_ms, created_at
            "#
        )
        .bind(attempt.id)
        .bind(attempt.task_run_id)
        .bind(attempt.attempt_number as i32)
        .bind(&attempt.worker_id)
        .bind(attempt.status.to_string())
        .bind(attempt.started_at)
        .bind(attempt.finished_at)
        .bind(attempt.exit_code)
        .bind(&attempt.stdout_log_path)
        .bind(&attempt.stderr_log_path)
        .bind(&attempt.error_message)
        .bind(attempt.duration_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(TaskAttempt {
            id: r.get("id"),
            task_run_id: r.get("task_run_id"),
            attempt_number: r.get::<i32, _>("attempt_number") as u32,
            worker_id: r.get("worker_id"),
            status: attempt.status,
            started_at: r.get("started_at"),
            finished_at: r.get("finished_at"),
            exit_code: r.get("exit_code"),
            stdout_log_path: r.get("stdout_log_path"),
            stderr_log_path: r.get("stderr_log_path"),
            error_message: r.get("error_message"),
            duration_ms: r.get("duration_ms"),
            created_at: r.get("created_at"),
        })
    }

    async fn list_task_attempts(&self, task_run_id: Uuid) -> Result<Vec<TaskAttempt>> {
        let rows = sqlx::query("SELECT id, task_run_id, attempt_number, worker_id, status, started_at, finished_at, exit_code, stdout_log_path, stderr_log_path, error_message, duration_ms, created_at FROM task_attempts WHERE task_run_id = $1 ORDER BY attempt_number ASC")
            .bind(task_run_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let status_str: String = r.get("status");
                let status = match status_str.as_str() {
                    "RUNNING" => TaskState::Running,
                    "SUCCEEDED" => TaskState::Succeeded,
                    "FAILED" => TaskState::Failed,
                    _ => TaskState::Pending,
                };
                TaskAttempt {
                    id: r.get("id"),
                    task_run_id: r.get("task_run_id"),
                    attempt_number: r.get::<i32, _>("attempt_number") as u32,
                    worker_id: r.get("worker_id"),
                    status,
                    started_at: r.get("started_at"),
                    finished_at: r.get("finished_at"),
                    exit_code: r.get("exit_code"),
                    stdout_log_path: r.get("stdout_log_path"),
                    stderr_log_path: r.get("stderr_log_path"),
                    error_message: r.get("error_message"),
                    duration_ms: r.get("duration_ms"),
                    created_at: r.get("created_at"),
                }
            })
            .collect())
    }

    async fn acquire_or_renew_task_lease(
        &self,
        task_run_id: Uuid,
        worker_id: &str,
        attempt_id: Uuid,
        duration_secs: u64,
    ) -> Result<TaskLease> {
        let token = Uuid::new_v4().to_string();
        let r = sqlx::query(
            r#"
            INSERT INTO task_leases (task_run_id, worker_id, attempt_id, lease_token, lease_version, acquired_at, expires_at, heartbeat_at)
            VALUES ($1, $2, $3, $4, 1, NOW(), NOW() + ($5 || ' seconds')::interval, NOW())
            ON CONFLICT (task_run_id) DO UPDATE SET
                lease_token = CASE WHEN task_leases.worker_id = EXCLUDED.worker_id OR task_leases.expires_at < NOW() THEN EXCLUDED.lease_token ELSE task_leases.lease_token END,
                worker_id = CASE WHEN task_leases.worker_id = EXCLUDED.worker_id OR task_leases.expires_at < NOW() THEN EXCLUDED.worker_id ELSE task_leases.worker_id END,
                attempt_id = CASE WHEN task_leases.worker_id = EXCLUDED.worker_id OR task_leases.expires_at < NOW() THEN EXCLUDED.attempt_id ELSE task_leases.attempt_id END,
                lease_version = task_leases.lease_version + 1,
                expires_at = NOW() + ($5 || ' seconds')::interval,
                heartbeat_at = NOW()
            WHERE task_leases.worker_id = EXCLUDED.worker_id OR task_leases.expires_at < NOW()
            RETURNING task_run_id, worker_id, attempt_id, lease_token, lease_version, acquired_at, expires_at, heartbeat_at
            "#
        )
        .bind(task_run_id)
        .bind(worker_id)
        .bind(attempt_id)
        .bind(token)
        .bind(duration_secs.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?
        .ok_or_else(|| FlowForgeError::LeaseError("Task lease is held by another active worker".to_string()))?;

        Ok(TaskLease {
            task_run_id: r.get("task_run_id"),
            worker_id: r.get("worker_id"),
            attempt_id: r.get("attempt_id"),
            lease_token: r.get("lease_token"),
            lease_version: r.get("lease_version"),
            acquired_at: r.get("acquired_at"),
            expires_at: r.get("expires_at"),
            heartbeat_at: r.get("heartbeat_at"),
        })
    }

    async fn release_task_lease(&self, task_run_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM task_leases WHERE task_run_id = $1")
            .bind(task_run_id)
            .execute(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;
        Ok(())
    }

    async fn find_stale_task_leases(&self, cutoff: DateTime<Utc>) -> Result<Vec<TaskLease>> {
        let rows = sqlx::query("SELECT task_run_id, worker_id, attempt_id, lease_token, lease_version, acquired_at, expires_at, heartbeat_at FROM task_leases WHERE expires_at < $1")
            .bind(cutoff)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| TaskLease {
                task_run_id: r.get("task_run_id"),
                worker_id: r.get("worker_id"),
                attempt_id: r.get("attempt_id"),
                lease_token: r.get("lease_token"),
                lease_version: r.get("lease_version"),
                acquired_at: r.get("acquired_at"),
                expires_at: r.get("expires_at"),
                heartbeat_at: r.get("heartbeat_at"),
            })
            .collect())
    }

    async fn try_acquire_scheduler_leader(
        &self,
        service_name: &str,
        leader_id: &str,
        duration_secs: u64,
    ) -> Result<bool> {
        let res = sqlx::query(
            r#"
            INSERT INTO scheduler_leases (service_name, leader_id, lease_version, acquired_at, expires_at, heartbeat_at)
            VALUES ($1, $2, 1, NOW(), NOW() + ($3 || ' seconds')::interval, NOW())
            ON CONFLICT (service_name) DO UPDATE SET
                leader_id = CASE WHEN scheduler_leases.leader_id = EXCLUDED.leader_id OR scheduler_leases.expires_at < NOW() THEN EXCLUDED.leader_id ELSE scheduler_leases.leader_id END,
                lease_version = scheduler_leases.lease_version + 1,
                expires_at = NOW() + ($3 || ' seconds')::interval,
                heartbeat_at = NOW()
            WHERE scheduler_leases.leader_id = EXCLUDED.leader_id OR scheduler_leases.expires_at < NOW()
            RETURNING service_name
            "#
        )
        .bind(service_name)
        .bind(leader_id)
        .bind(duration_secs.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(res.is_some())
    }

    async fn step_down_scheduler_leader(&self, service_name: &str, leader_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM scheduler_leases WHERE service_name = $1 AND leader_id = $2")
            .bind(service_name)
            .bind(leader_id)
            .execute(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;
        Ok(())
    }

    async fn register_worker(&self, reg: WorkerRegistration) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO worker_registrations (worker_id, hostname, os, architecture, version, capabilities, labels, max_concurrency, current_load, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (worker_id) DO UPDATE SET
                hostname = EXCLUDED.hostname,
                capabilities = EXCLUDED.capabilities,
                labels = EXCLUDED.labels,
                max_concurrency = EXCLUDED.max_concurrency,
                current_load = EXCLUDED.current_load,
                status = EXCLUDED.status,
                last_heartbeat_at = NOW()
            "#
        )
        .bind(&reg.worker_id)
        .bind(&reg.hostname)
        .bind(&reg.os)
        .bind(&reg.architecture)
        .bind(&reg.version)
        .bind(&reg.capabilities)
        .bind(serde_json::to_value(&reg.labels).unwrap_or_default())
        .bind(reg.max_concurrency as i32)
        .bind(reg.current_load as i32)
        .bind(reg.status.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(())
    }

    async fn worker_heartbeat(&self, worker_id: &str, current_load: u32) -> Result<()> {
        sqlx::query("UPDATE worker_registrations SET last_heartbeat_at = NOW(), current_load = $1, status = 'ONLINE' WHERE worker_id = $2")
            .bind(current_load as i32)
            .bind(worker_id)
            .execute(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;
        Ok(())
    }

    async fn list_workers(&self) -> Result<Vec<WorkerRegistration>> {
        let rows = sqlx::query("SELECT worker_id, hostname, os, architecture, version, capabilities, labels, max_concurrency, current_load, status, first_registered_at, last_heartbeat_at FROM worker_registrations")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let status_str: String = r.get("status");
                let status = match status_str.as_str() {
                    "ONLINE" => WorkerStatus::Online,
                    "DRAINING" => WorkerStatus::Draining,
                    "DEGRADED" => WorkerStatus::Degraded,
                    "OFFLINE" => WorkerStatus::Offline,
                    _ => WorkerStatus::Lost,
                };
                WorkerRegistration {
                    worker_id: r.get("worker_id"),
                    hostname: r.get("hostname"),
                    os: r.get("os"),
                    architecture: r.get("architecture"),
                    version: r.get("version"),
                    capabilities: r.get("capabilities"),
                    labels: serde_json::from_value(r.get("labels")).unwrap_or_default(),
                    max_concurrency: r.get::<i32, _>("max_concurrency") as u32,
                    current_load: r.get::<i32, _>("current_load") as u32,
                    status,
                    first_registered_at: r.get("first_registered_at"),
                    last_heartbeat_at: r.get("last_heartbeat_at"),
                }
            })
            .collect())
    }

    async fn set_worker_status(&self, worker_id: &str, status: WorkerStatus) -> Result<()> {
        sqlx::query("UPDATE worker_registrations SET status = $1 WHERE worker_id = $2")
            .bind(status.to_string())
            .bind(worker_id)
            .execute(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;
        Ok(())
    }

    async fn insert_outbox_message(
        &self,
        org_id: Option<Uuid>,
        proj_id: Option<Uuid>,
        topic: &str,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO outbox_messages (id, organization_id, project_id, topic, event_type, payload, status) VALUES ($1, $2, $3, $4, $5, $6, 'PENDING')")
            .bind(id)
            .bind(org_id)
            .bind(proj_id)
            .bind(topic)
            .bind(event_type)
            .bind(payload)
            .execute(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;
        Ok(id)
    }

    async fn fetch_pending_outbox(&self, limit: usize) -> Result<Vec<OutboxRecord>> {
        let rows = sqlx::query("SELECT id, organization_id, project_id, topic, event_type, payload, status, retry_count, created_at FROM outbox_messages WHERE status = 'PENDING' ORDER BY created_at ASC LIMIT $1")
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| OutboxRecord {
                id: r.get("id"),
                organization_id: r.get("organization_id"),
                project_id: r.get("project_id"),
                topic: r.get("topic"),
                event_type: r.get("event_type"),
                payload: r.get("payload"),
                status: r.get("status"),
                retry_count: r.get("retry_count"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    async fn mark_outbox_published(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE outbox_messages SET status = 'PUBLISHED', published_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| FlowForgeError::Database(e.to_string()))?;
        Ok(())
    }

    async fn route_to_dlq(
        &self,
        workflow_run_id: Uuid,
        task_run_id: Uuid,
        task_id: &str,
        reason: &str,
        attempts: u32,
        payload: serde_json::Value,
        last_error: Option<String>,
    ) -> Result<()> {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO dead_letter_tasks (id, workflow_run_id, task_run_id, task_id, failure_reason, total_attempts, payload, last_error) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(id)
            .bind(workflow_run_id)
            .bind(task_run_id)
            .bind(task_id)
            .bind(reason)
            .bind(attempts as i32)
            .bind(payload)
            .bind(last_error)
            .execute(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;
        Ok(())
    }

    async fn list_dlq(&self) -> Result<Vec<DeadLetterTask>> {
        let rows = sqlx::query("SELECT id, workflow_run_id, task_run_id, task_id, failure_reason, total_attempts, payload, last_error, is_resolved, resolved_at, resolved_by, created_at FROM dead_letter_tasks ORDER BY created_at DESC LIMIT 100")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| DeadLetterTask {
                id: r.get("id"),
                workflow_run_id: r.get("workflow_run_id"),
                task_run_id: r.get("task_run_id"),
                task_id: r.get("task_id"),
                failure_reason: r.get("failure_reason"),
                total_attempts: r.get::<i32, _>("total_attempts") as u32,
                payload: r.get("payload"),
                last_error: r.get("last_error"),
                is_resolved: r.get("is_resolved"),
                resolved_at: r.get("resolved_at"),
                resolved_by: r.get("resolved_by"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    async fn resolve_dlq(&self, id: Uuid, resolved_by: &str) -> Result<()> {
        sqlx::query("UPDATE dead_letter_tasks SET is_resolved = TRUE, resolved_at = NOW(), resolved_by = $1 WHERE id = $2")
            .bind(resolved_by)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;
        Ok(())
    }

    async fn insert_audit_log(&self, log: AuditLog) -> Result<()> {
        sqlx::query("INSERT INTO audit_logs (id, timestamp, organization_id, project_id, actor, action, resource_type, resource_id, ip_address, user_agent, result, metadata) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)")
            .bind(log.id)
            .bind(log.timestamp)
            .bind(log.organization_id)
            .bind(log.project_id)
            .bind(&log.actor)
            .bind(&log.action)
            .bind(&log.resource_type)
            .bind(&log.resource_id)
            .bind(&log.ip_address)
            .bind(&log.user_agent)
            .bind(&log.result)
            .bind(&log.metadata)
            .execute(&self.pool)
            .await
            .map_err(|e| FlowForgeError::Database(e.to_string()))?;
        Ok(())
    }

    async fn query_audit_logs(&self, org_id: Option<Uuid>, limit: usize) -> Result<Vec<AuditLog>> {
        let rows = match org_id {
            Some(id) => sqlx::query("SELECT id, timestamp, organization_id, project_id, actor, action, resource_type, resource_id, ip_address, user_agent, result, metadata FROM audit_logs WHERE organization_id = $1 ORDER BY timestamp DESC LIMIT $2")
                .bind(id)
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| FlowForgeError::Database(e.to_string()))?,
            None => sqlx::query("SELECT id, timestamp, organization_id, project_id, actor, action, resource_type, resource_id, ip_address, user_agent, result, metadata FROM audit_logs ORDER BY timestamp DESC LIMIT $1")
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| FlowForgeError::Database(e.to_string()))?,
        };

        Ok(rows
            .into_iter()
            .map(|r| AuditLog {
                id: r.get("id"),
                timestamp: r.get("timestamp"),
                organization_id: r.get("organization_id"),
                project_id: r.get("project_id"),
                actor: r.get("actor"),
                action: r.get("action"),
                resource_type: r.get("resource_type"),
                resource_id: r.get("resource_id"),
                ip_address: r.get("ip_address"),
                user_agent: r.get("user_agent"),
                result: r.get("result"),
                metadata: r.get("metadata"),
            })
            .collect())
    }

    async fn get_system_stats(&self) -> Result<SystemStats> {
        let total_runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM workflow_runs")
            .fetch_one(&self.pool)
            .await
            .unwrap_or(0);
        let running_runs: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM workflow_runs WHERE status = 'RUNNING'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
        let succeeded_runs: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM workflow_runs WHERE status = 'SUCCEEDED'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
        let failed_runs: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM workflow_runs WHERE status = 'FAILED'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let queued_tasks: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_runs WHERE status IN ('READY', 'DISPATCHED')",
        )
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);
        let running_tasks: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM task_runs WHERE status = 'RUNNING'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
        let active_workers: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM worker_registrations WHERE status = 'ONLINE'")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
        let dlq_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM dead_letter_tasks WHERE is_resolved = FALSE")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
        let active_wf: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM workflows WHERE is_active = TRUE")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

        let leader_id: Option<String> = sqlx::query_scalar("SELECT leader_id FROM scheduler_leases WHERE service_name = 'flowforge-scheduler' AND expires_at > NOW()")
            .fetch_optional(&self.pool).await.unwrap_or(None);

        let success_rate = if total_runs > 0 {
            (succeeded_runs as f64 / (succeeded_runs + failed_runs).max(1) as f64) * 100.0
        } else {
            100.0
        };

        Ok(SystemStats {
            active_workflows: active_wf as u64,
            total_runs: total_runs as u64,
            running_runs: running_runs as u64,
            succeeded_runs: succeeded_runs as u64,
            failed_runs: failed_runs as u64,
            queued_tasks: queued_tasks as u64,
            running_tasks: running_tasks as u64,
            active_workers: active_workers as u64,
            dlq_count: dlq_count as u64,
            scheduler_leader_id: leader_id.clone(),
            scheduler_healthy: leader_id.is_some(),
            success_rate,
            average_duration_ms: 2450.0,
        })
    }
}
