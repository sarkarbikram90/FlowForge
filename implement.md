# FlowForge — Distributed Workflow Scheduler

## Architecture Overview

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
│              │  (Core Brain)   │                                  │
│              │  - DAG Parser   │                                  │
│              │  - Dependency   │                                  │
│              │    Resolver     │                                  │
│              │  - Cron Ticker  │                                  │
│              └────────┬────────┘                                  │
│                       │                                          │
│              ┌────────▼────────┐                                 │
│              │   Redis Queue   │                                  │
│              │  (Task Queue)   │                                  │
│              └────────┬────────┘                                  │
│                       │                                          │
│         ┌─────────────┼─────────────┐                            │
│         ▼             ▼             ▼                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                       │
│  │ Worker 1 │  │ Worker 2 │  │ Worker N │                       │
│  │(Executor)│  │(Executor)│  │(Executor)│                       │
│  └──────┬───┘  └──────┬───┘  └──────┬───┘                       │
│         └─────────────┼─────────────┘                            │
│                       ▼                                          │
│              ┌─────────────────┐                                 │
│              │   PostgreSQL    │                                  │
│              │ (Metadata Store)│                                  │
│              └─────────────────┘                                 │
└──────────────────────────────────────────────────────────────────┘
```

## Data Flow

1. **DAG Submission**: User submits YAML DAG via CLI/API → stored in PostgreSQL
2. **Scheduling**: Scheduler reads DAGs, resolves dependencies, enqueues ready tasks to Redis
3. **Execution**: Workers dequeue tasks, execute shell commands, report results
4. **State**: All state transitions persisted in PostgreSQL with idempotency keys
5. **Observability**: Structured tracing throughout, Prometheus metrics exposed on `/metrics`

## Component Responsibilities

| Component | Crate | Responsibility |
|-----------|-------|---------------|
| **common** | `flowforge-common` | Shared types, DB models, Redis client, error types |
| **scheduler** | `flowforge-scheduler` | DAG parsing, dependency resolution, task enqueuing, cron |
| **worker** | `flowforge-worker` | Task dequeuing, shell execution, retry/backoff, heartbeat |
| **api** | `flowforge-api` | REST endpoints (Axum), DAG CRUD, run triggers, status |
| **cli** | `flowforge-cli` | Command-line interface wrapping API calls |
| **ui** | `ui/` | React SPA — DAG visualization, run status |
| **infra** | `infra/` | Docker Compose, Dockerfiles, Kubernetes manifests |

## Implementation Phases

### Phase 1: Foundation (common + scheduler)
- Cargo workspace setup
- Shared types & DB schema (SQLx migrations)
- DAG YAML parser & validator
- Dependency graph (topological sort)

### Phase 2: Execution (worker + queue)
- Redis task queue abstraction
- Worker loop: dequeue → execute → report
- Retry with exponential backoff
- Worker heartbeat & crash recovery

### Phase 3: API + CLI
- Axum REST API (DAG submit, trigger, status)
- CLI using clap wrapping API
- Prometheus metrics endpoint

### Phase 4: UI + Infra
- React DAG visualization
- Docker Compose (full stack)
- Example DAGs & end-to-end test

### Phase 5: Polish
- Integration tests
- Documentation (README)
- Cloud deployment guide

## Key Design Decisions

- **SQLx** for compile-time checked SQL (with runtime fallback for flexibility)
- **Redis BRPOPLPUSH** for reliable task dequeuing with visibility timeout
- **Topological sort** for dependency resolution — cycle detection built-in
- **Idempotency** via unique `(run_id, task_id)` composite keys
- **Exponential backoff**: `delay = base_delay * 2^attempt` capped at max_delay
- **Worker heartbeat**: workers send heartbeat every 30s; scheduler requeues tasks from dead workers after 90s
