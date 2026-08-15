use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;

pub struct MetricsRegistry;

impl MetricsRegistry {
    pub fn init() -> Result<(), String> {
        let builder = PrometheusBuilder::new();
        builder
            .install_recorder()
            .map_err(|e| format!("Failed to install prometheus recorder: {}", e))?;
        Ok(())
    }

    pub fn record_workflow_run_started(workflow_name: &str) {
        counter!("flowforge_workflow_runs_total", "workflow" => workflow_name.to_string(), "status" => "started").increment(1);
    }

    pub fn record_workflow_run_completed(workflow_name: &str, status: &str, duration_secs: f64) {
        counter!("flowforge_workflow_runs_total", "workflow" => workflow_name.to_string(), "status" => status.to_string()).increment(1);
        histogram!("flowforge_workflow_run_duration_seconds", "workflow" => workflow_name.to_string()).record(duration_secs);
    }

    pub fn record_task_executed(task_type: &str, status: &str, duration_secs: f64) {
        counter!("flowforge_tasks_total", "type" => task_type.to_string(), "status" => status.to_string()).increment(1);
        histogram!("flowforge_task_execution_duration_seconds", "type" => task_type.to_string())
            .record(duration_secs);
    }

    pub fn record_task_retry(task_id: &str) {
        counter!("flowforge_task_retries_total", "task" => task_id.to_string()).increment(1);
    }

    pub fn record_task_lost(task_id: &str) {
        counter!("flowforge_task_lost_total", "task" => task_id.to_string()).increment(1);
    }

    pub fn set_queue_depth(depth: f64) {
        gauge!("flowforge_queue_depth").set(depth);
    }

    pub fn set_worker_capacity(capacity: f64) {
        gauge!("flowforge_worker_capacity").set(capacity);
    }

    pub fn set_worker_utilization(utilization: f64) {
        gauge!("flowforge_worker_utilization").set(utilization);
    }

    pub fn record_leader_change() {
        counter!("flowforge_scheduler_leader_changes_total").increment(1);
    }
}
