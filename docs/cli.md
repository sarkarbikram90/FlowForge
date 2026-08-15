# FlowForge CLI Command Reference

The `flowforge` CLI provides complete administrative and operational control over FlowForge clusters.

---

## 1. Global Flags
- `--api-url, -a`: Target API server endpoint (default: `http://localhost:8080`, or env `FLOWFORGE_API_URL`).

---

## 2. Command Catalog

```bash
# Cluster status
flowforge status

# Authentication
flowforge auth login --email admin@flowforge.internal
flowforge auth whoami
flowforge auth apikey

# Workflows
flowforge workflow validate --file pipeline.yaml
flowforge workflow apply --file pipeline.yaml
flowforge workflow list
flowforge workflow get <workflow-id>

# Execution Runs
flowforge run trigger <workflow-name> --variables '{"batch_size": 10000}'
flowforge run list
flowforge run get <run-id>
flowforge run cancel <run-id>

# Workers
flowforge worker list
flowforge worker drain <worker-id>

# Queues & DLQ
flowforge queue dlq
flowforge queue resolve <dlq-id>
```
