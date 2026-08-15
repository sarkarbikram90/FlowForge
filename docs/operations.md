# FlowForge Day-2 Operations Runbook

## 1. Rolling Worker Node Upgrades
1. Drain worker:
   ```bash
   flowforge worker drain <worker-id>
   ```
2. Wait for active tasks on the worker to complete.
3. Terminate or upgrade worker pod/host.
4. Start new worker container; capability discovery and registration will auto-occur.

---

## 2. Managing Dead Letter Tasks
1. List DLQ items:
   ```bash
   flowforge queue dlq
   ```
2. Inspect failure reasons (e.g. timeout, missing image).
3. Fix underlying cause and trigger recovery:
   ```bash
   flowforge queue resolve <dlq-id>
   ```
