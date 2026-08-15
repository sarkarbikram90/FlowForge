# FlowForge Worker Fleet & Agent Architecture

## 1. Worker Capabilities and Registration

Upon startup, each `WorkerAgent`:
1. **Discovers Host Environment**: Detects OS (Linux/Windows/macOS), CPU architecture (`x86_64`, `aarch64`), available tools (`docker`, `python`, `sh`, `curl`).
2. **Registers with PostgreSQL**: Records capabilities, concurrency limit, and custom operator labels (e.g. `gpu: true`, `tier: high-compute`).
3. **Heartbeat Loop**: Emits periodic heartbeats every 10 seconds updating current concurrency load.

---

## 2. Pull-Based Task Consumption

Workers consume tasks via NATS JetStream durable pull consumers:
- Respects local concurrency quotas (e.g. max 16 concurrent tasks).
- Matches task execution requirements against local capabilities.
- Renews the task lease in PostgreSQL in the background while execution is active.

---

## 3. Graceful Draining

When a worker receives a `DRAIN` command or `SIGTERM`:
1. Status changes to `DRAINING`.
2. New pull consumptions are halted immediately.
3. In-flight tasks are allowed to complete up to a configurable timeout period.
4. Clean process exit once active task count reaches zero.
