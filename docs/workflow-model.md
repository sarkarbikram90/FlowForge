# FlowForge Workflow & Execution Model

## 1. Workflow Definition Schema

FlowForge supports YAML and JSON workflow definitions conforming to the `flowforge.io/v1` API spec.

```yaml
apiVersion: flowforge.io/v1
kind: Workflow

metadata:
  name: daily-etl-pipeline
  description: Extract, Transform and Load
  version: 1

spec:
  schedule:
    cron: "0 * * * *"
    timezone: "UTC"

  concurrency:
    maxRuns: 5

  retries:
    maxAttempts: 3
    backoff: exponential_with_jitter
    initialIntervalSecs: 5
    maxIntervalSecs: 120
    jitterFactor: 0.2

  sla:
    completionTime: "30m"
    severity: high

  tasks:
    - id: extract
      type: shell
      command: ./extract.sh

    - id: transform
      type: container
      image: company/transform:latest
      dependsOn:
        - extract

    - id: load
      type: http
      url: https://api.warehouse.internal/v1/ingest
      method: POST
      dependsOn:
        - transform
```

---

## 2. Supported Task Executors

| Task Type | Executor | Description |
|---|---|---|
| `shell` | `ShellExecutor` | Child process execution with environment variables and streamed stdout/stderr. |
| `container` / `docker` | `ContainerExecutor` | Docker container runner with resource limits and graceful termination. |
| `http` | `HttpExecutor` | Asynchronous REST and webhook requests via Reqwest. |
| `script` / `python` | `ScriptExecutor` | Inline script runner for Python, Node, or Bash scripts. |
| `wait` | `WaitExecutor` | Non-blocking asynchronous delay/timer. |
| `condition` | `ConditionExecutor` | Guard expressions and branching criteria. |
