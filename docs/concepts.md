# FlowForge Core Concepts & Domain Model

## 1. Domain Entities

- **Organization**: Top-level tenant container representing a customer, company, or division.
- **Project**: Isolation boundary within an Organization where workflows, secrets, variables, and workers reside.
- **Workflow**: Logical blueprint describing a DAG of tasks and their scheduling/concurrency policies.
- **Workflow Version**: Deterministically hashed, immutable snapshot of a workflow definition. Once a run begins, its version is fixed.
- **Workflow Run**: Single execution instance of a Workflow Version.
- **Task Run**: Execution state of an individual DAG node within a Workflow Run.
- **Task Attempt**: Specific execution attempt of a Task Run on an assigned Worker.
- **Task Lease**: Time-bounded reservation token ensuring single-worker execution ownership.
- **Worker**: Distributed execution daemon registering system capabilities, heartbeating, and pulling work units.
- **Dead Letter Queue (DLQ)**: Quarantine subsystem holding tasks that exhausted all retry attempts.

---

## 2. State Machines

### Workflow State Machine
```text
PENDING ──▶ QUEUED ──▶ RUNNING ──▶ SUCCEEDED (Terminal)
                          │
                          ├──▶ FAILED (Terminal)
                          ├──▶ CANCELING ──▶ CANCELED (Terminal)
                          ├──▶ TIMED_OUT (Terminal)
                          └──▶ PAUSED / RETRYING
```

### Task State Machine
```text
PENDING ──▶ BLOCKED ──▶ READY ──▶ DISPATCHED ──▶ RUNNING ──▶ SUCCEEDED (Terminal)
                                                   │
                                                   ├──▶ FAILED ──▶ RETRY_WAIT ──▶ READY
                                                   ├──▶ LOST ──▶ RETRY_WAIT
                                                   ├──▶ TIMED_OUT
                                                   ├──▶ CANCELED (Terminal)
                                                   └──▶ DEAD_LETTER (Terminal)
```
