# FlowForge Messaging & Transactional Outbox

## 1. Multi-Tenant Subject Routing

FlowForge structures NATS JetStream subjects hierarchically for strict multi-tenant isolation:

- **Task Dispatch**: `flowforge.tasks.dispatch.{org_id}.{project_id}.{task_type}`
- **Task Completion**: `flowforge.tasks.complete.{org_id}.{project_id}`
- **Workflow Events**: `flowforge.events.workflow.{org_id}.{project_id}.{workflow_id}`
- **Worker Heartbeats**: `flowforge.system.workers.{worker_id}.heartbeat`

---

## 2. Transactional Outbox Pattern

To eliminate dual-write inconsistencies between PostgreSQL and NATS:
1. When scheduler progresses a DAG or API triggers a run, state changes and outgoing messages are written in a single ACID PostgreSQL transaction to `outbox_messages`.
2. A background `OutboxPublisher` polls pending records, publishes them to NATS JetStream with at-least-once delivery guarantees, and marks records as published.
