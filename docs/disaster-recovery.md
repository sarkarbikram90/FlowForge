# FlowForge Disaster Recovery & Backup Runbook

## 1. RPO & RTO Objectives
- **Recovery Point Objective (RPO)**: &lt; 5 minutes (via PostgreSQL WAL archiving and MinIO replication).
- **Recovery Time Objective (RTO)**: &lt; 15 minutes (automated Helm restore).

---

## 2. Backup Strategy
- **Database**: Automated nightly `pg_dump` and continuous WAL shipping to S3.
- **Workflow State**: PostgreSQL contains all state, versions, and execution histories.
- **NATS JetStream**: Ephemeral in-transit buffers backed by outbox queue in PostgreSQL.

---

## 3. Disaster Recovery Restoration
1. Provision target Kubernetes cluster or database instance.
2. Restore latest PostgreSQL backup snapshot.
3. Deploy Helm release: `helm install flowforge ./deploy/helm/flowforge`.
4. Scheduler auto-resumes active runs and reconciles task states seamlessly.
