# FlowForge Architecture

FlowForge is a production-grade, distributed workload orchestration platform built with Rust, PostgreSQL, NATS JetStream, and a modern TypeScript operator console.

---

## 1. High-Level Architecture Diagram

```mermaid
flowchart TD
    subgraph UI ["Operator Console (TypeScript / React)"]
        A[Web Dashboard]
        CLI[FlowForge CLI]
    end

    subgraph Gateway ["Control Plane API (Axum 0.7)"]
        API[FlowForge API Gateway]
        AUTH[RBAC & Auth Middleware]
        SSE[Live SSE Event Stream]
    end

    subgraph Storage ["Durable Source of Truth"]
        PG[(PostgreSQL 18+)]
        OUTBOX[(Outbox Queue)]
    end

    subgraph Core ["Distributed Control Plane"]
        SCHED1[Scheduler Leader]
        SCHED2[Scheduler Standby]
        ELECT[Lease Leader Election & Fencing]
        PUB[Outbox Message Publisher]
    end

    subgraph Messaging ["Messaging Backbone"]
        NATS[NATS JetStream]
    end

    subgraph DataPlane ["Worker Execution Fleet"]
        W1[Worker 1 - Linux x86]
        W2[Worker 2 - Linux ARM]
        W3[Worker 3 - Kubernetes]
        EXE[Task Executors: Shell, Container, HTTP, Script]
    end

    A --> API
    CLI --> API
    API --> AUTH
    AUTH --> PG
    API --> SSE

    SCHED1 <--> ELECT
    SCHED2 <--> ELECT
    ELECT <--> PG

    PG --> OUTBOX
    OUTBOX --> PUB
    PUB --> NATS

    NATS --> W1
    NATS --> W2
    NATS --> W3

    W1 --> EXE
    W2 --> EXE
    W3 --> EXE

    W1 -- Heartbeats & Leases --> PG
    W2 -- Heartbeats & Leases --> PG
    W3 -- Heartbeats & Leases --> PG
```

---

## 2. Core Architectural Invariants

1. **PostgreSQL as Single Source of Truth**: The database stores canonical workflow state machines, immutable workflow definitions, run history, and worker registrations.
2. **Transactional Outbox Pattern**: State transitions and outgoing message dispatches occur in the same database transaction, eliminating dual-write inconsistencies.
3. **Application-Level Idempotency**: All workflow runs and task execution messages contain unique execution keys (`idempotency_key`, `task_run_id`, `attempt_id`).
4. **HA Scheduler with Distributed Leases**: Schedulers run as active-passive clusters with PostgreSQL lease-based leader election and monotonic fencing tokens (`lease_version`).
5. **Durable Worker Leases**: Every executing task is protected by a renewable 30-second lease. If a worker crashes, the lease expires and the task is re-queued according to its retry policy.
