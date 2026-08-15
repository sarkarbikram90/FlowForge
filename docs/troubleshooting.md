# FlowForge Troubleshooting & Diagnostic Guide

## 1. Common Symptoms & Root Causes

### Stale Scheduler Leader
- **Symptom**: Schedulers not progressing active runs.
- **Diagnostics**: Check `GET /api/v1/health/ready` or `scheduler_leases` table in PostgreSQL.
- **Resolution**: Verify database clock skew / NTP sync and network connectivity.

### Tasks Stuck in RUNNING
- **Symptom**: Worker crashed without reporting completion.
- **Diagnostics**: The stale lease detector automatically identifies tasks after 30s lease timeout and transitions them to `LOST` for retry.

### DLQ Ingestion Spikes
- **Symptom**: Excessive tasks routed to DLQ.
- **Diagnostics**: Inspect container registry authentication or database timeout limits.
