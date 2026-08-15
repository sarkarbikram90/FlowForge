# FlowForge HA Scheduler & Progression Engine

## 1. High Availability & Distributed Leader Election

The FlowForge Scheduler implements an active-passive clustering model backed by PostgreSQL distributed leases:

- **Leader Leases**: The leader node holds a 5-second lease in the `scheduler_leases` table.
- **Heartbeat & Renewal**: The active leader renews its lease every 2 seconds.
- **Failover**: If the active leader becomes partitioned or crashes, standby nodes attempt acquisition upon lease expiration.
- **Fencing Tokens**: Every lease renewal increments a monotonic `lease_version` fencing token. Any stale write from a partitioned prior leader is rejected.

---

## 2. DAG Progression Algorithm

The scheduler evaluates active workflow runs in parallel:

1. **Query Active Runs**: Fetches runs in `PENDING` or `RUNNING` status.
2. **Topological Evaluation**: Evaluates the Petgraph DAG structure for the run's immutable `WorkflowVersion`.
3. **Dependency Readiness**: Identifies tasks whose prerequisite dependencies have all reached `SUCCEEDED`.
4. **Dispatch Queueing**: Transitions ready tasks to `READY` and inserts dispatch events into the transactional outbox.
5. **Terminal Resolution**: When all tasks reach terminal states, marks the `WorkflowRun` as `SUCCEEDED` (or `FAILED` if any task failed without recovery).
