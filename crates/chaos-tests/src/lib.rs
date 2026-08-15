#[cfg(test)]
mod tests {
    use chrono::Utc;
    use flowforge_common::{TaskRun, TaskState, Workflow, WorkflowRun, WorkflowState};
    use flowforge_execution_engine::ExecutorRegistry;
    use flowforge_messaging::InMemoryMessageBus;
    use flowforge_persistence::{InMemoryDatabase, Repository};
    use flowforge_scheduler::{LeaderElector, SchedulerEngine, StaleLeaseDetector};
    use flowforge_worker::WorkerAgent;
    use flowforge_workflow_engine::WorkflowCompiler;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio_util::sync::CancellationToken;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_e2e_workflow_execution() {
        let repo = Arc::new(InMemoryDatabase::new());
        let bus = Arc::new(InMemoryMessageBus::new());
        let executors = Arc::new(ExecutorRegistry::default());

        let (org, proj) = repo.get_or_create_default_org().await.unwrap();

        let yaml = r#"
apiVersion: flowforge.io/v1
kind: Workflow
metadata:
  name: test-e2e-pipeline
spec:
  tasks:
    - id: task-a
      type: wait
      waitSecs: 1
    - id: task-b
      type: wait
      waitSecs: 1
      dependsOn:
        - task-a
"#;

        let wf = Workflow {
            id: Uuid::new_v4(),
            organization_id: org.id,
            project_id: proj.id,
            name: "test-e2e-pipeline".to_string(),
            description: None,
            is_active: true,
            concurrency_limit: 5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        repo.save_workflow(wf.clone()).await.unwrap();

        let version = WorkflowCompiler::compile_version(wf.id, 1, yaml, "test").unwrap();
        repo.save_workflow_version(version.clone()).await.unwrap();

        // Start worker agent
        let worker = Arc::new(WorkerAgent::new(
            "worker-test-1",
            repo.clone(),
            bus.clone(),
            executors.clone(),
            4,
        ));
        worker.register().await.unwrap();

        let cancel_token = CancellationToken::new();
        let pull_token = cancel_token.clone();
        let worker_clone = worker.clone();
        tokio::spawn(async move {
            worker_clone.run_task_pull_loop(pull_token).await;
        });

        // Trigger workflow run
        let run_id = Uuid::new_v4();
        let run = WorkflowRun {
            id: run_id,
            organization_id: org.id,
            project_id: proj.id,
            workflow_id: wf.id,
            workflow_version_id: version.id,
            idempotency_key: None,
            status: WorkflowState::Pending,
            triggered_by: "test".to_string(),
            trigger_metadata: serde_json::json!({}),
            variables: serde_json::json!({}),
            started_at: None,
            finished_at: None,
            duration_ms: None,
            error_summary: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        repo.create_workflow_run(run.clone()).await.unwrap();

        let engine = SchedulerEngine::new(repo.clone(), bus.clone(), Duration::from_millis(200));

        // Step 1: Progress first task (task-a)
        engine.progress_all_active_runs().await.unwrap();
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Step 2: Progress second task (task-b) after task-a completes
        engine.progress_all_active_runs().await.unwrap();
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Step 3: Final evaluation
        engine.progress_all_active_runs().await.unwrap();

        let updated_run = repo.get_workflow_run(run_id).await.unwrap();
        assert_eq!(updated_run.status, WorkflowState::Succeeded);

        cancel_token.cancel();
    }

    #[tokio::test]
    async fn test_scheduler_leader_failover() {
        let repo = Arc::new(InMemoryDatabase::new());

        let elector1 = LeaderElector::new(
            repo.clone(),
            "scheduler",
            "node-1",
            2,
            Duration::from_millis(100),
        );
        let elector2 = LeaderElector::new(
            repo.clone(),
            "scheduler",
            "node-2",
            2,
            Duration::from_millis(100),
        );

        let cancel1 = CancellationToken::new();
        let cancel2 = CancellationToken::new();

        let c1 = cancel1.clone();
        let elector1_arc = Arc::new(elector1);
        let e1 = elector1_arc.clone();
        tokio::spawn(async move { e1.run_election_loop(c1).await });

        let c2 = cancel2.clone();
        let elector2_arc = Arc::new(elector2);
        let e2 = elector2_arc.clone();
        tokio::spawn(async move { e2.run_election_loop(c2).await });

        tokio::time::sleep(Duration::from_millis(400)).await;

        // Exactly one should be leader
        let l1 = elector1_arc.is_leader();
        let l2 = elector2_arc.is_leader();
        assert!(l1 ^ l2, "Exactly one node must be elected leader");

        // Stop leader
        if l1 {
            cancel1.cancel();
        } else {
            cancel2.cancel();
        }

        // Wait for lease to expire and standby to take over
        tokio::time::sleep(Duration::from_millis(2500)).await;

        let final_l1 = elector1_arc.is_leader();
        let final_l2 = elector2_arc.is_leader();
        assert!(
            final_l1 || final_l2,
            "Standby node must take over leadership"
        );

        cancel1.cancel();
        cancel2.cancel();
    }

    #[tokio::test]
    async fn test_worker_crash_stale_lease_recovery() {
        let repo = Arc::new(InMemoryDatabase::new());
        let (org, proj) = repo.get_or_create_default_org().await.unwrap();

        let run_id = Uuid::new_v4();
        let task_run_id = Uuid::new_v4();

        let run = WorkflowRun {
            id: run_id,
            organization_id: org.id,
            project_id: proj.id,
            workflow_id: Uuid::new_v4(),
            workflow_version_id: Uuid::new_v4(),
            idempotency_key: None,
            status: WorkflowState::Running,
            triggered_by: "test".to_string(),
            trigger_metadata: serde_json::json!({}),
            variables: serde_json::json!({}),
            started_at: Some(Utc::now()),
            finished_at: None,
            duration_ms: None,
            error_summary: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        repo.create_workflow_run(run).await.unwrap();

        let task_run = TaskRun {
            id: task_run_id,
            workflow_run_id: run_id,
            task_id: "critical-step".to_string(),
            task_type: "shell".to_string(),
            status: TaskState::Running,
            attempt_count: 1,
            max_attempts: 3,
            current_worker_id: Some("crashed-worker".to_string()),
            started_at: Some(Utc::now() - chrono::Duration::seconds(40)),
            finished_at: None,
            duration_ms: None,
            output_data: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        repo.create_task_run(task_run).await.unwrap();

        // Simulate expired lease
        let _lease = repo
            .acquire_or_renew_task_lease(task_run_id, "crashed-worker", Uuid::new_v4(), 1)
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(1500)).await;

        // Run detector
        let detector = StaleLeaseDetector::new(repo.clone(), Duration::from_millis(100));
        detector.sweep_stale_leases().await.unwrap();

        let recovered_task = repo.get_task_run(task_run_id).await.unwrap();
        assert_eq!(recovered_task.status, TaskState::Ready);
    }
}
