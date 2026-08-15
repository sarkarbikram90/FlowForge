export type WorkflowState =
  | 'PENDING'
  | 'QUEUED'
  | 'RUNNING'
  | 'PAUSED'
  | 'SUCCEEDED'
  | 'FAILED'
  | 'CANCELING'
  | 'CANCELED'
  | 'TIMED_OUT'
  | 'RETRYING'
  | 'SUSPENDED';

export type TaskState =
  | 'PENDING'
  | 'BLOCKED'
  | 'READY'
  | 'DISPATCHED'
  | 'RUNNING'
  | 'SUCCEEDED'
  | 'FAILED'
  | 'RETRY_WAIT'
  | 'CANCELED'
  | 'TIMED_OUT'
  | 'LOST'
  | 'DEAD_LETTER';

export type WorkerStatus = 'ONLINE' | 'DEGRADED' | 'DRAINING' | 'OFFLINE' | 'LOST';

export interface Workflow {
  id: string;
  organization_id: string;
  project_id: string;
  name: string;
  description?: string;
  is_active: boolean;
  concurrency_limit: number;
  created_at: string;
  updated_at: string;
}

export interface WorkflowVersion {
  id: string;
  workflow_id: string;
  version_number: number;
  definition_yaml: string;
  definition_json: any;
  hash_sha256: string;
  is_latest: boolean;
  change_summary?: string;
  created_by: string;
  created_at: string;
}

export interface WorkflowRun {
  id: string;
  organization_id: string;
  project_id: string;
  workflow_id: string;
  workflow_version_id: string;
  idempotency_key?: string;
  status: WorkflowState;
  triggered_by: string;
  trigger_metadata: any;
  variables: any;
  started_at?: string;
  finished_at?: string;
  duration_ms?: number;
  error_summary?: string;
  created_at: string;
  updated_at: string;
}

export interface TaskRun {
  id: string;
  workflow_run_id: string;
  task_id: string;
  task_type: string;
  status: TaskState;
  attempt_count: number;
  max_attempts: number;
  current_worker_id?: string;
  started_at?: string;
  finished_at?: string;
  duration_ms?: number;
  output_data?: string;
  error_message?: string;
  created_at: string;
  updated_at: string;
}

export interface TaskAttempt {
  id: string;
  task_run_id: string;
  attempt_number: number;
  worker_id: string;
  status: TaskState;
  started_at: string;
  finished_at?: string;
  exit_code?: number;
  stdout_log_path?: string;
  stderr_log_path?: string;
  error_message?: string;
  duration_ms?: number;
  created_at: string;
}

export interface WorkerRegistration {
  worker_id: string;
  hostname: string;
  os: string;
  architecture: string;
  version: string;
  capabilities: string[];
  labels: Record<string, string>;
  max_concurrency: number;
  current_load: number;
  status: WorkerStatus;
  first_registered_at: string;
  last_heartbeat_at: string;
}

export interface DeadLetterTask {
  id: string;
  workflow_run_id: string;
  task_run_id: string;
  task_id: string;
  failure_reason: string;
  total_attempts: number;
  payload: any;
  last_error?: string;
  is_resolved: boolean;
  resolved_at?: string;
  resolved_by?: string;
  created_at: string;
}

export interface AuditLog {
  id: string;
  timestamp: string;
  organization_id?: string;
  project_id?: string;
  actor: string;
  action: string;
  resource_type: string;
  resource_id?: string;
  ip_address?: string;
  user_agent?: string;
  result: string;
  metadata: any;
}

export interface SystemStats {
  active_workflows: number;
  total_runs: number;
  running_runs: number;
  succeeded_runs: number;
  failed_runs: number;
  queued_tasks: number;
  running_tasks: number;
  active_workers: number;
  dlq_count: number;
  scheduler_leader_id?: string;
  scheduler_healthy: boolean;
  success_rate: number;
  average_duration_ms: number;
}
