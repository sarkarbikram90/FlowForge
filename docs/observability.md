# FlowForge Observability, Metrics & Tracing

FlowForge integrates OpenTelemetry and Prometheus out of the box.

---

## 1. Metrics Reference

Available on the `/api/v1/metrics` Prometheus scrape endpoint:

| Metric Name | Type | Description |
|---|---|---|
| `flowforge_workflow_runs_total` | Counter | Total workflow runs started by state |
| `flowforge_task_executions_total` | Counter | Total task execution attempts by outcome |
| `flowforge_task_duration_seconds` | Histogram | Latency distribution of task executions |
| `flowforge_workers_active` | Gauge | Currently connected and healthy worker agents |
| `flowforge_dlq_items_count` | Gauge | Count of unresolved Dead Letter Queue tasks |
| `flowforge_scheduler_leader` | Gauge | 1 if local instance is elected leader, 0 otherwise |

---

## 2. Distributed Tracing

Spans are exported via OTLP / gRPC and propagate correlation IDs across API gateway, scheduler progression cycles, and worker execution attempts.
