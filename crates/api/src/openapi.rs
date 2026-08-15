use axum::{http::StatusCode, response::IntoResponse, Json};

pub async fn get_openapi_json() -> impl IntoResponse {
    let spec = serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "FlowForge REST API",
            "version": "1.0.0",
            "description": "Production-grade cloud-native distributed workload orchestration platform API."
        },
        "paths": {
            "/api/v1/health/live": {
                "get": {
                    "summary": "Liveness probe",
                    "responses": { "200": { "description": "System is live" } }
                }
            },
            "/api/v1/health/ready": {
                "get": {
                    "summary": "Readiness probe",
                    "responses": { "200": { "description": "System is ready to accept traffic" } }
                }
            },
            "/api/v1/stats": {
                "get": {
                    "summary": "Platform operational metrics and KPIs",
                    "responses": { "200": { "description": "System stats" } }
                }
            },
            "/api/v1/workflows": {
                "get": {
                    "summary": "List workflows",
                    "responses": { "200": { "description": "List of workflows" } }
                },
                "post": {
                    "summary": "Apply or update a workflow definition YAML",
                    "responses": { "201": { "description": "Workflow created or updated" } }
                }
            },
            "/api/v1/workflows/validate": {
                "post": {
                    "summary": "Validate workflow YAML definition",
                    "responses": { "200": { "description": "Validation result" } }
                }
            },
            "/api/v1/workflow-runs": {
                "get": {
                    "summary": "List workflow runs",
                    "responses": { "200": { "description": "List of workflow runs" } }
                },
                "post": {
                    "summary": "Trigger a new workflow run",
                    "parameters": [{
                        "name": "Idempotency-Key",
                        "in": "header",
                        "schema": { "type": "string" }
                    }],
                    "responses": { "201": { "description": "Workflow run created" } }
                }
            },
            "/api/v1/workflow-runs/{id}": {
                "get": {
                    "summary": "Get workflow run details, DAG state, and tasks",
                    "responses": { "200": { "description": "Workflow run details" } }
                }
            },
            "/api/v1/workflow-runs/{id}/cancel": {
                "post": {
                    "summary": "Cancel a running workflow",
                    "responses": { "200": { "description": "Workflow canceled" } }
                }
            },
            "/api/v1/workers": {
                "get": {
                    "summary": "List registered workers",
                    "responses": { "200": { "description": "Worker list" } }
                }
            },
            "/api/v1/workers/{id}/drain": {
                "post": {
                    "summary": "Drain worker",
                    "responses": { "200": { "description": "Worker set to draining" } }
                }
            },
            "/api/v1/stream": {
                "get": {
                    "summary": "Server-Sent Events (SSE) live execution stream",
                    "responses": { "200": { "description": "SSE stream" } }
                }
            }
        }
    });

    (StatusCode::OK, Json(spec))
}
