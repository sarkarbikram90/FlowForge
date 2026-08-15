#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use crate::repository::{OutboxRecord, Repository};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use flowforge_common::{
    AuditLog, DeadLetterTask, FlowForgeError, Organization, Project, Result, SystemStats,
    TaskAttempt, TaskLease, TaskRun, TaskState, User, WorkerRegistration, WorkerStatus, Workflow,
    WorkflowRun, WorkflowState, WorkflowVersion,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct InMemoryDatabase {
    orgs: Arc<RwLock<HashMap<Uuid, Organization>>>,
    projects: Arc<RwLock<HashMap<Uuid, Project>>>,
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    workflows: Arc<RwLock<HashMap<Uuid, Workflow>>>,
    versions: Arc<RwLock<HashMap<Uuid, WorkflowVersion>>>,
    runs: Arc<RwLock<HashMap<Uuid, WorkflowRun>>>,
    task_runs: Arc<RwLock<HashMap<Uuid, TaskRun>>>,
    task_attempts: Arc<RwLock<HashMap<Uuid, Vec<TaskAttempt>>>>,
    leases: Arc<RwLock<HashMap<Uuid, TaskLease>>>,
    scheduler_leader: Arc<RwLock<Option<(String, String, DateTime<Utc>, i64)>>>, // service, leader, expires, version
    workers: Arc<RwLock<HashMap<String, WorkerRegistration>>>,
    outbox: Arc<RwLock<Vec<OutboxRecord>>>,
    dlq: Arc<RwLock<HashMap<Uuid, DeadLetterTask>>>,
    audit_logs: Arc<RwLock<Vec<AuditLog>>>,
}

impl InMemoryDatabase {
    pub fn new() -> Self {
        let db = Self::default();
        // Seed default organization and project
        let org_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();
        let proj_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();

        db.orgs.write().insert(
            org_id,
            Organization {
                id: org_id,
                name: "FlowForge Global".to_string(),
                slug: "default".to_string(),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        db.projects.write().insert(
            proj_id,
            Project {
                id: proj_id,
                organization_id: org_id,
                name: "Production Workloads".to_string(),
                slug: "production".to_string(),
                description: Some("Default production workloads project".to_string()),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        db
    }
}

#[async_trait]
impl Repository for InMemoryDatabase {
    async fn get_or_create_default_org(&self) -> Result<(Organization, Project)> {
        let org = self.orgs.read().values().next().cloned().unwrap();
        let proj = self.projects.read().values().next().cloned().unwrap();
        Ok((org, proj))
    }

    async fn list_organizations(&self) -> Result<Vec<Organization>> {
        Ok(self.orgs.read().values().cloned().collect())
    }

    async fn create_organization(&self, name: &str, slug: &str) -> Result<Organization> {
        let org = Organization {
            id: Uuid::new_v4(),
            name: name.to_string(),
            slug: slug.to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.orgs.write().insert(org.id, org.clone());
        Ok(org)
    }

    async fn list_projects(&self, org_id: Uuid) -> Result<Vec<Project>> {
        Ok(self
            .projects
            .read()
            .values()
            .filter(|p| p.organization_id == org_id)
            .cloned()
            .collect())
    }

    async fn create_project(
        &self,
        org_id: Uuid,
        name: &str,
        slug: &str,
        desc: Option<&str>,
    ) -> Result<Project> {
        let proj = Project {
            id: Uuid::new_v4(),
            organization_id: org_id,
            name: name.to_string(),
            slug: slug.to_string(),
            description: desc.map(|s| s.to_string()),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.projects.write().insert(proj.id, proj.clone());
        Ok(proj)
    }

    async fn list_users(&self, org_id: Uuid) -> Result<Vec<User>> {
        Ok(self
            .users
            .read()
            .values()
            .filter(|u| u.organization_id == org_id)
            .cloned()
            .collect())
    }

    async fn create_user(
        &self,
        org_id: Uuid,
        email: &str,
        full_name: &str,
        role: &str,
    ) -> Result<User> {
        let user = User {
            id: Uuid::new_v4(),
            organization_id: org_id,
            email: email.to_string(),
            full_name: full_name.to_string(),
            role: role.to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        self.users.write().insert(user.id, user.clone());
        Ok(user)
    }

    async fn list_workflows(&self, project_id: Uuid) -> Result<Vec<Workflow>> {
        Ok(self
            .workflows
            .read()
            .values()
            .filter(|w| w.project_id == project_id)
            .cloned()
            .collect())
    }

    async fn get_workflow(&self, id: Uuid) -> Result<Workflow> {
        self.workflows
            .read()
            .get(&id)
            .cloned()
            .ok_or_else(|| FlowForgeError::NotFound {
                entity_type: "Workflow".to_string(),
                id: id.to_string(),
            })
    }

    async fn get_workflow_by_name(&self, project_id: Uuid, name: &str) -> Result<Option<Workflow>> {
        Ok(self
            .workflows
            .read()
            .values()
            .find(|w| w.project_id == project_id && w.name == name)
            .cloned())
    }

    async fn save_workflow(&self, wf: Workflow) -> Result<Workflow> {
        self.workflows.write().insert(wf.id, wf.clone());
        Ok(wf)
    }

    async fn save_workflow_version(&self, ver: WorkflowVersion) -> Result<WorkflowVersion> {
        self.versions.write().insert(ver.id, ver.clone());
        Ok(ver)
    }

    async fn get_latest_version(&self, workflow_id: Uuid) -> Result<WorkflowVersion> {
        let versions = self.versions.read();
        let mut matching: Vec<WorkflowVersion> = versions
            .values()
            .filter(|v| v.workflow_id == workflow_id)
            .cloned()
            .collect();
        matching.sort_by_key(|v| v.version_number);
        matching
            .last()
            .cloned()
            .ok_or_else(|| FlowForgeError::NotFound {
                entity_type: "WorkflowVersion".to_string(),
                id: workflow_id.to_string(),
            })
    }

    async fn get_version(&self, version_id: Uuid) -> Result<WorkflowVersion> {
        self.versions
            .read()
            .get(&version_id)
            .cloned()
            .ok_or_else(|| FlowForgeError::NotFound {
                entity_type: "WorkflowVersion".to_string(),
                id: version_id.to_string(),
            })
    }

    async fn list_versions(&self, workflow_id: Uuid) -> Result<Vec<WorkflowVersion>> {
        Ok(self
            .versions
            .read()
            .values()
            .filter(|v| v.workflow_id == workflow_id)
            .cloned()
            .collect())
    }

    async fn create_workflow_run(&self, run: WorkflowRun) -> Result<WorkflowRun> {
        self.runs.write().insert(run.id, run.clone());
        Ok(run)
    }

    async fn get_workflow_run(&self, id: Uuid) -> Result<WorkflowRun> {
        self.runs
            .read()
            .get(&id)
            .cloned()
            .ok_or_else(|| FlowForgeError::NotFound {
                entity_type: "WorkflowRun".to_string(),
                id: id.to_string(),
            })
    }

    async fn get_workflow_run_by_idempotency_key(
        &self,
        project_id: Uuid,
        key: &str,
    ) -> Result<Option<WorkflowRun>> {
        Ok(self
            .runs
            .read()
            .values()
            .find(|r| r.project_id == project_id && r.idempotency_key.as_deref() == Some(key))
            .cloned())
    }

    async fn update_workflow_run_status(
        &self,
        id: Uuid,
        status: WorkflowState,
        error_summary: Option<String>,
    ) -> Result<()> {
        let mut runs = self.runs.write();
        if let Some(run) = runs.get_mut(&id) {
            run.status = status;
            if status.is_terminal() {
                run.finished_at = Some(Utc::now());
                if let Some(started) = run.started_at {
                    run.duration_ms = Some((Utc::now() - started).num_milliseconds());
                }
            } else if status == WorkflowState::Running && run.started_at.is_none() {
                run.started_at = Some(Utc::now());
            }
            if error_summary.is_some() {
                run.error_summary = error_summary;
            }
            run.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn list_workflow_runs(&self, project_id: Uuid, limit: usize) -> Result<Vec<WorkflowRun>> {
        let mut runs: Vec<WorkflowRun> = self
            .runs
            .read()
            .values()
            .filter(|r| r.project_id == project_id)
            .cloned()
            .collect();
        runs.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        runs.truncate(limit);
        Ok(runs)
    }

    async fn create_task_run(&self, task_run: TaskRun) -> Result<TaskRun> {
        self.task_runs.write().insert(task_run.id, task_run.clone());
        Ok(task_run)
    }

    async fn get_task_run(&self, id: Uuid) -> Result<TaskRun> {
        self.task_runs
            .read()
            .get(&id)
            .cloned()
            .ok_or_else(|| FlowForgeError::NotFound {
                entity_type: "TaskRun".to_string(),
                id: id.to_string(),
            })
    }

    async fn get_task_runs_for_workflow_run(&self, run_id: Uuid) -> Result<Vec<TaskRun>> {
        Ok(self
            .task_runs
            .read()
            .values()
            .filter(|t| t.workflow_run_id == run_id)
            .cloned()
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
        let mut task_runs = self.task_runs.write();
        if let Some(t) = task_runs.get_mut(&id) {
            t.status = status;
            if let Some(w) = worker_id {
                t.current_worker_id = Some(w);
            }
            if output.is_some() {
                t.output_data = output;
            }
            if error.is_some() {
                t.error_message = error;
            }
            if status.is_terminal() {
                t.finished_at = Some(Utc::now());
                if let Some(st) = t.started_at {
                    t.duration_ms = Some((Utc::now() - st).num_milliseconds());
                }
            } else if status == TaskState::Running && t.started_at.is_none() {
                t.started_at = Some(Utc::now());
                t.attempt_count += 1;
            }
            t.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn create_task_attempt(&self, attempt: TaskAttempt) -> Result<TaskAttempt> {
        self.task_attempts
            .write()
            .entry(attempt.task_run_id)
            .or_default()
            .push(attempt.clone());
        Ok(attempt)
    }

    async fn list_task_attempts(&self, task_run_id: Uuid) -> Result<Vec<TaskAttempt>> {
        Ok(self
            .task_attempts
            .read()
            .get(&task_run_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn acquire_or_renew_task_lease(
        &self,
        task_run_id: Uuid,
        worker_id: &str,
        attempt_id: Uuid,
        duration_secs: u64,
    ) -> Result<TaskLease> {
        let mut leases = self.leases.write();
        let now = Utc::now();
        let expires = now + chrono::Duration::seconds(duration_secs as i64);

        if let Some(existing) = leases.get_mut(&task_run_id) {
            if existing.worker_id == worker_id {
                existing.expires_at = expires;
                existing.heartbeat_at = now;
                existing.lease_version += 1;
                return Ok(existing.clone());
            } else if existing.expires_at < now {
                // Lease expired, allow new worker takeover
                let lease = TaskLease {
                    task_run_id,
                    worker_id: worker_id.to_string(),
                    attempt_id,
                    lease_token: Uuid::new_v4().to_string(),
                    lease_version: existing.lease_version + 1,
                    acquired_at: now,
                    expires_at: expires,
                    heartbeat_at: now,
                };
                leases.insert(task_run_id, lease.clone());
                return Ok(lease);
            } else {
                return Err(FlowForgeError::LeaseError(format!(
                    "Task lease currently held by active worker '{}'",
                    existing.worker_id
                )));
            }
        }

        let lease = TaskLease {
            task_run_id,
            worker_id: worker_id.to_string(),
            attempt_id,
            lease_token: Uuid::new_v4().to_string(),
            lease_version: 1,
            acquired_at: now,
            expires_at: expires,
            heartbeat_at: now,
        };
        leases.insert(task_run_id, lease.clone());
        Ok(lease)
    }

    async fn release_task_lease(&self, task_run_id: Uuid) -> Result<()> {
        self.leases.write().remove(&task_run_id);
        Ok(())
    }

    async fn find_stale_task_leases(&self, cutoff: DateTime<Utc>) -> Result<Vec<TaskLease>> {
        Ok(self
            .leases
            .read()
            .values()
            .filter(|l| l.expires_at < cutoff)
            .cloned()
            .collect())
    }

    async fn try_acquire_scheduler_leader(
        &self,
        service_name: &str,
        leader_id: &str,
        duration_secs: u64,
    ) -> Result<bool> {
        let mut leader_guard = self.scheduler_leader.write();
        let now = Utc::now();
        let expires = now + chrono::Duration::seconds(duration_secs as i64);

        if let Some((_svc, current_leader, current_expires, ver)) = &mut *leader_guard {
            if current_leader == leader_id {
                *current_expires = expires;
                *ver += 1;
                return Ok(true);
            } else if *current_expires < now {
                let new_ver = *ver + 1;
                *leader_guard = Some((
                    service_name.to_string(),
                    leader_id.to_string(),
                    expires,
                    new_ver,
                ));
                return Ok(true);
            } else {
                return Ok(false); // another leader active
            }
        }

        *leader_guard = Some((service_name.to_string(), leader_id.to_string(), expires, 1));
        Ok(true)
    }

    async fn step_down_scheduler_leader(&self, _service_name: &str, leader_id: &str) -> Result<()> {
        let mut leader_guard = self.scheduler_leader.write();
        if let Some((_, current_leader, _, _)) = &*leader_guard {
            if current_leader == leader_id {
                *leader_guard = None;
            }
        }
        Ok(())
    }

    async fn register_worker(&self, reg: WorkerRegistration) -> Result<()> {
        self.workers.write().insert(reg.worker_id.clone(), reg);
        Ok(())
    }

    async fn worker_heartbeat(&self, worker_id: &str, current_load: u32) -> Result<()> {
        let mut workers = self.workers.write();
        if let Some(w) = workers.get_mut(worker_id) {
            w.last_heartbeat_at = Utc::now();
            w.current_load = current_load;
            if w.status == WorkerStatus::Lost || w.status == WorkerStatus::Offline {
                w.status = WorkerStatus::Online;
            }
        }
        Ok(())
    }

    async fn list_workers(&self) -> Result<Vec<WorkerRegistration>> {
        Ok(self.workers.read().values().cloned().collect())
    }

    async fn set_worker_status(&self, worker_id: &str, status: WorkerStatus) -> Result<()> {
        let mut workers = self.workers.write();
        if let Some(w) = workers.get_mut(worker_id) {
            w.status = status;
        }
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
        let record = OutboxRecord {
            id,
            organization_id: org_id,
            project_id: proj_id,
            topic: topic.to_string(),
            event_type: event_type.to_string(),
            payload,
            status: "PENDING".to_string(),
            retry_count: 0,
            created_at: Utc::now(),
        };
        self.outbox.write().push(record);
        Ok(id)
    }

    async fn fetch_pending_outbox(&self, limit: usize) -> Result<Vec<OutboxRecord>> {
        let outbox = self.outbox.read();
        Ok(outbox
            .iter()
            .filter(|o| o.status == "PENDING")
            .take(limit)
            .cloned()
            .collect())
    }

    async fn mark_outbox_published(&self, id: Uuid) -> Result<()> {
        let mut outbox = self.outbox.write();
        if let Some(m) = outbox.iter_mut().find(|o| o.id == id) {
            m.status = "PUBLISHED".to_string();
        }
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
        let dlq_task = DeadLetterTask {
            id,
            workflow_run_id,
            task_run_id,
            task_id: task_id.to_string(),
            failure_reason: reason.to_string(),
            total_attempts: attempts,
            payload,
            last_error,
            is_resolved: false,
            resolved_at: None,
            resolved_by: None,
            created_at: Utc::now(),
        };
        self.dlq.write().insert(id, dlq_task);
        Ok(())
    }

    async fn list_dlq(&self) -> Result<Vec<DeadLetterTask>> {
        Ok(self.dlq.read().values().cloned().collect())
    }

    async fn resolve_dlq(&self, id: Uuid, resolved_by: &str) -> Result<()> {
        let mut dlq = self.dlq.write();
        if let Some(item) = dlq.get_mut(&id) {
            item.is_resolved = true;
            item.resolved_at = Some(Utc::now());
            item.resolved_by = Some(resolved_by.to_string());
        }
        Ok(())
    }

    async fn insert_audit_log(&self, log: AuditLog) -> Result<()> {
        self.audit_logs.write().push(log);
        Ok(())
    }

    async fn query_audit_logs(&self, org_id: Option<Uuid>, limit: usize) -> Result<Vec<AuditLog>> {
        let logs = self.audit_logs.read();
        let mut filtered: Vec<AuditLog> = match org_id {
            Some(id) => logs
                .iter()
                .filter(|l| l.organization_id == Some(id))
                .cloned()
                .collect(),
            None => logs.clone(),
        };
        filtered.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        filtered.truncate(limit);
        Ok(filtered)
    }

    async fn get_system_stats(&self) -> Result<SystemStats> {
        let runs = self.runs.read();
        let tasks = self.task_runs.read();
        let workers = self.workers.read();
        let dlq = self.dlq.read();
        let leader = self.scheduler_leader.read();

        let total_runs = runs.len() as u64;
        let running_runs = runs
            .values()
            .filter(|r| r.status == WorkflowState::Running)
            .count() as u64;
        let succeeded_runs = runs
            .values()
            .filter(|r| r.status == WorkflowState::Succeeded)
            .count() as u64;
        let failed_runs = runs
            .values()
            .filter(|r| r.status == WorkflowState::Failed)
            .count() as u64;

        let queued_tasks = tasks
            .values()
            .filter(|t| t.status == TaskState::Ready || t.status == TaskState::Dispatched)
            .count() as u64;
        let running_tasks = tasks
            .values()
            .filter(|t| t.status == TaskState::Running)
            .count() as u64;
        let active_workers = workers
            .values()
            .filter(|w| w.status == WorkerStatus::Online)
            .count() as u64;
        let dlq_count = dlq.values().filter(|d| !d.is_resolved).count() as u64;

        let success_rate = if total_runs > 0 {
            (succeeded_runs as f64 / (succeeded_runs + failed_runs).max(1) as f64) * 100.0
        } else {
            100.0
        };

        let durations: Vec<i64> = runs.values().filter_map(|r| r.duration_ms).collect();
        let average_duration_ms = if !durations.is_empty() {
            durations.iter().sum::<i64>() as f64 / durations.len() as f64
        } else {
            0.0
        };

        Ok(SystemStats {
            active_workflows: self.workflows.read().len() as u64,
            total_runs,
            running_runs,
            succeeded_runs,
            failed_runs,
            queued_tasks,
            running_tasks,
            active_workers,
            dlq_count,
            scheduler_leader_id: leader.as_ref().map(|(_, l, _, _)| l.clone()),
            scheduler_healthy: leader.is_some(),
            success_rate,
            average_duration_ms,
        })
    }
}
