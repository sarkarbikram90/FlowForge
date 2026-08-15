<div align="center">
  <img src="assets/logo.svg" alt="FlowForge Logo" width="180" height="180" />
  <h1>⚡ FlowForge</h1>
  <p><strong>Cloud-Native, Rust-Powered Distributed Workload Orchestration Platform</strong></p>
  <p>Scheduled, event-driven, observable and fault-tolerant workflow automation across containers, Kubernetes, servers, scripts and APIs.</p>
</div>

---

## 🌟 Key Highlights

- **Rust-Native Core**: High-throughput async control plane built with Tokio, Axum, SQLx, and Petgraph.
- **Transactional Outbox & NATS JetStream**: Zero lost messages and guaranteed crash consistency using the transactional outbox pattern.
- **High-Availability Distributed Scheduler**: Active-passive scheduler cluster with distributed PostgreSQL leader election and monotonic fencing tokens.
- **State Machine Enforcement**: Formal workflow and task state transitions preventing invalid terminal mutations or split-brain executions.
- **Durable Worker Leases & Auto-Recovery**: Tasks are bound to renewable leases. Stale or crashed workers are automatically detected, tasks transitioned to `LOST`, and requeued according to configurable exponential backoff with jitter.
- **Modern Operator Dashboard**: High-performance dark-mode TypeScript console featuring live DAG visualization, Gantt execution timelines, streaming terminal logs, and fleet telemetry.
- **Production Hardened**: Native Kubernetes Helm charts with PodDisruptionBudgets, NetworkPolicies, and horizontal autoscaling.

---

## 🏗️ Architecture

```mermaid
flowchart TD
    subgraph UI ["Operator Console (React + TypeScript)"]
        DASH[Web Dashboard]
        CLI[FlowForge CLI]
    end

    subgraph API ["Control Plane Gateway (Axum)"]
        GW[REST API v1]
        SSE[Live SSE Stream]
    end

    subgraph DB ["Authoritative Storage"]
        PG[(PostgreSQL 18+)]
        OUTBOX[(Outbox Queue)]
    end

    subgraph ControlPlane ["HA Scheduler Cluster"]
        SCHED[Scheduler Leader]
        LEAD[Leader Election & Fencing]
        PUB[Outbox Message Publisher]
    end

    subgraph Messaging ["Messaging Backbone"]
        NATS[NATS JetStream]
    end

    subgraph Workers ["Distributed Worker Fleet"]
        W1[Worker 1 - Host Shell]
        W2[Worker 2 - Docker Container]
        W3[Worker 3 - HTTP / Scripts]
    end

    DASH --> GW
    CLI --> GW
    GW --> PG
    GW --> SSE

    SCHED <--> LEAD
    LEAD <--> PG
    PG --> OUTBOX
    OUTBOX --> PUB
    PUB --> NATS

    NATS --> W1
    NATS --> W2
    NATS --> W3

    W1 -- Leases & Heartbeats --> PG
    W2 -- Leases & Heartbeats --> PG
    W3 -- Leases & Heartbeats --> PG
```

---

## 📦 Workspace Crates

```text
crates/
├── common/             # Domain models, state machines, retry backoff with jitter
├── workflow-engine/    # DAG validation, cycle detection, critical path, compiler
├── execution-engine/   # TaskExecutor trait (Shell, Container, HTTP, Script, Wait)
├── persistence/        # SQLx migrations, DB repositories, outbox, leases
├── messaging/          # NATS JetStream messaging, pull consumers, outbox publisher
├── auth/               # Multi-tenancy, RBAC roles & permissions, API keys
├── observability/      # OpenTelemetry tracing, Prometheus metrics, structured logs
├── scheduler/          # HA leader election, cron trigger engine, stale lease reaper
├── worker/             # Distributed worker agent, heartbeat, task pull loop, draining
├── api/                # Axum REST API v1, SSE live stream, OpenAPI 3.1
├── cli/                # Clap-based command line interface
└── chaos-tests/        # Automated resilience, leader failover & crash test suite
```

---

## 🚀 Quickstart

### 1. Build and Run Tests
```bash
# Run all unit tests and chaos test suites
cargo test --workspace

# Build optimized release binaries
cargo build --release --workspace
```

### 2. Start Local Development Stack
```bash
# Start PostgreSQL, NATS JetStream, MinIO and OTel Collector
make dev-infra

# Run API Gateway locally
make dev-api

# Run HA Scheduler
make dev-scheduler

# Run Worker Agent
make dev-worker

# Run Frontend UI
make dev-ui
```

### 3. Apply Workflow via CLI
```bash
# Apply example workflow
cargo run -p flowforge-cli -- workflow apply --file examples/daily-etl-pipeline.yaml

# Trigger execution run
cargo run -p flowforge-cli -- run trigger daily-etl-pipeline

# View platform status
cargo run -p flowforge-cli -- status
```

---

## 🛡️ Production Deployment (Kubernetes / Helm)

```bash
helm install flowforge ./deploy/helm/flowforge
```

---

## 📜 License

MIT License.
