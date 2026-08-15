# FlowForge Kubernetes & Production Deployment

FlowForge deploys natively on Kubernetes via Helm charts located in `deploy/helm/flowforge/`.

---

## 1. Quick Helm Installation

```bash
# Add values override if needed
helm install flowforge ./deploy/helm/flowforge \
  --set global.environment=production \
  --set postgresql.auth.password="supersecret"
```

---

## 2. Hardening Features
- **Pod Disruption Budgets (PDB)**: Guarantees API and scheduler availability during node drains.
- **Network Policies**: Restricts ingress and egress to authorized pods and ports.
- **Horizontal Pod Autoscaling (HPA)**: Dynamically scales worker pods based on CPU and queue backpressure.
- **Readiness/Liveness Probes**: Integrated with `/api/v1/health/ready` and `/api/v1/health/live`.
