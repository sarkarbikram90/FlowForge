# FlowForge Persistence & Data Model

## 1. Relational Schema Architecture

FlowForge uses PostgreSQL 16+ with SQLx asynchronous pooling. The database is organized into 6 versioned migration sets:

1. **Identity & Tenancy**: `organizations`, `projects`, `users`, `roles`, `permissions`, `user_roles`, `service_accounts`, `api_keys`.
2. **Workflows & Blueprints**: `workflows`, `workflow_versions`, `workflow_tasks`, `workflow_triggers`, `workflow_variables`.
3. **Executions & Leases**: `workflow_runs`, `task_runs`, `task_attempts`, `task_leases`, `worker_registrations`, `scheduler_leases`.
4. **Reliability & Outbox**: `outbox_messages`, `execution_events`, `dead_letter_tasks`, `schedules`.
5. **Security & Notifications**: `audit_logs`, `secret_references`, `notification_channels`, `notification_rules`, `notification_deliveries`.
6. **Usage & Quotas**: `quotas`, `usage_records`.

---

## 2. Idempotency and Locking Semantics

- Every workflow run insertion verifies uniqueness on `(project_id, idempotency_key)` if specified.
- Task leases use optimistic locking with monotonic versions to prevent simultaneous execution across workers.
