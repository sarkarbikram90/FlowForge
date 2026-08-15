use std::collections::HashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{is_cyclic_directed, toposort};
use flowforge_common::{FlowForgeError, Result, TaskSpec};

#[derive(Debug, Clone)]
pub struct DagGraph {
    pub graph: DiGraph<String, ()>,
    pub node_indices: HashMap<String, NodeIndex>,
    pub task_map: HashMap<String, TaskSpec>,
}

impl DagGraph {
    pub fn build(tasks: &[TaskSpec]) -> Result<Self> {
        let mut graph = DiGraph::<String, ()>::new();
        let mut node_indices = HashMap::new();
        let mut task_map = HashMap::new();

        // 1. Add all nodes and check for duplicate task IDs
        for task in tasks {
            if task_map.contains_key(&task.id) {
                return Err(FlowForgeError::Validation(format!(
                    "Duplicate task ID '{}' found in workflow definition",
                    task.id
                )));
            }
            let idx = graph.add_node(task.id.clone());
            node_indices.insert(task.id.clone(), idx);
            task_map.insert(task.id.clone(), task.clone());
        }

        // 2. Add edges and check for missing dependencies
        for task in tasks {
            let task_idx = node_indices[&task.id];
            for dep in &task.depends_on {
                match node_indices.get(dep) {
                    Some(&dep_idx) => {
                        graph.add_edge(dep_idx, task_idx, ());
                    }
                    None => {
                        return Err(FlowForgeError::Validation(format!(
                            "Task '{}' depends on non-existent task '{}'",
                            task.id, dep
                        )));
                    }
                }
            }
        }

        // 3. Cycle detection
        if is_cyclic_directed(&graph) {
            return Err(FlowForgeError::CycleDetected(
                "Workflow contains a dependency cycle / recursion".to_string(),
            ));
        }

        Ok(Self {
            graph,
            node_indices,
            task_map,
        })
    }

    pub fn topological_order(&self) -> Vec<String> {
        match toposort(&self.graph, None) {
            Ok(indices) => indices
                .into_iter()
                .map(|idx| self.graph[idx].clone())
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn get_roots(&self) -> Vec<String> {
        self.node_indices
            .iter()
            .filter(|(_, &idx)| {
                self.graph
                    .neighbors_directed(idx, petgraph::Direction::Incoming)
                    .count()
                    == 0
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn get_dependencies(&self, task_id: &str) -> Vec<String> {
        if let Some(&idx) = self.node_indices.get(task_id) {
            self.graph
                .neighbors_directed(idx, petgraph::Direction::Incoming)
                .map(|n_idx| self.graph[n_idx].clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_dependents(&self, task_id: &str) -> Vec<String> {
        if let Some(&idx) = self.node_indices.get(task_id) {
            self.graph
                .neighbors_directed(idx, petgraph::Direction::Outgoing)
                .map(|n_idx| self.graph[n_idx].clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn compute_critical_path(&self, task_durations: &HashMap<String, u64>) -> Vec<String> {
        let order = self.topological_order();
        let mut dist: HashMap<String, u64> = HashMap::new();
        let mut prev: HashMap<String, String> = HashMap::new();

        for task_id in &order {
            let task_cost = *task_durations.get(task_id).unwrap_or(&10);
            let current_max = *dist.get(task_id).unwrap_or(&0) + task_cost;
            dist.insert(task_id.clone(), current_max);

            for child in self.get_dependents(task_id) {
                let existing = *dist.get(&child).unwrap_or(&0);
                if current_max > existing {
                    dist.insert(child.clone(), current_max);
                    prev.insert(child, task_id.clone());
                }
            }
        }

        let mut max_node = order.first().cloned().unwrap_or_default();
        let mut max_dist = 0;
        for (node, &d) in &dist {
            if d > max_dist {
                max_dist = d;
                max_node = node.clone();
            }
        }

        let mut path = Vec::new();
        let mut curr = Some(max_node);
        while let Some(node) = curr {
            path.push(node.clone());
            curr = prev.get(&node).cloned();
        }
        path.reverse();
        path
    }
}
