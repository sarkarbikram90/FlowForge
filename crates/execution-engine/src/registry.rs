use std::collections::HashMap;
use std::sync::Arc;
use flowforge_common::{FlowForgeError, Result};
use crate::executor::TaskExecutor;
use crate::shell::ShellExecutor;
use crate::http::HttpExecutor;
use crate::container::ContainerExecutor;
use crate::script::ScriptExecutor;
use crate::wait::WaitExecutor;
use crate::condition::ConditionExecutor;

pub struct ExecutorRegistry {
    executors: HashMap<String, Arc<dyn TaskExecutor>>,
}

impl Default for ExecutorRegistry {
    fn default() -> Self {
        let mut registry = Self {
            executors: HashMap::new(),
        };

        registry.register(Arc::new(ShellExecutor));
        registry.register(Arc::new(HttpExecutor::default()));
        registry.register(Arc::new(ContainerExecutor));
        registry.register(Arc::new(ScriptExecutor));
        registry.register(Arc::new(WaitExecutor));
        registry.register(Arc::new(ConditionExecutor));

        registry
    }
}

impl ExecutorRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, executor: Arc<dyn TaskExecutor>) {
        self.executors.insert(executor.supported_type().to_string(), executor);
    }

    pub fn get(&self, task_type: &str) -> Result<Arc<dyn TaskExecutor>> {
        // Map alias types: docker -> container, kubernetes -> container, python -> script
        let resolved_type = match task_type {
            "docker" | "kubernetes" => "container",
            "python" => "script",
            other => other,
        };

        self.executors
            .get(resolved_type)
            .cloned()
            .ok_or_else(|| FlowForgeError::Validation(format!(
                "No registered executor found for task type '{}'",
                task_type
            )))
    }
}
