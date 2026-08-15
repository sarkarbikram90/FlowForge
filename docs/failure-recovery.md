# FlowForge Failure Modes and Automatic Recovery

## 1. Failure Scenarios and Guarantees

```mermaid
sequenceDiagram
    autonumber
    participant W as Worker 1
    participant DB as PostgreSQL
    participant S as Scheduler Leader
    participant W2 as Worker 2

    Note over W: Worker 1 crashes mid-task
    W--xDB: Heartbeat stops
    Note over DB: Task Lease expires (30s timeout)
    S->>DB: Stale lease sweep
    DB-->>S: Returns expired task lease
    Note over S: Mark task status as LOST
    Note over S: Evaluate Retry Policy (attempts < max)
    S->>DB: Update task to READY
    S->>W2: Dispatch new attempt to Worker 2
    W2->>DB: Acquire new Lease & Execute
    W2->>DB: Mark SUCCEEDED
```

### Automatic Recovery Guarantees
1. **Worker Crash / Loss**: Monitored via 30s leases. Tasks are transitioned to `LOST` and automatically requeued.
2. **Scheduler Crash / Leader Loss**: Standby schedulers automatically acquire the lease within 2 seconds.
3. **Duplicate Message Redelivery**: Application-level idempotency prevents double execution or conflicting completions.
4. **Network Partitioning**: Fencing tokens reject stale writes from isolated components.
