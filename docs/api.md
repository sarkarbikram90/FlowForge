# FlowForge REST API Specification (v1)

Base URL: `http://localhost:8080/api/v1`

## Endpoints Summary

### Health & Observability
- `GET /health/live`: Liveness probe
- `GET /health/ready`: Readiness probe checking database & messaging connectivity
- `GET /stats`: Operational metrics (runs, tasks, workers, DLQ, success rate)
- `GET /metrics`: Prometheus metric scrape endpoint
- `GET /openapi.json`: OpenAPI 3.1 schema specification

### Workflows
- `GET /workflows`: List all workflows
- `POST /workflows`: Apply/update workflow YAML definition
- `GET /workflows/:id`: Get workflow details and latest immutable version
- `POST /workflows/validate`: Validate workflow YAML definition

### Workflow Runs
- `GET /workflow-runs`: List workflow runs
- `POST /workflow-runs`: Trigger a new run (supports `Idempotency-Key` header)
- `GET /workflow-runs/:id`: Get run details, DAG state, and tasks
- `POST /workflow-runs/:id/cancel`: Cancel an active workflow run

### Workers Fleet
- `GET /workers`: List registered workers and current loads
- `POST /workers/:id/drain`: Set worker to `DRAINING` mode

### Queues & Dead Letter Subsystem
- `GET /dlq`: List dead letter tasks
- `POST /dlq/:id/resolve`: Resolve and requeue dead letter task

### Audit Trail
- `GET /audit`: Query immutable audit logs
