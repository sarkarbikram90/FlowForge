# FlowForge ⚡

**A lightweight distributed workflow scheduler written in Rust.**

FlowForge is a DAG-based workflow orchestration system — similar to Apache Airflow — but optimized for performance, simplicity, and reliability. It features task scheduling with dependency resolution, distributed worker execution, retry logic with exponential backoff, a REST API, CLI, and web UI.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        FlowForge System                         │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────┐    ┌───────────┐    ┌──────────┐    ┌───────────┐  │
│  │   CLI   │───▶│  REST API │◀───│    UI    │    │  Metrics  │  │
│  └─────────┘    │  (Axum)   │    │ (React)  │    │/metrics   │  │
│                 └─────┬─────┘    └──────────┘    └───────────┘  │
│                       │                                          │
│              ┌────────▼────────┐                                 │
│              │    Scheduler    │                                  │
│              │  - DAG Parser   │                                  │
│              │  - Dep Resolver │                                  │
│              │  - Cron Ticker  │                                  │
│              │  - Crash Recov. │                                  │
│              └────────┬────────┘                                  │
│                       │                                          │
│              ┌────────▼────────┐                                 │
│              │   Redis Queue   │                                  │
│              │  (BRPOPLPUSH)   │                                  │
│              └────────┬────────┘                                  │
│                       │                                          │
│         ┌─────────────┼─────────────┐                            │
│         ▼             ▼             ▼                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                       │
│  │ Worker 1 │  │ Worker 2 │  │ Worker N │                       │
│  └──────┬───┘  └──────┬───┘  └──────┬───┘                       │
│         └─────────────┼─────────────┘                            │
│                       ▼                                          │
│              ┌─────────────────┐                                 │
│              │   PostgreSQL    │                                  │
│              └─────────────────┘                                 │
└──────────────────────────────────────────────────────────────────┘
```

## Features

- **DAG-based orchestration** — Define workflows as YAML with task dependencies
- **Distributed execution** — Multiple workers process tasks concurrently
- **Retry with exponential backoff** — Configurable per-task retry limits
- **Fault tolerance** — Worker heartbeat monitoring, automatic task requeuing
- **Cron scheduling** — Time-based DAG triggers via standard cron expressions
- **REST API** — Full CRUD for DAGs, runs, and tasks
- **CLI** — Submit DAGs, trigger runs, inspect status from the terminal
- **Web UI** — Dashboard with DAG visualization and task status tracking
- **Observability** — Structured JSON logging (tracing), Prometheus metrics
- **Idempotent execution** — `(run_id, task_id)` uniqueness prevents duplicates

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (stable) |
| Web Framework | Axum 0.7 |
| Async Runtime | Tokio |
| Database | PostgreSQL 16 |
| Queue | Redis 7 |
| Serialization | Serde (JSON + YAML) |
| Logging | tracing + tracing-subscriber |
| Metrics | metrics + metrics-exporter-prometheus |
| DAG Engine | petgraph (topological sort) |
| CLI | clap 4 |
| UI | React 18 + Vite |

## Project Structure

```
.
├── Cargo.toml           # Workspace root
├── common/              # Shared types, models, DB, queue abstractions
├── scheduler/           # Core brain — dependency resolution, task enqueuing
├── worker/              # Task executor — shell commands, retries, heartbeat
├── api/                 # REST API server (Axum)
├── cli/                 # Command-line interface (clap)
├── ui/                  # React web dashboard
├── examples/            # Sample DAG definitions
├── infra/               # Kubernetes manifests
├── docker-compose.yml   # Full-stack local setup
├── Dockerfile           # Multi-stage build
└── Makefile             # Build automation
```

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Rust 1.75+ (for local development)

### One-command setup

```bash
make up
```

This starts PostgreSQL, Redis, the API server, Scheduler, Workers, and the UI.

### Local Development

```bash
# Start infrastructure only
make dev-infra

# Run components locally (in separate terminals)
make dev-api        # API on :8080
make dev-scheduler  # Scheduler
make dev-worker     # Worker
```

### Seed example DAGs

```bash
make seed
```

### Trigger a run

```bash
make run-hello
```

---

## Example DAG

```yaml
id: etl-pipeline
name: ETL Data Pipeline
description: Extract, transform, and load data
schedule: "0 0 * * * *"  # Every hour
default_retries: 3

tasks:
  - id: extract-users
    name: Extract Users
    command: "echo 'Extracting users...' && sleep 2"
    depends_on: []
    timeout_secs: 120

  - id: extract-orders
    name: Extract Orders
    command: "echo 'Extracting orders...' && sleep 2"
    depends_on: []

  - id: transform
    name: Transform Data
    command: "echo 'Transforming...' && sleep 3"
    depends_on:
      - extract-users
      - extract-orders

  - id: load
    name: Load to Warehouse
    command: "echo 'Loading...' && sleep 1"
    depends_on:
      - transform
    retries: 5
    timeout_secs: 300
    env:
      WAREHOUSE_URL: "postgres://warehouse:5432/analytics"
```

---

## CLI Usage

```bash
# Submit a DAG from YAML
flowforge submit --file examples/etl-pipeline.yaml

# Trigger a DAG run
flowforge trigger etl-pipeline

# List all DAGs
flowforge dags

# List recent runs
flowforge runs

# View run details with task statuses
flowforge run <run-id>

# System status
flowforge status

# List active workers
flowforge workers

# Point to a different API server
flowforge --api-url http://production:8080 status
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/metrics` | Prometheus metrics |
| `GET` | `/api/v1/dags` | List all DAGs |
| `POST` | `/api/v1/dags` | Submit a DAG (JSON body: `{"yaml": "..."}`) |
| `GET` | `/api/v1/dags/{dag_id}` | Get DAG details |
| `DELETE` | `/api/v1/dags/{dag_id}` | Deactivate a DAG |
| `POST` | `/api/v1/runs` | Trigger a run (JSON: `{"dag_id": "...", "triggered_by": "cli"}`) |
| `GET` | `/api/v1/runs` | List recent runs |
| `GET` | `/api/v1/runs/{run_id}` | Get run details |
| `GET` | `/api/v1/runs/{run_id}/tasks` | Get task instances for a run |
| `GET` | `/api/v1/status` | System status overview |
| `GET` | `/api/v1/workers` | List active workers |

## Design Decisions

### Why Redis BRPOPLPUSH?
Provides atomic move of a task from the pending queue to a processing queue, preventing task loss if a worker crashes mid-dequeue. The processing queue acts as a visibility timeout — the scheduler can requeue orphaned tasks.

### Why petgraph for DAG validation?
petgraph's topological sort detects cycles at parse time and produces a valid execution order. This is more robust than hand-rolled graph traversal and handles complex diamond dependencies correctly.

### Why inline schema instead of migrations?
Using `CREATE TABLE IF NOT EXISTS` in code ensures the schema is always applied on startup without requiring separate migration tooling. For production, you'd typically use `sqlx migrate` — the schema is structured to be additive.

### Single scheduler, multiple workers
The scheduler is a single process that resolves dependencies and enqueues tasks. Workers are stateless and horizontally scalable. This avoids distributed coordination complexity while still supporting high throughput.

### Exponential backoff for retries
Failed tasks retry with `delay = 2^attempt` seconds, capped at 60s. This prevents thundering herd effects and gives transient failures time to resolve.

## Trade-offs

| Decision | Benefit | Cost |
|----------|---------|------|
| Redis as queue | Simple, fast, widely available | Not durable by default (enable AOF for persistence) |
| Single scheduler | Simple state management | Single point of failure (mitigate with health checks + restart) |
| Shell command execution | Maximum flexibility | Security risk (sandbox recommended for untrusted workloads) |
| Polling-based scheduling | Simple implementation | Slightly higher latency than event-driven (configurable interval) |
| PostgreSQL for metadata | ACID guarantees, rich queries | Heavier than SQLite for small deployments |

## Observability

### Structured Logging
All components emit JSON-formatted structured logs via the `tracing` crate:
```json
{"timestamp":"2024-01-01T00:00:00Z","level":"INFO","target":"scheduler","message":"Task enqueued","task_id":"extract-users","run_id":"..."}
```

Set log level via `RUST_LOG` environment variable:
```bash
RUST_LOG=debug cargo run --bin flowforge-scheduler
```

### Prometheus Metrics
Metrics are exposed on `/metrics` (API on :8080, Scheduler on :9090):
- `scheduler.ticks` — Scheduler loop iterations
- `scheduler.tasks_enqueued` — Tasks sent to workers
- `scheduler.task_retries` — Retry events
- `scheduler.runs_completed` — DAG runs finished
- `worker.tasks_processed` — Tasks executed by workers
- `worker.tasks_succeeded` / `worker.tasks_failed`
- `api.dags_submitted` / `api.runs_triggered`

## Cloud Deployment

### Kubernetes

```bash
kubectl apply -f infra/k8s/flowforge.yaml
```

The manifest creates:
- Namespace `flowforge`
- PostgreSQL StatefulSet with PVC
- Redis Deployment
- API Deployment (2 replicas) with LoadBalancer Service
- Scheduler Deployment (1 replica)
- Worker Deployment (3 replicas)
- Secrets for database credentials

### Docker-based VM

```bash
# On the target server
git clone <repo>
cd Workflow_Scheduler
docker compose up -d

# Verify
curl http://localhost:8080/health
```

## Testing

```bash
# Run all unit tests
cargo test --workspace

# Run specific crate tests
cargo test -p flowforge-common
cargo test -p flowforge-worker
```

Test coverage:
- **DAG validation**: cycle detection, missing dependencies, duplicates, topological ordering
- **Task execution**: successful commands, failed commands, timeout handling
- **Ready-task resolution**: dependency graph traversal

## License

MIT
