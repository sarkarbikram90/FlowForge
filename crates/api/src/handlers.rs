use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    Json,
};
use chrono::Utc;
use flowforge_auth::{ApiKeyManager, AuthContext, Role};
use flowforge_common::{
    AuditLog, Result, TaskState, WorkerStatus, Workflow, WorkflowRun, WorkflowSpec,
    WorkflowState,
};
use flowforge_workflow_engine::{DagGraph, WorkflowCompiler, WorkflowValidator};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use uuid::Uuid;
use crate::state::AppState;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub request_id: String,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(code: &str, message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code.to_string(),
                message: message.to_string(),
                request_id: Uuid::new_v4().to_string(),
            }),
        }
    }
}

// ─── Health & Metrics Handlers ───

pub async fn health_live() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "LIVE" })))
}

pub async fn health_ready(State(state): State<AppState>) -> impl IntoResponse {
    let stats = state.repo.get_system_stats().await;
    match stats {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({ "status": "READY" }))),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "status": "DEGRADED", "error": e.to_string() })),
        ),
    }
}

pub async fn health_startup() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "STARTED" })))
}

pub async fn get_system_stats(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.get_system_stats().await {
        Ok(stats) => (StatusCode::OK, Json(ApiResponse::ok(stats))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("STATS_ERROR", &e.to_string())),
        ),
    }
}

// ─── Auth Handlers ───

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    let (org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("AUTH_ERROR", &e.to_string()))),
    };

    let user_id = Uuid::new_v4();
    let auth = AuthContext {
        user_id,
        organization_id: org.id,
        project_id: proj.id,
        email: payload.email.clone(),
        role: Role::PlatformAdmin,
    };

    (StatusCode::OK, Json(ApiResponse::ok(auth)))
}

pub async fn whoami() -> impl IntoResponse {
    let default_auth = AuthContext::default();
    (StatusCode::OK, Json(ApiResponse::ok(default_auth)))
}

pub async fn generate_api_key(State(state): State<AppState>) -> impl IntoResponse {
    let (raw_key, prefix, hash) = ApiKeyManager::generate();
    let res = serde_json::json!({
        "api_key": raw_key,
        "prefix": prefix,
        "hash": hash,
        "created_at": Utc::now()
    });
    (StatusCode::CREATED, Json(ApiResponse::ok(res)))
}

// ─── Organization & Project Handlers ───

pub async fn list_organizations(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.list_organizations().await {
        Ok(orgs) => (StatusCode::OK, Json(ApiResponse::ok(orgs))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("ORG_ERROR", &e.to_string()))),
    }
}

pub async fn list_projects(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.list_projects(org_id).await {
        Ok(projs) => (StatusCode::OK, Json(ApiResponse::ok(projs))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("PROJ_ERROR", &e.to_string()))),
    }
}

// ─── Workflows Handlers ───

#[derive(Deserialize)]
pub struct WorkflowApplyRequest {
    pub yaml: String,
}

pub async fn apply_workflow(
    State(state): State<AppState>,
    Json(payload): Json<WorkflowApplyRequest>,
) -> impl IntoResponse {
    let (spec, _dag) = match WorkflowValidator::parse_and_validate_yaml(&payload.yaml) {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::err("VALIDATION_ERROR", &e.to_string())),
            )
        }
    };

    let (org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::err("DB_ERROR", &e.to_string())),
            )
        }
    };

    // Check if workflow exists or create new
    let existing = state
        .repo
        .get_workflow_by_name(proj.id, &spec.metadata.name)
        .await
        .unwrap_or(None);

    let (workflow, version_num) = match existing {
        Some(wf) => {
            let versions = state.repo.list_versions(wf.id).await.unwrap_or_default();
            let next_ver = versions.len() as u32 + 1;
            (wf, next_ver)
        }
        None => {
            let wf = Workflow {
                id: Uuid::new_v4(),
                organization_id: org.id,
                project_id: proj.id,
                name: spec.metadata.name.clone(),
                description: spec.metadata.description.clone(),
                is_active: true,
                concurrency_limit: spec
                    .spec
                    .concurrency
                    .as_ref()
                    .map(|c| c.max_runs)
                    .unwrap_or(10),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let saved_wf = match state.repo.save_workflow(wf).await {
                Ok(w) => w,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::err("DB_ERROR", &e.to_string())),
                    )
                }
            };
            (saved_wf, 1)
        }
    };

    // Compile and save version
    let version = match WorkflowCompiler::compile_version(
        workflow.id,
        version_num,
        &payload.yaml,
        "admin",
    ) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::err("COMPILER_ERROR", &e.to_string())),
            )
        }
    };

    if let Err(e) = state.repo.save_workflow_version(version.clone()).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::err("DB_ERROR", &e.to_string())),
        );
    }

    // Audit log
    let _ = state
        .repo
        .insert_audit_log(AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            organization_id: Some(org.id),
            project_id: Some(proj.id),
            actor: "admin".to_string(),
            action: "WORKFLOW_APPLIED".to_string(),
            resource_type: "workflow".to_string(),
            resource_id: Some(workflow.id.to_string()),
            ip_address: None,
            user_agent: None,
            result: "SUCCESS".to_string(),
            metadata: serde_json::json!({ "name": workflow.name, "version": version_num }),
        })
        .await;

    state.broadcast_event(&format!("workflow_applied:{}", workflow.id));

    (
        StatusCode::CREATED,
        Json(ApiResponse::ok(serde_json::json!({
            "workflow": workflow,
            "version": version
        }))),
    )
}

pub async fn list_workflows(State(state): State<AppState>) -> impl IntoResponse {
    let (_org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("DB_ERROR", &e.to_string()))),
    };

    match state.repo.list_workflows(proj.id).await {
        Ok(wfs) => (StatusCode::OK, Json(ApiResponse::ok(wfs))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("DB_ERROR", &e.to_string()))),
    }
}

pub async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.repo.get_workflow(id).await {
        Ok(wf) => {
            let version = state.repo.get_latest_version(wf.id).await.ok();
            (
                StatusCode::OK,
                Json(ApiResponse::ok(serde_json::json!({
                    "workflow": wf,
                    "latest_version": version
                }))),
            )
        }
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::err("NOT_FOUND", &e.to_string()))),
    }
}

pub async fn validate_workflow(Json(payload): Json<WorkflowApplyRequest>) -> impl IntoResponse {
    match WorkflowValidator::parse_and_validate_yaml(&payload.yaml) {
        Ok((spec, dag)) => {
            let order = dag.topological_order();
            let roots = dag.get_roots();
            (
                StatusCode::OK,
                Json(ApiResponse::ok(serde_json::json!({
                    "valid": true,
                    "workflow_name": spec.metadata.name,
                    "task_count": spec.spec.tasks.len(),
                    "topological_order": order,
                    "root_tasks": roots
                }))),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err("VALIDATION_ERROR", &e.to_string())),
        ),
    }
}

// ─── Workflow Runs Handlers ───

#[derive(Deserialize)]
pub struct TriggerRunRequest {
    pub workflow_id: Option<Uuid>,
    pub workflow_name: Option<String>,
    #[serde(default)]
    pub variables: serde_json::Value,
}

pub async fn trigger_workflow_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<TriggerRunRequest>,
) -> impl IntoResponse {
    let (_org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("DB_ERROR", &e.to_string()))),
    };

    // Check idempotency header
    let idempotency_key = headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(key) = &idempotency_key {
        if let Ok(Some(existing_run)) = state
            .repo
            .get_workflow_run_by_idempotency_key(proj.id, key)
            .await
        {
            return (StatusCode::OK, Json(ApiResponse::ok(existing_run)));
        }
    }

    // Resolve workflow
    let workflow = if let Some(id) = payload.workflow_id {
        state.repo.get_workflow(id).await
    } else if let Some(name) = &payload.workflow_name {
        state
            .repo
            .get_workflow_by_name(proj.id, name)
            .await
            .and_then(|opt| {
                opt.ok_or_else(|| {
                    flowforge_common::FlowForgeError::NotFound {
                        entity_type: "Workflow".to_string(),
                        id: name.clone(),
                    }
                })
            })
    } else {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::err("BAD_REQUEST", "Must provide workflow_id or workflow_name")),
        );
    };

    let wf = match workflow {
        Ok(w) => w,
        Err(e) => return (StatusCode::NOT_FOUND, Json(ApiResponse::err("NOT_FOUND", &e.to_string()))),
    };

    let latest_ver = match state.repo.get_latest_version(wf.id).await {
        Ok(v) => v,
        Err(e) => return (StatusCode::NOT_FOUND, Json(ApiResponse::err("VERSION_NOT_FOUND", &e.to_string()))),
    };

    let run_id = Uuid::new_v4();
    let run = WorkflowRun {
        id: run_id,
        organization_id: wf.organization_id,
        project_id: wf.project_id,
        workflow_id: wf.id,
        workflow_version_id: latest_ver.id,
        idempotency_key,
        status: WorkflowState::Pending,
        triggered_by: "api".to_string(),
        trigger_metadata: serde_json::json!({ "api_version": "v1" }),
        variables: payload.variables,
        started_at: None,
        finished_at: None,
        duration_ms: None,
        error_summary: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created_run = match state.repo.create_workflow_run(run).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("DB_ERROR", &e.to_string()))),
    };

    // Audit log
    let _ = state
        .repo
        .insert_audit_log(AuditLog {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            organization_id: Some(wf.organization_id),
            project_id: Some(wf.project_id),
            actor: "api".to_string(),
            action: "WORKFLOW_TRIGGERED".to_string(),
            resource_type: "workflow_run".to_string(),
            resource_id: Some(run_id.to_string()),
            ip_address: None,
            user_agent: None,
            result: "SUCCESS".to_string(),
            metadata: serde_json::json!({ "workflow_name": wf.name }),
        })
        .await;

    state.broadcast_event(&format!("run_triggered:{}", run_id));

    (StatusCode::CREATED, Json(ApiResponse::ok(created_run)))
}

pub async fn list_workflow_runs(State(state): State<AppState>) -> impl IntoResponse {
    let (_org, proj) = match state.repo.get_or_create_default_org().await {
        Ok(res) => res,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("DB_ERROR", &e.to_string()))),
    };

    match state.repo.list_workflow_runs(proj.id, 50).await {
        Ok(runs) => (StatusCode::OK, Json(ApiResponse::ok(runs))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("DB_ERROR", &e.to_string()))),
    }
}

pub async fn get_workflow_run(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let run = match state.repo.get_workflow_run(id).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::NOT_FOUND, Json(ApiResponse::err("NOT_FOUND", &e.to_string()))),
    };

    let tasks = state
        .repo
        .get_task_runs_for_workflow_run(id)
        .await
        .unwrap_or_default();
    let version = state.repo.get_version(run.workflow_version_id).await.ok();
    let workflow = state.repo.get_workflow(run.workflow_id).await.ok();

    (
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({
            "run": run,
            "workflow": workflow,
            "version": version,
            "tasks": tasks
        }))),
    )
}

pub async fn cancel_workflow_run(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(e) = state
        .repo
        .update_workflow_run_status(
            id,
            WorkflowState::Canceled,
            Some("Canceled by operator request".to_string()),
        )
        .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("CANCEL_ERROR", &e.to_string())));
    }

    state.broadcast_event(&format!("run_canceled:{}", id));
    (
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "status": "CANCELED" }))),
    )
}

// ─── Workers & Queues Handlers ───

pub async fn list_workers(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.list_workers().await {
        Ok(workers) => (StatusCode::OK, Json(ApiResponse::ok(workers))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("WORKER_ERROR", &e.to_string()))),
    }
}

pub async fn drain_worker(
    State(state): State<AppState>,
    Path(worker_id): Path<String>,
) -> impl IntoResponse {
    if let Err(e) = state
        .repo
        .set_worker_status(&worker_id, WorkerStatus::Draining)
        .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("DRAIN_ERROR", &e.to_string())));
    }

    state.broadcast_event(&format!("worker_draining:{}", worker_id));
    (
        StatusCode::OK,
        Json(ApiResponse::ok(serde_json::json!({ "status": "DRAINING", "worker_id": worker_id }))),
    )
}

pub async fn list_dlq(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.list_dlq().await {
        Ok(dlq) => (StatusCode::OK, Json(ApiResponse::ok(dlq))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("DLQ_ERROR", &e.to_string()))),
    }
}

pub async fn resolve_dlq_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    if let Err(e) = state.repo.resolve_dlq(id, "admin").await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("DLQ_ERROR", &e.to_string())));
    }
    (StatusCode::OK, Json(ApiResponse::ok(serde_json::json!({ "resolved": true }))))
}

// ─── Audit Log Handlers ───

pub async fn query_audit_logs(State(state): State<AppState>) -> impl IntoResponse {
    match state.repo.query_audit_logs(None, 100).await {
        Ok(logs) => (StatusCode::OK, Json(ApiResponse::ok(logs))),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse::err("AUDIT_ERROR", &e.to_string()))),
    }
}

// ─── Real-Time Live Execution SSE Stream ───

pub async fn execution_stream(
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = std::result::Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|item| {
        item.ok().map(|msg| Ok(Event::default().data(msg)))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
