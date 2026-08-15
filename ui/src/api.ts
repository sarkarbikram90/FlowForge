import {
  AuditLog,
  DeadLetterTask,
  SystemStats,
  TaskRun,
  WorkerRegistration,
  Workflow,
  WorkflowRun,
  WorkflowVersion,
} from './types';

const API_BASE = (import.meta as any).env?.VITE_API_URL || 'http://localhost:8080/api/v1';

// In-memory fallback mock data if API server is not running locally
const MOCK_STATS: SystemStats = {
  active_workflows: 12,
  total_runs: 1420,
  running_runs: 4,
  succeeded_runs: 1380,
  failed_runs: 36,
  queued_tasks: 8,
  running_tasks: 14,
  active_workers: 6,
  dlq_count: 2,
  scheduler_leader_id: 'sched-primary-01',
  scheduler_healthy: true,
  success_rate: 97.4,
  average_duration_ms: 3240,
};

const MOCK_WORKFLOWS: Workflow[] = [
  {
    id: 'wf-001',
    organization_id: 'org-01',
    project_id: 'proj-01',
    name: 'daily-etl-pipeline',
    description: 'High-throughput customer analytics and data warehouse ETL sync',
    is_active: true,
    concurrency_limit: 10,
    created_at: new Date(Date.now() - 86400000 * 14).toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'wf-002',
    organization_id: 'org-01',
    project_id: 'proj-01',
    name: 'k8s-model-training',
    description: 'Distributed PyTorch model training and artifact persistence on S3',
    is_active: true,
    concurrency_limit: 4,
    created_at: new Date(Date.now() - 86400000 * 7).toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'wf-003',
    organization_id: 'org-01',
    project_id: 'proj-01',
    name: 'security-compliance-audit',
    description: 'Continuous container vulnerability and cloud configuration scans',
    is_active: true,
    concurrency_limit: 2,
    created_at: new Date(Date.now() - 86400000 * 3).toISOString(),
    updated_at: new Date().toISOString(),
  },
];

const MOCK_RUNS: WorkflowRun[] = [
  {
    id: 'run-9021',
    organization_id: 'org-01',
    project_id: 'proj-01',
    workflow_id: 'wf-001',
    workflow_version_id: 'ver-01',
    status: 'RUNNING',
    triggered_by: 'cron (0 * * * *)',
    trigger_metadata: {},
    variables: { batch_size: 50000 },
    started_at: new Date(Date.now() - 42000).toISOString(),
    created_at: new Date(Date.now() - 45000).toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'run-9020',
    organization_id: 'org-01',
    project_id: 'proj-01',
    workflow_id: 'wf-002',
    workflow_version_id: 'ver-02',
    status: 'SUCCEEDED',
    triggered_by: 'api',
    trigger_metadata: {},
    variables: { epochs: 20 },
    started_at: new Date(Date.now() - 3600000).toISOString(),
    finished_at: new Date(Date.now() - 3540000).toISOString(),
    duration_ms: 60000,
    created_at: new Date(Date.now() - 3605000).toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'run-9019',
    organization_id: 'org-01',
    project_id: 'proj-01',
    workflow_id: 'wf-003',
    workflow_version_id: 'ver-03',
    status: 'FAILED',
    triggered_by: 'webhook',
    trigger_metadata: {},
    variables: {},
    started_at: new Date(Date.now() - 7200000).toISOString(),
    finished_at: new Date(Date.now() - 7190000).toISOString(),
    duration_ms: 10000,
    error_summary: 'Task "scan-registry" failed with exit code 1',
    created_at: new Date(Date.now() - 7205000).toISOString(),
    updated_at: new Date().toISOString(),
  },
];

const MOCK_WORKERS: WorkerRegistration[] = [
  {
    worker_id: 'worker-us-east-01',
    hostname: 'k8s-node-4a8b.us-east.flowforge.internal',
    os: 'Linux (Ubuntu 24.04 LTS)',
    architecture: 'x86_64',
    version: '0.2.0',
    capabilities: ['shell', 'docker', 'container', 'http', 'python'],
    labels: { region: 'us-east', tier: 'high-compute' },
    max_concurrency: 16,
    current_load: 6,
    status: 'ONLINE',
    first_registered_at: new Date(Date.now() - 86400000 * 5).toISOString(),
    last_heartbeat_at: new Date().toISOString(),
  },
  {
    worker_id: 'worker-us-east-02',
    hostname: 'k8s-node-7f2c.us-east.flowforge.internal',
    os: 'Linux (Ubuntu 24.04 LTS)',
    architecture: 'x86_64',
    version: '0.2.0',
    capabilities: ['shell', 'docker', 'http', 'script'],
    labels: { region: 'us-east', tier: 'standard' },
    max_concurrency: 8,
    current_load: 2,
    status: 'ONLINE',
    first_registered_at: new Date(Date.now() - 86400000 * 2).toISOString(),
    last_heartbeat_at: new Date().toISOString(),
  },
  {
    worker_id: 'worker-eu-west-01',
    hostname: 'k8s-node-1e9d.eu-west.flowforge.internal',
    os: 'Linux (Ubuntu 24.04 LTS)',
    architecture: 'aarch64',
    version: '0.2.0',
    capabilities: ['shell', 'http', 'wait', 'condition'],
    labels: { region: 'eu-west', tier: 'arm64' },
    max_concurrency: 8,
    current_load: 0,
    status: 'DRAINING',
    first_registered_at: new Date(Date.now() - 86400000 * 10).toISOString(),
    last_heartbeat_at: new Date().toISOString(),
  },
];

async function apiFetch<T>(endpoint: string, options?: RequestInit): Promise<T> {
  try {
    const res = await fetch(`${API_BASE}${endpoint}`, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });
    const json = await res.json();
    if (json.success) {
      return json.data;
    }
    throw new Error(json.error?.message || 'API request failed');
  } catch (err) {
    console.warn(`[FlowForge API] Backend call to ${endpoint} fallback:`, err);
    return getFallbackData<T>(endpoint, options);
  }
}

function getFallbackData<T>(endpoint: string, options?: RequestInit): T {
  if (endpoint === '/stats') return MOCK_STATS as unknown as T;
  if (endpoint === '/workflows') {
    if (options?.method === 'POST') {
      const newWf: Workflow = {
        id: `wf-${Date.now()}`,
        organization_id: 'org-01',
        project_id: 'proj-01',
        name: 'custom-workflow',
        description: 'Applied via Web Console',
        is_active: true,
        concurrency_limit: 5,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      MOCK_WORKFLOWS.unshift(newWf);
      return { workflow: newWf } as unknown as T;
    }
    return MOCK_WORKFLOWS as unknown as T;
  }
  if (endpoint === '/workflow-runs') {
    if (options?.method === 'POST') {
      const newRun: WorkflowRun = {
        id: `run-${Math.floor(1000 + Math.random() * 9000)}`,
        organization_id: 'org-01',
        project_id: 'proj-01',
        workflow_id: 'wf-001',
        workflow_version_id: 'ver-01',
        status: 'RUNNING',
        triggered_by: 'operator (ui)',
        trigger_metadata: {},
        variables: {},
        started_at: new Date().toISOString(),
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      };
      MOCK_RUNS.unshift(newRun);
      return newRun as unknown as T;
    }
    return MOCK_RUNS as unknown as T;
  }
  if (endpoint === '/workers') return MOCK_WORKERS as unknown as T;
  if (endpoint === '/dlq') {
    return [
      {
        id: 'dlq-1',
        workflow_run_id: 'run-9019',
        task_run_id: 'task-scan',
        task_id: 'scan-registry',
        failure_reason: 'CONTAINER_IMAGE_PULL_FAILED',
        total_attempts: 3,
        payload: { image: 'internal.registry/sec-scan:v2.1' },
        last_error: 'Connection timeout connecting to image repository',
        is_resolved: false,
        created_at: new Date(Date.now() - 7190000).toISOString(),
      },
    ] as unknown as T;
  }
  if (endpoint === '/audit') {
    return [
      {
        id: 'aud-1',
        timestamp: new Date().toISOString(),
        actor: 'admin@flowforge.internal',
        action: 'WORKFLOW_TRIGGERED',
        resource_type: 'workflow_run',
        resource_id: 'run-9021',
        ip_address: '10.0.4.12',
        user_agent: 'FlowForge-UI/0.2.0',
        result: 'SUCCESS',
        metadata: { workflow: 'daily-etl-pipeline' },
      },
      {
        id: 'aud-2',
        timestamp: new Date(Date.now() - 3600000).toISOString(),
        actor: 'scheduler',
        action: 'WORKER_HEARTBEAT_RENEWED',
        resource_type: 'worker',
        resource_id: 'worker-us-east-01',
        ip_address: '10.0.1.20',
        user_agent: 'FlowForge-Worker/0.2.0',
        result: 'SUCCESS',
        metadata: { current_load: 6 },
      },
    ] as unknown as T;
  }
  return {} as T;
}

export const api = {
  getStats: () => apiFetch<SystemStats>('/stats'),
  getWorkflows: () => apiFetch<Workflow[]>('/workflows'),
  getWorkflow: (id: string) => apiFetch<{ workflow: Workflow; latest_version?: WorkflowVersion }>(`/workflows/${id}`),
  applyWorkflow: (yaml: string) =>
    apiFetch<{ workflow: Workflow; version: WorkflowVersion }>('/workflows', {
      method: 'POST',
      body: JSON.stringify({ yaml }),
    }),
  validateWorkflow: (yaml: string) =>
    apiFetch<any>('/workflows/validate', {
      method: 'POST',
      body: JSON.stringify({ yaml }),
    }),
  getRuns: () => apiFetch<WorkflowRun[]>('/workflow-runs'),
  getRun: (id: string) =>
    apiFetch<{ run: WorkflowRun; workflow?: Workflow; version?: WorkflowVersion; tasks: TaskRun[] }>(
      `/workflow-runs/${id}`
    ),
  triggerRun: (workflowName: string, variables = {}) =>
    apiFetch<WorkflowRun>('/workflow-runs', {
      method: 'POST',
      body: JSON.stringify({ workflow_name: workflowName, variables }),
    }),
  cancelRun: (id: string) =>
    apiFetch<any>(`/workflow-runs/${id}/cancel`, {
      method: 'POST',
    }),
  getWorkers: () => apiFetch<WorkerRegistration[]>('/workers'),
  drainWorker: (workerId: string) =>
    apiFetch<any>(`/workers/${workerId}/drain`, {
      method: 'POST',
    }),
  getDlq: () => apiFetch<DeadLetterTask[]>('/dlq'),
  resolveDlq: (id: string) =>
    apiFetch<any>(`/dlq/${id}/resolve`, {
      method: 'POST',
    }),
  getAuditLogs: () => apiFetch<AuditLog[]>('/audit'),
  generateApiKey: () => apiFetch<any>('/auth/keys', { method: 'POST' }),
};
