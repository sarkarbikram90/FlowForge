use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use crate::handlers;
use crate::openapi::get_openapi_json;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health & Stats
        .route("/api/v1/health/live", get(handlers::health_live))
        .route("/api/v1/health/ready", get(handlers::health_ready))
        .route("/api/v1/health/startup", get(handlers::health_startup))
        .route("/api/v1/stats", get(handlers::get_system_stats))
        // Auth
        .route("/api/v1/auth/login", post(handlers::login))
        .route("/api/v1/auth/whoami", get(handlers::whoami))
        .route("/api/v1/auth/keys", post(handlers::generate_api_key))
        // Tenancy
        .route("/api/v1/organizations", get(handlers::list_organizations))
        .route("/api/v1/organizations/:org_id/projects", get(handlers::list_projects))
        // Workflows
        .route("/api/v1/workflows", get(handlers::list_workflows).post(handlers::apply_workflow))
        .route("/api/v1/workflows/:id", get(handlers::get_workflow))
        .route("/api/v1/workflows/validate", post(handlers::validate_workflow))
        // Runs
        .route("/api/v1/workflow-runs", get(handlers::list_workflow_runs).post(handlers::trigger_workflow_run))
        .route("/api/v1/workflow-runs/:id", get(handlers::get_workflow_run))
        .route("/api/v1/workflow-runs/:id/cancel", post(handlers::cancel_workflow_run))
        // Workers
        .route("/api/v1/workers", get(handlers::list_workers))
        .route("/api/v1/workers/:worker_id/drain", post(handlers::drain_worker))
        // DLQ
        .route("/api/v1/dlq", get(handlers::list_dlq))
        .route("/api/v1/dlq/:id/resolve", post(handlers::resolve_dlq_item))
        // Audit
        .route("/api/v1/audit", get(handlers::query_audit_logs))
        // Real-Time SSE Stream
        .route("/api/v1/stream", get(handlers::execution_stream))
        // OpenAPI Spec
        .route("/api/v1/openapi.json", get(get_openapi_json))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
