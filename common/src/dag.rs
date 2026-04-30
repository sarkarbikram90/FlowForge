use petgraph::graph::DiGraph;
use petgraph::algo::toposort;
use std::collections::HashMap;
use crate::models::DagDefinition;
use crate::error::{FlowForgeError, Result};

/// Validate a DAG definition: check for cycles, missing deps, duplicate task IDs.
pub fn validate_dag(dag: &DagDefinition) -> Result<Vec<String>> {
    if dag.tasks.is_empty() {
        return Err(FlowForgeError::DagValidation(
            "DAG must have at least one task".to_string(),
        ));
    }

    // Check for duplicate task IDs
    let mut seen = std::collections::HashSet::new();
    for task in &dag.tasks {
        if !seen.insert(&task.id) {
            return Err(FlowForgeError::DagValidation(format!(
                "Duplicate task ID: {}",
                task.id
            )));
        }
    }

    let task_ids: HashMap<&str, usize> = dag
        .tasks
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id.as_str(), i))
        .collect();

    // Check all dependencies reference existing tasks
    for task in &dag.tasks {
        for dep in &task.depends_on {
            if !task_ids.contains_key(dep.as_str()) {
                return Err(FlowForgeError::DagValidation(format!(
                    "Task '{}' depends on unknown task '{}'",
                    task.id, dep
                )));
            }
        }
    }

    // Build graph and check for cycles via topological sort
    let mut graph = DiGraph::<&str, ()>::new();
    let mut node_map = HashMap::new();

    for task in &dag.tasks {
        let idx = graph.add_node(task.id.as_str());
        node_map.insert(task.id.as_str(), idx);
    }

    for task in &dag.tasks {
        let to = node_map[task.id.as_str()];
        for dep in &task.depends_on {
            let from = node_map[dep.as_str()];
            graph.add_edge(from, to, ());
        }
    }

    match toposort(&graph, None) {
        Ok(sorted) => {
            let order: Vec<String> = sorted
                .iter()
                .map(|idx| graph[*idx].to_string())
                .collect();
            Ok(order)
        }
        Err(_) => Err(FlowForgeError::CycleDetected(dag.id.clone())),
    }
}

/// Get tasks that are ready to execute (all dependencies satisfied).
pub fn get_ready_tasks(
    dag: &DagDefinition,
    completed_tasks: &std::collections::HashSet<String>,
) -> Vec<String> {
    dag.tasks
        .iter()
        .filter(|t| {
            !completed_tasks.contains(&t.id)
                && t.depends_on.iter().all(|dep| completed_tasks.contains(dep))
        })
        .map(|t| t.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DagDefinition, TaskDefinition};

    fn make_dag(tasks: Vec<TaskDefinition>) -> DagDefinition {
        DagDefinition {
            id: "test-dag".to_string(),
            name: "Test DAG".to_string(),
            description: String::new(),
            schedule: None,
            default_retries: 3,
            tasks,
        }
    }

    fn task(id: &str, deps: Vec<&str>) -> TaskDefinition {
        TaskDefinition {
            id: id.to_string(),
            name: id.to_string(),
            command: format!("echo {id}"),
            depends_on: deps.into_iter().map(String::from).collect(),
            retries: None,
            timeout_secs: 60,
            env: Default::default(),
        }
    }

    #[test]
    fn test_valid_linear_dag() {
        let dag = make_dag(vec![
            task("a", vec![]),
            task("b", vec!["a"]),
            task("c", vec!["b"]),
        ]);
        let order = validate_dag(&dag).unwrap();
        let a_pos = order.iter().position(|x| x == "a").unwrap();
        let b_pos = order.iter().position(|x| x == "b").unwrap();
        let c_pos = order.iter().position(|x| x == "c").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_cycle_detection() {
        let dag = make_dag(vec![
            task("a", vec!["c"]),
            task("b", vec!["a"]),
            task("c", vec!["b"]),
        ]);
        assert!(matches!(validate_dag(&dag), Err(FlowForgeError::CycleDetected(_))));
    }

    #[test]
    fn test_missing_dependency() {
        let dag = make_dag(vec![task("a", vec!["nonexistent"])]);
        assert!(matches!(validate_dag(&dag), Err(FlowForgeError::DagValidation(_))));
    }

    #[test]
    fn test_duplicate_task_id() {
        let dag = make_dag(vec![task("a", vec![]), task("a", vec![])]);
        assert!(matches!(validate_dag(&dag), Err(FlowForgeError::DagValidation(_))));
    }

    #[test]
    fn test_ready_tasks() {
        let dag = make_dag(vec![
            task("a", vec![]),
            task("b", vec!["a"]),
            task("c", vec!["a"]),
            task("d", vec!["b", "c"]),
        ]);
        let mut completed = std::collections::HashSet::new();

        // Initially only 'a' is ready
        let ready = get_ready_tasks(&dag, &completed);
        assert_eq!(ready, vec!["a"]);

        // After 'a' completes, 'b' and 'c' are ready
        completed.insert("a".to_string());
        let mut ready = get_ready_tasks(&dag, &completed);
        ready.sort();
        assert_eq!(ready, vec!["b", "c"]);

        // After 'b' and 'c' complete, 'd' is ready
        completed.insert("b".to_string());
        completed.insert("c".to_string());
        let ready = get_ready_tasks(&dag, &completed);
        assert_eq!(ready, vec!["d"]);
    }
}
