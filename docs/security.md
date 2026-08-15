# FlowForge Security & RBAC

## 1. Role-Based Access Control (RBAC)

FlowForge provides fine-grained, multi-tenant role assignments:

| Role | Description | Permissions |
|---|---|---|
| `PlatformAdmin` | Superuser across all organizations | Full system control, cluster management |
| `OrgAdmin` | Administrator of an Organization | Project provisioning, user management, billing |
| `ProjectAdmin` | Administrator of a Project | Workflow authoring, worker management, secret management |
| `WorkflowEditor` | Developer | Create, update, and validate workflows |
| `WorkflowOperator` | Operations Engineer | Trigger runs, cancel runs, drain workers, resolve DLQ |
| `Viewer` | Read-only user | View runs, DAGs, logs, metrics |
| `Auditor` | Compliance Officer | Query immutable audit logs and security events |

---

## 2. API Key Authentication

- API keys use the prefix format: `ff_live_<hex32>` (e.g. `ff_live_7a8b9c0d1e...`).
- Keys are hashed with SHA-256 before storage in PostgreSQL.
- Authenticated requests pass credentials via the `Authorization: Bearer <api_key>` header.
