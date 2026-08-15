# FlowForge Task Execution Model

FlowForge separates workflow topology resolution (DAG progression) from task execution (Worker runtime).

---

## 1. Execution Lifecycle

```text
[Scheduler Engine] ──▶ Evaluates DAG Dependencies
          │
          ├──▶ Prerequisite Tasks SUCCEEDED?
          │         │
          │         ├──▶ YES: Mark Task READY, Insert Outbox Message
          │         └──▶ NO:  Keep Task BLOCKED
          │
[Outbox Publisher] ──▶ Publish TaskDispatchMessage to NATS JetStream
          │
[Worker Pull Loop] ──▶ Fetch next TaskDispatchMessage matching capabilities
          │
[Worker Lease]     ──▶ Acquire 30-second renewable Task Lease in PostgreSQL
          │
[Task Executor]    ──▶ Execute payload (Shell, Container, HTTP, Script, Wait)
          │            │
          │            ├──▶ Stream logs / capture stdout & stderr
          │            ├──▶ Heartbeat / renew lease every 10 seconds
          │            └──▶ Honor cancellation tokens on user cancel / timeout
          │
[Completion]       ──▶ Publish TaskCompletionMessage (SUCCEEDED / FAILED)
          │
[Scheduler Engine] ──▶ Update TaskRun status, evaluate downstream DAG dependents
```

---

## 2. Process Isolation & Resource Limits

1. **Process Isolation**: Each shell execution runs in its own process tree with environment variable sanitization.
2. **Container Runner**: Docker containers execute with `--rm`, explicit memory/CPU constraints, and automatic cleanup hooks.
3. **Cancellation Safety**: When a workflow is canceled or times out, the cancellation token triggers child process termination (`SIGTERM` followed by `SIGKILL` if unyielding).
