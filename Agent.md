# FlowForge — Production-Grade Workload Orchestration Platform

## Mission

Redesign and rebuild the existing FlowForge project into a **production-grade, market-ready distributed workload orchestration platform**.

Do not treat this as a UI redesign or a simple refactor.

Treat it as a **full platform engineering transformation** from a functional prototype into a system that could reasonably be evaluated as:

* a serious open-source infrastructure project,
* a production platform engineering product,
* a cloud-native workload automation platform,
* a credible alternative in the developer-first space adjacent to Airflow, Temporal, AutoSys, Control-M, ActiveBatch and Redwood.

The goal is not to blindly copy any of those products.

The goal is to build a differentiated product:

> **FlowForge is a cloud-native, Rust-powered workload orchestration platform for reliably scheduling, executing, observing and recovering distributed workloads across containers, Kubernetes, servers, scripts and APIs.**

The final product must demonstrate strong engineering across:

* distributed systems
* workflow orchestration
* fault tolerance
* durable execution
* concurrency
* scheduling
* event-driven architecture
* API design
* security
* multi-tenancy
* observability
* cloud-native deployment
* frontend engineering
* operational excellence

Do not stop at making the happy path work.

The system must explicitly handle crashes, duplicate messages, network failures, worker loss, scheduler failover, database failure, stale leases, partial execution, retries, cancellation, deployment upgrades and recovery.

---

# 1. Existing System Context

The existing FlowForge implementation is a Rust-based DAG workflow scheduler with:

* Rust backend
* Axum API
* Tokio async runtime
* PostgreSQL
* Redis
* distributed workers
* DAG parsing using petgraph
* cron scheduling
* retry with exponential backoff
* worker heartbeats
* REST API
* CLI
* React UI
* Prometheus metrics
* structured logging
* Kubernetes manifests
* Docker Compose
* idempotency based on `(run_id, task_id)`

The existing architecture is approximately:

```text
CLI ──────┐
          ├── REST API ─── Scheduler ─── Redis ─── Workers
UI ───────┘                   │                    │
                             PostgreSQL ◀──────────┘
```

The current design also intentionally uses:

* Redis `BRPOPLPUSH` for task movement
* a single scheduler
* shell command execution
* PostgreSQL metadata
* Kubernetes deployment

These decisions were useful for the prototype but must be reconsidered for a production platform.

Do not preserve an architectural decision merely because it already exists.

Preserve behavior where possible, but redesign internals where necessary.

---

# 2. Non-Negotiable Technology Direction

Use the following architecture unless there is an exceptionally strong engineering reason to deviate.

## Frontend

Use:

* TypeScript
* React
* Vite
* React Router
* TanStack Query
* Zustand only for truly client-side/global UI state
* Tailwind CSS
* accessible component primitives
* a consistent component system
* WebSockets or Server-Sent Events where appropriate for live execution updates
* ECharts or another production-quality charting library
* strict TypeScript
* ESLint
* Prettier
* Vitest
* Playwright

Do not build the UI as a collection of ad-hoc components.

Create a reusable design system for:

* buttons
* forms
* dialogs
* tables
* status badges
* filters
* command palettes
* navigation
* cards
* charts
* timelines
* DAG nodes
* DAG edges
* log viewers
* alerts
* empty states
* loading states
* error states

The UI must look like a serious infrastructure SaaS product, not a developer demo.

---

# 3. Backend

Use:

* Rust stable
* Tokio
* Axum
* SQLx
* Serde
* tracing
* OpenTelemetry
* `opentelemetry-rust`
* structured JSON logging
* Prometheus-compatible metrics
* strong typing throughout
* async-first implementation
* explicit error types
* no unnecessary `unwrap()`
* no unnecessary `expect()`
* graceful shutdown everywhere
* cancellation-aware async tasks

Use a modular workspace.

Recommended structure:

```text
flowforge/
├── Cargo.toml
├── crates/
│   ├── api/
│   ├── scheduler/
│   ├── dispatcher/
│   ├── worker/
│   ├── workflow-engine/
│   ├── execution-engine/
│   ├── event-engine/
│   ├── auth/
│   ├── tenancy/
│   ├── persistence/
│   ├── messaging/
│   ├── artifacts/
│   ├── observability/
│   ├── common/
│   └── cli/
├── frontend/
├── migrations/
├── deploy/
│   ├── docker/
│   ├── helm/
│   └── kubernetes/
├── integration-tests/
├── e2e/
├── examples/
├── docs/
└── scripts/
```

Keep domain logic independent from Axum.

Do not put business logic directly into HTTP handlers.

---

# 4. Primary Architecture

The production architecture should look conceptually like this:

```text
                         ┌──────────────────────┐
                         │     TypeScript UI     │
                         │ React + TanStack      │
                         └──────────┬───────────┘
                                    │
                              HTTPS / WebSocket
                                    │
                         ┌──────────▼───────────┐
                         │      API Gateway      │
                         │       Axum API        │
                         └──────────┬───────────┘
                                    │
                 ┌──────────────────┼──────────────────┐
                 │                  │                  │
                 ▼                  ▼                  ▼
          Workflow Service    Execution Service   Event Service
                 │                  │                  │
                 └──────────────────┼──────────────────┘
                                    │
                      PostgreSQL — Source of Truth
                                    │
                    ┌───────────────┼─────────────────┐
                    │               │                 │
                    ▼               ▼                 ▼
               Scheduler        NATS JetStream     Object Storage
               / Leader         durable events     Logs/Artifacts
                    │               │
                    │        ┌──────┼──────┐
                    │        ▼      ▼      ▼
                    │      Worker Worker Worker
                    │
                    ▼
              Workflow State
```

---

# 5. Database — PostgreSQL

Use PostgreSQL as the **authoritative source of truth**.

Target PostgreSQL 18+ compatible design.

Do not use PostgreSQL merely as a place to dump metadata.

The database must represent the workflow state machine correctly.

Use SQLx migrations.

Do NOT continue using:

```sql
CREATE TABLE IF NOT EXISTS ...
```

as the primary migration mechanism.

Create versioned migrations:

```text
migrations/
├── 0001_initial_schema.sql
├── 0002_tenants.sql
├── 0003_workflows.sql
├── 0004_workflow_versions.sql
├── 0005_runs.sql
├── 0006_tasks.sql
├── 0007_workers.sql
├── 0008_events.sql
├── 0009_audit_logs.sql
└── ...
```

---

# 6. Required Database Model

Design normalized relational tables for at minimum:

## Identity / tenancy

```text
organizations
projects
users
roles
permissions
user_roles
service_accounts
api_keys
```

## Workflow

```text
workflows
workflow_versions
workflow_tasks
workflow_dependencies
workflow_triggers
workflow_variables
```

## Execution

```text
workflow_runs
task_runs
task_attempts
task_dependencies
task_leases
worker_registrations
worker_heartbeats
```

## Reliability

```text
execution_events
dead_letter_tasks
retry_policies
schedules
```

## Security / compliance

```text
audit_logs
secret_references
```

## Notifications

```text
notification_channels
notification_rules
notification_deliveries
```

## Usage

```text
usage_records
quotas
billing_usage
```

Do not create every table blindly.

Use appropriate relational boundaries and explain why each table exists.

---

# 7. Workflow Versioning

A workflow must be immutable once a run starts.

Example:

```text
workflow:
    etl-pipeline

version:
    17
```

A run must reference:

```text
workflow_id
workflow_version_id
```

Never execute an old run against whatever happens to be the latest workflow definition.

This is mandatory for deterministic recovery.

A user updating a workflow creates a new version.

Existing runs continue using the original version.

---

# 8. Workflow Definition

Support YAML and JSON.

Eventually provide a TypeScript SDK, but keep the workflow representation language-neutral.

Example:

```yaml
apiVersion: flowforge.io/v1
kind: Workflow

metadata:
  name: daily-etl
  version: 3

spec:
  schedule:
    cron: "0 * * * *"

  concurrency:
    maxRuns: 2

  retries:
    maxAttempts: 5
    backoff:
      type: exponential
      initial: 5s
      max: 5m

  tasks:

    - id: extract-users
      type: shell
      command: ./extract-users.sh

    - id: extract-orders
      type: shell
      command: ./extract-orders.sh

    - id: transform
      type: container
      image: company/transform:1.4.2
      dependsOn:
        - extract-users
        - extract-orders

    - id: load
      type: container
      image: company/load:2.0.1
      dependsOn:
        - transform
```

Validate workflow definitions before persistence.

Validation must include:

* duplicate task IDs
* missing dependencies
* cycles
* invalid cron
* invalid retry configuration
* invalid timeouts
* unsupported task type
* invalid environment variables
* invalid resource requests
* invalid concurrency settings
* invalid permissions
* invalid references

Return structured validation errors.

---

# 9. Task Model

Define a strong task abstraction.

Initial task types:

```text
shell
container
http
python
kubernetes
docker
script
wait
condition
```

Design the task engine so new task types can be added through a plugin/executor interface.

For example:

```rust
trait TaskExecutor {
    async fn validate(&self, task: &TaskDefinition) -> Result<()>;

    async fn execute(
        &self,
        context: ExecutionContext,
    ) -> Result<TaskExecutionResult>;

    async fn cancel(
        &self,
        context: ExecutionContext,
    ) -> Result<()>;
}
```

Do not couple the scheduler to shell execution.

The scheduler decides **what should happen**.

The executor determines **how it happens**.

---

# 10. Message Bus — Replace Redis Core Queue

Use **NATS JetStream** as the internal messaging and task-dispatch backbone.

Do not use Redis as the durable task queue.

Use NATS JetStream for:

```text
workflow.commands
workflow.events
task.dispatch
task.retry
task.cancel
worker.registration
worker.heartbeat
notifications
```

Use explicit subject naming:

```text
flowforge.<tenant>.<project>.task.dispatch
flowforge.<tenant>.<project>.task.events
flowforge.<tenant>.<project>.workflow.events
```

Design subjects carefully so the product remains multi-tenant.

Use durable consumers.

Prefer pull consumers for horizontally scalable workers.

Each worker should:

1. receive a task
2. establish an execution lease
3. persist execution state
4. execute the task
5. emit progress
6. persist outcome
7. acknowledge the message

Do not assume message delivery equals execution exactly once.

The platform must use **application-level idempotency**.

NATS JetStream can provide durable acknowledged delivery and redelivery, but duplicate delivery is still an application concern.

Therefore every execution must have a unique:

```text
execution_id
attempt_id
task_run_id
```

and the worker must safely handle duplicate dispatch.

---

# 11. PostgreSQL + NATS Consistency

This is critical.

Never perform an unsafe sequence such as:

```text
UPDATE database
publish message
```

without thinking about crash consistency.

Implement an **outbox pattern**.

Example:

```text
Transaction:

BEGIN

insert task_run
insert execution_event
insert outbox_message

COMMIT
```

Then an outbox publisher reads:

```text
outbox_messages
```

and publishes to NATS.

After successful publication:

```text
mark published
```

This prevents the classic:

```text
DB commit succeeds
NATS publish fails
```

failure mode.

Likewise design the worker completion path carefully so the system remains correct if:

```text
task completed
DB update succeeded
ACK lost
message redelivered
```

The second execution must not corrupt state.

---

# 12. Scheduler Architecture

Replace the current single scheduler with a highly available scheduler service.

Run:

```text
scheduler-1
scheduler-2
scheduler-3
```

All scheduler instances may be active but only the elected leader performs operations requiring singleton ownership.

Implement distributed leader election using PostgreSQL-backed leases or another explicitly justified coordination mechanism.

Do not rely on process-local locks.

The scheduler must survive:

* process crash
* node crash
* network disconnect
* Kubernetes rescheduling
* database connection interruption
* rolling deployments

When the leader disappears, another scheduler must acquire the lease.

Design the system so there is no dual-active scheduler.

Every leadership lease should contain:

```text
leader_id
lease_version
acquired_at
expires_at
heartbeat_at
```

Use fencing tokens or an equivalent mechanism to prevent a stale leader from continuing to perform writes after losing leadership.

This is extremely important.

---

# 13. Scheduling Engine

Support:

### Time-based triggers

```text
cron
fixed interval
daily
weekly
one-time
```

### Event-based triggers

```text
webhook
message
file arrival
API event
workflow completion
workflow failure
```

### Dependency triggers

```text
workflow A succeeds
workflow B starts
task X completes
```

### Manual triggers

```text
Run now
Run with parameters
Rerun failed tasks
Resume workflow
```

The scheduler must not execute shell commands.

It should produce executable work units.

---

# 14. Durable Execution State Machine

Define explicit states.

Workflow states:

```text
PENDING
QUEUED
RUNNING
PAUSED
SUCCEEDED
FAILED
CANCELING
CANCELED
TIMED_OUT
RETRYING
SUSPENDED
```

Task states:

```text
PENDING
BLOCKED
READY
DISPATCHED
RUNNING
SUCCEEDED
FAILED
RETRY_WAIT
CANCELED
TIMED_OUT
LOST
DEAD_LETTER
```

Make all transitions explicit.

Do not scatter arbitrary status updates throughout the codebase.

Create a state transition layer:

```rust
transition_task(
    current_state,
    event,
) -> Result<new_state>
```

Reject illegal transitions.

Add transition tests.

---

# 15. Leases

Every running task must have a lease.

Example:

```text
task_run_id
worker_id
attempt_id
lease_token
lease_expires_at
heartbeat_at
```

Workers periodically renew leases.

If a worker disappears:

```text
lease expires
      ↓
controller detects stale task
      ↓
task marked LOST
      ↓
retry policy evaluated
      ↓
task requeued
```

Do not immediately requeue every missed heartbeat.

Account for:

* transient network delay
* worker pauses
* GC pauses in external tasks
* database latency
* overloaded worker
* clock skew

Use server-side timestamps wherever possible.

---

# 16. Retry Semantics

Support:

```text
fixed
linear
exponential
exponential + jitter
```

Example:

```text
attempt 1 → 5s
attempt 2 → 10s
attempt 3 → 20s
attempt 4 → 40s
attempt 5 → 80s
```

Apply jitter to reduce synchronized retries.

Allow:

```yaml
retry:
  maxAttempts: 5
  backoff:
    type: exponential
    initial: 5s
    max: 5m
    jitter: 20%
```

Differentiate:

```text
retryable failure
non-retryable failure
timeout
canceled
infrastructure failure
application failure
```

Allow task-level retry policy to override workflow-level defaults.

---

# 17. Dead Letter Handling

Never allow permanently failing jobs to disappear.

Implement a dead-letter subsystem.

When maximum attempts are exhausted:

```text
task
  ↓
DEAD_LETTER
  ↓
DLQ
```

UI must expose:

* failure reason
* all attempts
* timestamps
* worker IDs
* logs
* retry history
* stack/error information
* input parameters
* workflow version

Allow:

```text
Retry
Retry from this task
Retry entire workflow
Clone run
Ignore task
Resume downstream
```

with appropriate permissions and audit logging.

---

# 18. Cancellation

Cancellation must be first-class.

Support:

```text
Cancel workflow
Cancel task
Cancel pending work
Cancel running work
Force cancel
```

Cancellation should propagate:

```text
Workflow
   ↓
Task Run
   ↓
Worker
   ↓
Executor
   ↓
Process / Container / Kubernetes Job
```

Workers must gracefully terminate child processes.

Respect termination signals.

Implement cancellation timeouts.

If graceful cancellation fails:

```text
TERM
 ↓
grace period
 ↓
KILL
```

Do not leave orphan processes.

---

# 19. Worker Architecture

Workers should be stateless.

Worker responsibilities:

```text
register
heartbeat
pull work
execute
stream progress
publish result
acknowledge
```

Worker registration must include:

```text
worker_id
version
hostname
os
architecture
capabilities
labels
max_concurrency
current_load
```

Example:

```text
worker:
  type: kubernetes
  region: us-east
  environment: production
  capabilities:
    - docker
    - python
    - kubectl
```

Support worker routing by labels.

For example:

```yaml
requirements:
  workerLabels:
    environment: production
    region: us-east
```

---

# 20. Worker Concurrency

Implement:

```text
global concurrency
organization concurrency
project concurrency
workflow concurrency
worker concurrency
task-type concurrency
```

Example:

```text
organization:
    max_concurrent_tasks = 1000

workflow:
    max_concurrent_runs = 10

worker:
    max_concurrent_tasks = 20
```

Use backpressure.

Never allow an uncontrolled queue explosion.

Expose queue depth and saturation metrics.

---

# 21. Backpressure

The platform must behave predictably under load.

When workers are saturated:

```text
scheduler
    ↓
READY
    ↓
capacity unavailable
    ↓
remain queued
```

Do not endlessly publish tasks into a queue without considering worker capacity.

Implement:

* queue depth limits
* tenant quotas
* rate limits
* concurrency limits
* admission control
* overload protection

---

# 22. Security Model

Security is mandatory for market readiness.

Implement authentication through OIDC/OAuth2.

Support providers such as:

```text
Google
Microsoft Entra ID
GitHub
Keycloak
generic OIDC
```

Do not implement your own password authentication unless absolutely required.

Provide local Keycloak in the development environment for testing.

Implement:

```text
organizations
projects
roles
permissions
```

RBAC example:

```text
Platform Admin
Organization Admin
Project Admin
Workflow Editor
Workflow Operator
Viewer
Auditor
```

Permissions should cover:

```text
workflow:create
workflow:update
workflow:delete
workflow:execute
workflow:cancel
workflow:read
worker:read
worker:admin
secret:read
audit:read
organization:admin
```

Enforce authorization on the backend.

Never trust frontend authorization.

---

# 23. Multi-Tenancy

Build tenancy from the beginning.

Every relevant record should contain:

```text
organization_id
project_id
```

Never rely on frontend filtering to isolate tenants.

Enforce tenant boundaries at the API and persistence layers.

Use PostgreSQL row-level security only where it genuinely improves safety and maintainability; otherwise enforce strong repository/service-layer scoping.

Add tests for cross-tenant access.

Example:

```text
Tenant A
   ↓
workflow A
   ↓
run A

Tenant B
   ↓
workflow B
   ↓
run B
```

Tenant A must never be able to retrieve Tenant B data even by guessing UUIDs.

---

# 24. Secrets

Never store secrets as plain workflow YAML.

Create secret references:

```yaml
secrets:
  - DATABASE_PASSWORD
  - API_TOKEN
```

Workers receive only the secrets required by the task.

Prefer integration with an external secret manager:

```text
HashiCorp Vault
AWS Secrets Manager
GCP Secret Manager
Azure Key Vault
Kubernetes Secrets
```

Create a secrets abstraction layer so the backend is not tied to one provider.

Never log secret values.

Add automatic secret redaction to logs.

---

# 25. Task Isolation

Shell execution is dangerous.

Do not treat arbitrary shell execution as trusted.

Provide execution modes:

```text
host
container
kubernetes
sandbox
```

Default production execution should be isolated.

Implement configurable:

```text
CPU
memory
disk
network access
timeout
process count
filesystem
environment
```

For Kubernetes execution:

* separate namespaces where appropriate
* service accounts
* Pod Security standards
* NetworkPolicies
* resource requests/limits
* security contexts
* non-root execution where possible
* read-only filesystems where practical
* dropped Linux capabilities

Never execute arbitrary customer code in the API process.

---

# 26. Object Storage

Use S3-compatible object storage for:

* task logs
* workflow artifacts
* execution artifacts
* exports
* large event payloads
* archived execution history

Support:

```text
AWS S3
MinIO
GCS-compatible adapter
Azure Blob adapter
```

Do not put huge logs into PostgreSQL.

PostgreSQL stores:

```text
metadata
index
location
checksum
size
retention
```

Object storage holds:

```text
content
```

Implement configurable retention.

---

# 27. Logging

Every component must use structured logs.

Example:

```json
{
  "timestamp": "...",
  "level": "INFO",
  "service": "worker",
  "tenant_id": "...",
  "project_id": "...",
  "workflow_id": "...",
  "run_id": "...",
  "task_run_id": "...",
  "attempt_id": "...",
  "worker_id": "...",
  "event": "task_completed",
  "duration_ms": 4821
}
```

Use correlation IDs throughout.

A user should be able to go from:

```text
Workflow Run
    ↓
Task
    ↓
Attempt
    ↓
Trace
    ↓
Logs
    ↓
Worker
```

without losing context.

---

# 28. Observability

Instrument the complete platform with OpenTelemetry.

Produce:

```text
traces
metrics
logs
```

Use OTLP.

Instrument:

```text
API requests
scheduler decisions
database calls
NATS publish
NATS consume
workflow execution
task dispatch
worker execution
retries
timeouts
cancellation
external API calls
```

Create metrics such as:

```text
flowforge_workflows_total
flowforge_workflow_runs_total
flowforge_workflow_run_duration_seconds
flowforge_tasks_total
flowforge_task_execution_duration_seconds
flowforge_task_failures_total
flowforge_task_retries_total
flowforge_task_timeouts_total
flowforge_task_lost_total
flowforge_queue_depth
flowforge_worker_capacity
flowforge_worker_utilization
flowforge_scheduler_leader_changes_total
flowforge_nats_publish_failures_total
flowforge_db_transaction_failures_total
```

Make metric labels cardinality-safe.

Do not put arbitrary workflow input into metric labels.

OpenTelemetry Collector should be supported as the standard telemetry gateway.

---

# 29. UI — Product-Level Dashboard

The frontend must be completely redesigned around an operator's workflow.

Primary navigation:

```text
Overview
Workflows
Runs
Workers
Queues
Events
Schedules
Connections
Secrets
Alerts
Audit
Usage
Settings
```

The dashboard should immediately answer:

```text
Is the platform healthy?

What is running?

What is failing?

What is delayed?

Which workflows are at risk?

Are workers overloaded?

Are queues building?

Are scheduled workloads missing SLAs?

What changed recently?
```

---

# 30. Executive Dashboard

Create a serious operations dashboard.

Top-level KPIs:

```text
Running
Succeeded
Failed
Queued
Delayed
Success Rate
Average Duration
SLA Compliance
Active Workers
Queue Depth
```

Charts:

```text
Workflow success rate
Execution volume
Latency
Failure trend
Retry trend
Worker utilization
Queue depth
SLA breaches
```

Provide configurable time range:

```text
15m
1h
6h
24h
7d
30d
custom
```

---

# 31. Workflow Explorer

Workflow page must show:

```text
workflow name
description
version
owner
schedule
status
last run
next run
success rate
average duration
SLA
dependencies
```

DAG visualization should support:

* zoom
* pan
* search
* task selection
* status coloring
* dependency highlighting
* critical-path highlighting
* failed task highlighting
* running animation
* retry indicator
* queue duration
* execution duration

Clicking a task should open:

```text
Task configuration
Dependencies
Current state
Last execution
Retries
Logs
Metrics
Artifacts
Worker
Execution history
```

---

# 32. Run Detail Page

This must be one of the strongest pages in the product.

Display:

```text
Run ID
Workflow
Version
Triggered by
Started
Ended
Duration
Status
```

Then:

```text
DAG
Timeline
Task list
Event stream
Logs
Artifacts
Trace
```

Timeline must show:

```text
queued
dispatched
started
heartbeat
retry
completed
```

Allow filtering.

Provide a critical-path analysis.

Example:

```text
extract-users      4s
extract-orders     3s
transform         42s   ← critical
load              12s
```

---

# 33. Live Execution

The UI must receive live updates.

Do not require users to refresh.

Use:

```text
WebSocket
```

or:

```text
Server-Sent Events
```

for execution updates.

Show:

```text
task started
task progress
task retrying
worker lost
task completed
task failed
workflow completed
```

Make reconnection automatic.

The client must recover state after reconnect.

---

# 34. Logs UI

Build a production-quality log viewer.

Features:

```text
live tail
pause
resume
search
regex
level filter
time filter
download
copy
context
```

Show correlation:

```text
workflow
run
task
attempt
worker
trace
```

Logs should be streamed to object storage for durable retention.

---

# 35. Worker UI

Workers page:

```text
Worker ID
Status
Version
Hostname
Region
Labels
Capacity
Utilization
Current tasks
Heartbeat
Last seen
```

Statuses:

```text
ONLINE
DEGRADED
DRAINING
OFFLINE
LOST
```

Allow:

```text
Drain worker
Restart worker
Inspect worker
View active tasks
```

Draining means:

```text
stop accepting new tasks
finish current tasks
then exit
```

This is required for safe rolling deployments.

---

# 36. Queue UI

Expose:

```text
queue name
depth
oldest message
consumers
throughput
retry rate
DLQ count
```

Make it obvious when the platform is experiencing backpressure.

---

# 37. Event Engine

Build an event abstraction.

Events may originate from:

```text
HTTP webhook
NATS
workflow completion
task completion
schedule
file arrival
manual trigger
external systems
```

Example:

```yaml
trigger:
  type: webhook

  path: /hooks/orders
```

Then:

```text
Webhook
   ↓
Event Validation
   ↓
Event Store
   ↓
Event Router
   ↓
Workflow Trigger
   ↓
Workflow Run
```

Store enough information to audit why a workflow was started.

---

# 38. Notifications

Support:

```text
Email
Slack
Microsoft Teams
PagerDuty
Webhook
```

Notification rules:

```text
workflow failed
workflow SLA breached
task failed
worker lost
queue threshold exceeded
DLQ threshold exceeded
scheduler leadership changed
```

Implement notification deduplication.

Do not spam users during repeated failures.

---

# 39. SLA Management

Add first-class SLA support.

Example:

```yaml
sla:
  completionTime: "30m"
  severity: high
```

Track:

```text
SLA met
SLA at risk
SLA breached
```

Predict potential breaches based on:

```text
elapsed runtime
historical duration
queue delay
worker capacity
dependencies
```

For the first production version, deterministic heuristics are sufficient.

Do not add fake AI.

---

# 40. Audit Logging

Every security-sensitive operation must create an audit record.

Examples:

```text
workflow created
workflow updated
workflow deleted
workflow executed
workflow canceled
secret created
secret changed
user added
role changed
worker drained
API key created
API key revoked
```

Audit record:

```text
timestamp
actor
organization
project
action
resource
resource_id
IP
user_agent
result
metadata
```

Audit logs must be immutable from the normal application path.

---

# 41. API Design

Design a versioned REST API:

```text
/api/v1/
```

Resources:

```text
organizations
projects
workflows
workflow-versions
workflow-runs
task-runs
workers
queues
events
schedules
connections
secrets
alerts
audit
usage
```

Use consistent:

```text
pagination
filtering
sorting
error schema
request IDs
idempotency keys
```

Example error:

```json
{
  "error": {
    "code": "WORKFLOW_VALIDATION_FAILED",
    "message": "Workflow contains a dependency cycle",
    "request_id": "..."
  }
}
```

Publish OpenAPI specification.

Generate TypeScript API types from OpenAPI rather than duplicating types manually.

---

# 42. Idempotent APIs

Critical mutation endpoints should support idempotency.

Examples:

```text
POST /workflow-runs
POST /workflows
POST /events
POST /notifications
```

Allow:

```http
Idempotency-Key: <uuid>
```

Persist idempotency keys and resulting resource references.

A client retry must not accidentally create two workflow runs.

---

# 43. CLI

The CLI should become a first-class product surface.

Example:

```bash
flowforge auth login

flowforge workflow list
flowforge workflow validate workflow.yaml
flowforge workflow apply workflow.yaml

flowforge run create my-workflow
flowforge run list
flowforge run get <id>
flowforge run cancel <id>

flowforge worker list
flowforge worker drain <worker-id>

flowforge logs <run-id>
```

CLI should use the public API.

Do not create a second business-logic implementation in the CLI.

---

# 44. TypeScript SDK

Create an official TypeScript SDK.

Example:

```typescript
import { FlowForge } from "@flowforge/sdk";

const ff = new FlowForge({
  endpoint: process.env.FLOWFORGE_URL,
  token: process.env.FLOWFORGE_TOKEN
});

await ff.workflows.run("daily-etl");
```

Support:

```text
workflows
runs
tasks
workers
events
artifacts
```

Publish the package cleanly.

---

# 45. Rust SDK / Client

Provide a Rust client crate eventually:

```text
flowforge-client
```

but do not let SDK design compromise the public API.

---

# 46. Kubernetes Deployment

Create proper Helm charts.

Do not ship only raw manifests.

Support:

```text
helm install flowforge ./deploy/helm/flowforge
```

Components:

```text
api
scheduler
dispatcher
workers
event-engine
```

External infrastructure:

```text
PostgreSQL
NATS
Object Storage
OIDC Provider
OpenTelemetry Collector
```

Support both:

```text
single-node development
production HA
```

---

# 47. Kubernetes Production Hardening

Every deployment should include:

```text
readinessProbe
livenessProbe
startupProbe
resources.requests
resources.limits
podDisruptionBudget
anti-affinity
topology spread
securityContext
serviceAccount
networkPolicy
```

Implement graceful termination.

Workers must enter:

```text
DRAINING
```

before process termination.

API instances must stop accepting new work appropriately before shutdown.

Kubernetes supports graceful pod termination; design the application lifecycle around that rather than abruptly killing workers.

---

# 48. High Availability Targets

Design for:

```text
API:
    3 replicas

Scheduler:
    3 replicas
    1 leader

Dispatcher:
    2+ replicas

Worker:
    horizontally scalable

NATS:
    3-node cluster

PostgreSQL:
    HA configuration

Object Storage:
    external durable storage
```

The platform must continue operating through:

```text
API pod failure
scheduler pod failure
worker failure
NATS node failure
database failover
network interruption
Kubernetes rescheduling
```

Where an external service is responsible for HA, document that explicitly.

Do not pretend local Docker Compose provides production HA.

---

# 49. Failure Injection Testing

Create automated resilience tests.

Simulate:

```text
kill scheduler
kill worker
kill API
pause worker
disconnect worker
restart NATS node
restart PostgreSQL
network delay
network partition
duplicate message
lost acknowledgment
slow database
slow worker
```

Verify:

```text
no lost task
no silently stuck workflow
no duplicate terminal completion
no corruption of workflow state
proper retry
proper recovery
```

Build a test suite named:

```text
chaos-tests
```

---

# 50. Exactly-Once Semantics

Do not falsely advertise:

> exactly-once execution

unless it can actually be proven.

The system should instead clearly distinguish:

```text
durable message delivery
at-least-once task dispatch
idempotent task state transitions
deduplicated workflow completion
```

Document the distinction between:

```text
exactly-once message processing
```

and:

```text
exactly-once external side effects
```

For external side effects, provide idempotency mechanisms.

---

# 51. Concurrency and Transactions

Use PostgreSQL transactions for state transitions.

Use row-level locking carefully.

Avoid holding DB transactions open while running external tasks.

Never do:

```text
BEGIN
run shell command for 10 minutes
COMMIT
```

Instead:

```text
transaction
  → reserve / update state
commit

execute externally

transaction
  → persist result
commit
```

---

# 52. Performance Requirements

Target meaningful initial production benchmarks.

Define benchmark scenarios:

### Scenario A

```text
10,000 workflows
```

### Scenario B

```text
100,000 tasks/hour
```

### Scenario C

```text
1,000 concurrent task executions
```

### Scenario D

```text
worker failures under load
```

Measure:

```text
scheduler latency
dispatch latency
queue latency
execution latency
database latency
NATS latency
API p50
API p95
API p99
```

Do not invent benchmark results.

Generate real benchmark reports.

---

# 53. Load Testing

Use a dedicated load-testing tool.

Simulate:

```text
workflow creation
workflow scheduling
workflow execution
task completion
worker churn
large DAGs
high-frequency schedules
```

Document:

```text
throughput
latency
resource utilization
bottlenecks
```

---

# 54. Large DAG Support

Test DAGs with:

```text
10 tasks
100 tasks
1,000 tasks
10,000 tasks
```

Optimize DAG validation and execution planning.

Avoid repeatedly recomputing large graphs unnecessarily.

Cache compiled workflow plans where safe.

Invalidate cached plans when workflow version changes.

---

# 55. Scheduler Correctness

Write formal-ish invariants and enforce them in tests.

Examples:

1. A task cannot become RUNNING without being assigned.
2. A task cannot be terminal and RUNNING simultaneously.
3. A workflow cannot succeed while required tasks remain incomplete.
4. A task can have only one authoritative terminal outcome.
5. A stale worker cannot finalize another worker's current attempt.
6. An old scheduler leader cannot perform privileged scheduling operations after lease loss.
7. A workflow run always references an immutable workflow version.
8. Retried execution always creates a new attempt.
9. A task cancellation cannot silently become success.
10. A terminal workflow cannot return to RUNNING.

---

# 56. Security Testing

Add:

```text
unit tests
integration tests
API authorization tests
tenant-isolation tests
dependency scanning
container scanning
SAST
DAST
secret scanning
```

Run these in CI.

Reject builds with high-severity known vulnerabilities unless explicitly documented and approved.

---

# 57. CI/CD

Use GitHub Actions.

Pipeline:

```text
format
lint
unit tests
integration tests
frontend tests
build
security scan
container build
container scan
migration validation
E2E tests
Helm lint
Helm template
```

For protected branches require:

```text
all tests passing
review
security scan
```

Build immutable versioned images.

Example:

```text
ghcr.io/flowforge/api:<git-sha>
```

Do not deploy `latest` in production.

---

# 58. Release Engineering

Use semantic versioning.

```text
MAJOR.MINOR.PATCH
```

Generate:

```text
CHANGELOG
release notes
migration notes
upgrade notes
```

Document breaking API/database changes.

---

# 59. Configuration

Never hardcode environment-specific settings.

Provide:

```text
FLOWFORGE_DATABASE_URL
FLOWFORGE_NATS_URL
FLOWFORGE_OBJECT_STORAGE_ENDPOINT
FLOWFORGE_OBJECT_STORAGE_BUCKET
FLOWFORGE_OTEL_ENDPOINT
FLOWFORGE_OIDC_ISSUER
```

Separate:

```text
development
test
staging
production
```

Configuration must be validated during startup.

Fail fast when required configuration is invalid.

---

# 60. Local Development

Provide a one-command environment:

```bash
make dev
```

Start:

```text
PostgreSQL
NATS JetStream
MinIO
Keycloak
OpenTelemetry Collector
FlowForge API
Scheduler
Dispatcher
Worker
Frontend
```

Provide seeded development data.

Example:

```bash
make seed
```

should produce:

```text
demo organization
demo project
demo workflows
demo users
demo workers
sample executions
sample failures
```

Make the UI immediately useful after startup.

---

# 61. Production Docker Images

Use multi-stage builds.

Backend:

```text
builder
    ↓
minimal runtime
```

Prefer distroless/minimal runtime images where practical.

Do not run as root unless required.

Frontend should produce a static optimized bundle served through a proper web server/CDN.

---

# 62. Documentation

Create serious documentation.

```text
docs/
├── architecture.md
├── concepts.md
├── workflow-model.md
├── execution-model.md
├── scheduler.md
├── worker.md
├── messaging.md
├── persistence.md
├── failure-recovery.md
├── security.md
├── multi-tenancy.md
├── observability.md
├── api.md
├── cli.md
├── sdk.md
├── kubernetes.md
├── operations.md
├── troubleshooting.md
├── performance.md
├── disaster-recovery.md
└── contributing.md
```

Include architecture diagrams using Mermaid.

---

# 63. Product Positioning

Do not position FlowForge as:

> "yet another Airflow clone."

Position it as:

> **Cloud-native workload orchestration for engineering and infrastructure teams.**

Core differentiation:

```text
Rust
↓
lightweight control plane
↓
high concurrency
↓
strong reliability
↓
cloud-native execution
↓
event-driven automation
↓
excellent observability
↓
developer-first API/CLI
↓
operator-first UI
```

The system should be able to manage workloads such as:

```text
Kubernetes jobs
Docker workloads
CI/CD steps
infrastructure automation
ETL
data processing
API workflows
scheduled scripts
ML pipelines
batch processing
operational automation
incident remediation
```

---

# 64. Product Editions

Design with future product tiers in mind.

### Community

```text
single organization
core workflows
workers
CLI
API
basic UI
```

### Team

```text
multiple projects
RBAC
SSO
audit
notifications
advanced observability
```

### Enterprise

```text
advanced RBAC
SCIM
SAML/OIDC
enterprise audit
HA
advanced governance
multi-region
custom retention
policy controls
```

Do not implement billing prematurely.

Implement domain structures so these capabilities can be introduced without rebuilding the core architecture.

---

# 65. Future AI Layer

Do not contaminate the deterministic execution engine with AI.

Keep AI as an optional control-plane intelligence layer.

Potential future capabilities:

```text
failure summarization
root-cause suggestions
SLA breach prediction
workflow optimization
anomaly detection
retry recommendations
incident correlation
natural-language workflow creation
```

AI must never bypass authorization or directly execute destructive operations without explicit policy.

Architecture:

```text
Deterministic Core
        ↑
Policy Engine
        ↑
AI Assistant
```

not:

```text
LLM
 ↓
randomly executes production commands
```

---

# 66. Frontend UX Requirements

The UI should feel closer to:

```text
Datadog
Grafana
Vercel
Linear
GitHub
Cloud provider consoles
```

than:

```text
generic admin dashboard
```

Use:

* clear visual hierarchy
* restrained colors
* excellent typography
* consistent spacing
* dense but readable tables
* keyboard shortcuts
* command palette
* responsive layout
* dark mode
* accessible contrast
* skeleton loading
* meaningful empty states
* contextual error states

Every page must answer:

> What is happening?
> Why is it happening?
> What should I do next?

---

# 67. UI Consistency Audit

Before considering the UI complete, audit every screen for:

```text
spacing
typography
font sizes
border radius
icon usage
status colors
button hierarchy
empty states
loading states
error states
table density
navigation
breadcrumbs
pagination
filtering
responsive behavior
keyboard navigation
accessibility
dark mode
```

Remove visual inconsistencies.

Do not allow five different implementations of the same status badge or button.

Create shared components and refactor duplicates.

---

# 68. Accessibility

Target WCAG 2.2 AA where practical.

Support:

```text
keyboard navigation
focus indicators
semantic HTML
screen readers
accessible forms
accessible dialogs
reduced motion
sufficient contrast
```

Do not use color alone to communicate state.

Example:

```text
FAILED
icon + text + color
```

not just:

```text
red dot
```

---

# 69. Error Handling

Every layer needs explicit error handling.

API:

```text
structured errors
HTTP semantics
request_id
```

Scheduler:

```text
retry transient infrastructure failures
```

Worker:

```text
classify execution failures
```

Database:

```text
retry safe transient failures
```

NATS:

```text
reconnect
backoff
redelivery
```

Frontend:

```text
recoverable error UI
retry
offline/reconnecting state
```

Never display raw internal stack traces to users.

---

# 70. Disaster Recovery

Document:

```text
RPO
RTO
backup
restore
failover
data retention
```

Support PostgreSQL backups.

Support restoration testing.

A backup that has never been restored is not a recovery strategy.

Create automated restore verification in staging.

---

# 71. Health Endpoints

Implement:

```text
/health/live
/health/ready
/health/startup
```

Readiness must check required dependencies appropriately.

Do not make liveness fail merely because PostgreSQL is temporarily unavailable.

Otherwise the system may restart itself into a worse state.

---

# 72. Dependency Health

Expose:

```text
PostgreSQL
NATS
Object Storage
OIDC
OTel Collector
```

with states:

```text
HEALTHY
DEGRADED
UNAVAILABLE
```

Show these in the UI.

---

# 73. Deployment Safety

Implement:

```text
rolling deployment
worker draining
schema compatibility
backward-compatible API changes
grace periods
migration safety
```

Database migrations must be designed for rolling upgrades.

Do not introduce destructive schema changes that break old application versions during deployment.

Prefer:

```text
expand
migrate
contract
```

migration strategy.

---

# 74. CLI/API/UI Parity

Anything operationally important should be possible through the API.

The UI and CLI should call the same APIs.

For example:

```text
UI → API
CLI → API
SDK → API
```

Avoid:

```text
UI → private endpoint
CLI → database
```

---

# 75. Testing Strategy

Implement all four layers:

## Unit tests

Test:

```text
DAG engine
state transitions
retry policies
scheduler decisions
authorization
serialization
```

## Integration tests

Use real:

```text
PostgreSQL
NATS
MinIO
```

Do not mock everything.

## End-to-end tests

Use:

```text
frontend
API
scheduler
worker
database
messaging
```

Run real workflows.

## Chaos tests

Kill components and prove recovery.

---

# 76. Acceptance Test — Basic Workflow

The following must work:

```text
Create organization
    ↓
Create project
    ↓
Create workflow
    ↓
Validate workflow
    ↓
Create workflow version
    ↓
Schedule workflow
    ↓
Scheduler fires
    ↓
Tasks become READY
    ↓
NATS dispatch
    ↓
Worker receives
    ↓
Worker executes
    ↓
Worker streams heartbeat
    ↓
Worker completes
    ↓
Next DAG task becomes READY
    ↓
Workflow succeeds
    ↓
UI updates live
    ↓
Audit record created
```

---

# 77. Acceptance Test — Worker Crash

Scenario:

```text
Task running
↓
worker killed
```

Expected:

```text
heartbeat stops
↓
lease expires
↓
task becomes LOST
↓
retry policy evaluated
↓
new attempt created
↓
task dispatched
↓
new worker executes
↓
workflow continues
```

No manual database intervention allowed.

---

# 78. Acceptance Test — Scheduler Crash

Scenario:

```text
scheduler leader executing
↓
process killed
```

Expected:

```text
lease expires
↓
new scheduler becomes leader
↓
scheduling resumes
```

No duplicate scheduling.

No workflow corruption.

---

# 79. Acceptance Test — Duplicate Message

Send the same task dispatch message twice.

Expected:

```text
attempt state remains correct
```

No duplicate successful terminal execution.

---

# 80. Acceptance Test — API Retry

Send:

```http
POST /workflow-runs
Idempotency-Key: abc
```

twice.

Expected:

```text
one workflow run
```

not:

```text
two workflow runs
```

---

# 81. Acceptance Test — Tenant Isolation

Create:

```text
Tenant A
Tenant B
```

Verify:

```text
A cannot access B workflow
A cannot access B run
A cannot access B logs
A cannot access B artifacts
A cannot access B workers
```

Test using direct API calls, not only the UI.

---

# 82. Acceptance Test — Deployment

During a rolling deployment:

```text
workflow is executing
workers are running
new application version deployed
```

Expected:

```text
no task corruption
no orphaned execution
workers drain safely
new workers join
old workers leave
workflow continues
```

---

# 83. Acceptance Test — Database Failover

Trigger database failover.

Expected:

```text
temporary degradation
reconnection
execution resumes
no corrupted workflow state
```

---

# 84. Acceptance Test — NATS Failure

Restart one NATS node.

Expected:

```text
workers reconnect
durable consumers survive
messages are not silently lost
processing resumes
```

---

# 85. Acceptance Test — Complete System

Build a reference workload:

```text
100 workflows
1000 tasks
parallel DAG branches
retries
timeouts
worker failure
scheduler restart
notification
artifacts
logs
```

Run it automatically.

The complete workload must recover without manual intervention.

---

# 86. Observability Acceptance

For every workflow run, the operator should be able to navigate:

```text
UI
 ↓
Run
 ↓
Task
 ↓
Attempt
 ↓
Worker
 ↓
Trace
 ↓
Logs
 ↓
Artifact
```

All correlation IDs must remain consistent.

---

# 87. Operational Acceptance

The product must answer:

### Reliability

```text
Are tasks being lost?
Are workers healthy?
Are schedulers healthy?
Are queues growing?
```

### Performance

```text
How long are tasks waiting?
How long are they running?
Where is the bottleneck?
```

### Business operations

```text
Which workflows breached SLA?
Which workflows failed?
What needs operator attention?
```

---

# 88. Production Readiness Checklist

Do not declare the platform complete until all of the following are true:

```text
[ ] PostgreSQL migrations
[ ] workflow versioning
[ ] explicit workflow state machine
[ ] explicit task state machine
[ ] scheduler HA
[ ] leader election
[ ] fencing
[ ] NATS JetStream
[ ] durable consumers
[ ] outbox pattern
[ ] worker leases
[ ] heartbeat monitoring
[ ] retry policies
[ ] dead-letter queue
[ ] cancellation
[ ] worker draining
[ ] backpressure
[ ] concurrency controls
[ ] idempotent APIs
[ ] RBAC
[ ] OIDC
[ ] tenant isolation
[ ] audit logging
[ ] secret abstraction
[ ] object storage
[ ] structured logs
[ ] OpenTelemetry
[ ] Prometheus metrics
[ ] distributed tracing
[ ] live execution UI
[ ] production workflow DAG UI
[ ] log viewer
[ ] worker management
[ ] queue management
[ ] SLA tracking
[ ] notifications
[ ] CLI
[ ] TypeScript SDK
[ ] OpenAPI
[ ] Helm chart
[ ] Kubernetes hardening
[ ] security scanning
[ ] dependency scanning
[ ] integration tests
[ ] E2E tests
[ ] chaos tests
[ ] load tests
[ ] performance benchmarks
[ ] backup and restore
[ ] disaster recovery documentation
[ ] upgrade documentation
[ ] architecture documentation
```

---

# 89. Code Quality Rules

Follow these rules rigorously.

Rust:

* no unnecessary `unwrap`
* no unnecessary `expect`
* no panics in request paths
* no blocking operations in async runtime
* explicit timeouts around external I/O
* cancellation-aware operations
* typed domain errors
* structured logging
* bounded channels
* bounded concurrency
* no unbounded task spawning

TypeScript:

* strict mode
* no `any` unless justified
* typed API layer
* centralized query management
* reusable components
* error boundaries
* loading/error/empty states
* accessibility checks
* no duplicated API logic

---

# 90. Engineering Principles

Favor:

```text
correctness > cleverness
durability > convenience
explicitness > magic
observability > assumptions
backpressure > uncontrolled concurrency
idempotency > hope
typed contracts > duplicated schemas
immutable workflow versions > mutable execution definitions
```

Do not build "fake production".

For every reliability feature, implement the actual behavior and prove it with tests.

Do not create UI buttons for capabilities that don't work end-to-end.

Do not create mock health information.

Do not hard-code success states.

Do not use static fake metrics in production components.

---

# 91. Implementation Strategy

Do not attempt to rewrite the entire repository blindly in one step.

Execute the transformation in controlled phases.

## Phase 1 — Architecture

Produce:

```text
architecture.md
ADR documents
database model
message model
state machines
API contract
security model
```

Review the existing implementation and map every existing feature to:

```text
keep
refactor
replace
delete
```

## Phase 2 — Core domain

Implement:

```text
workflow versioning
state machines
persistence
transactions
outbox
```

## Phase 3 — Messaging

Implement:

```text
NATS
JetStream
dispatch
acknowledgement
retries
redelivery
```

## Phase 4 — HA

Implement:

```text
scheduler replicas
leader election
fencing
leases
worker heartbeats
```

## Phase 5 — Execution

Implement:

```text
task executors
container execution
Kubernetes execution
shell execution
cancellation
timeouts
```

## Phase 6 — Security

Implement:

```text
OIDC
RBAC
multi-tenancy
audit
secrets
```

## Phase 7 — Observability

Implement:

```text
OTel
metrics
logs
tracing
```

## Phase 8 — Frontend

Rebuild the frontend around:

```text
overview
workflow explorer
run explorer
live execution
workers
queues
events
audit
settings
```

## Phase 9 — Production deployment

Implement:

```text
Helm
Kubernetes
HA configuration
PDB
network policies
security contexts
resource limits
```

## Phase 10 — Verification

Run:

```text
unit
integration
E2E
chaos
load
security
performance
```

---

# 92. Definition of Done

The implementation is complete only when:

1. A user can create an organization and project.
2. A user can authenticate through OIDC.
3. A user can create a workflow.
4. Workflow validation detects invalid DAGs.
5. Workflow versions are immutable.
6. A workflow can be scheduled.
7. A workflow can be manually triggered.
8. Tasks are persisted transactionally.
9. Task messages are published through NATS JetStream.
10. Workers consume tasks durably.
11. Workers heartbeat.
12. Task leases expire correctly.
13. Dead workers are detected.
14. Lost tasks recover automatically.
15. Retries work correctly.
16. Duplicate messages are safe.
17. Cancellation works.
18. Dead-letter handling works.
19. Scheduler leader failover works.
20. Tenant isolation works.
21. RBAC works.
22. Audit logs work.
23. Secrets are protected.
24. Logs are durable.
25. Traces are correlated.
26. Metrics are available.
27. UI receives live execution updates.
28. Kubernetes deployment is production hardened.
29. Backup and restore procedures work.
30. Chaos tests pass.
31. Load tests produce documented results.
32. There are no known critical security issues.
33. Documentation matches actual behavior.
34. No UI control exists for a feature that is not implemented end-to-end.

---

# 93. Final Product Standard

The final system should feel like a combination of:

```text
Temporal
    +
Airflow
    +
modern Kubernetes-native execution
    +
enterprise workload automation
    +
Datadog/Grafana-quality observability
    +
developer-first CLI/API
```

but should remain distinctly FlowForge.

The product should be:

```text
Rust-native
cloud-native
API-first
event-driven
observable
fault-tolerant
secure
multi-tenant
operator-friendly
developer-friendly
```

Most importantly:

**Do not optimize for feature count. Optimize for correctness, reliability, operational clarity and architectural integrity.**

A smaller set of capabilities implemented exceptionally well is preferable to 100 shallow features.

The final implementation should be something an experienced Staff/Principal Engineer could inspect and say:

> "The authors understand distributed systems, failure modes, production operations and platform engineering."

Do not stop at "the application runs."

The acceptance bar is:

> **the system continues behaving correctly when things go wrong.**