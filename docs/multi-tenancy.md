# FlowForge Multi-Tenancy & Resource Isolation

FlowForge enforces multi-tenant boundaries at every architectural layer:

1. **Database Row-Level Isolation**: All entities (workflows, runs, tasks, leases) carry `organization_id` and `project_id` foreign keys.
2. **NATS Subject Isolation**: Queue subjects are namespaced per tenant (`flowforge.tasks.dispatch.{org_id}.{project_id}`).
3. **Execution Quotas**: Configurable rate limits and concurrency caps per organization/project preventing noisy-neighbor starvation.
4. **Secret Namespacing**: Secret references are scoped strictly to the owning project.
