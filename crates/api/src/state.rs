use flowforge_common::PlatformConfig;
use flowforge_messaging::{InMemoryMessageBus, MessageBus};
use flowforge_persistence::{InMemoryDatabase, Repository};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn Repository>,
    pub bus: Arc<dyn MessageBus>,
    pub config: PlatformConfig,
}

impl AppState {
    pub fn new_in_memory() -> Self {
        Self {
            repo: Arc::new(InMemoryDatabase::new()),
            bus: Arc::new(InMemoryMessageBus::new()),
            config: PlatformConfig::default(),
        }
    }

    pub fn new_with_db<R: Repository + 'static>(repo: R, config: PlatformConfig) -> Self {
        Self {
            repo: Arc::new(repo),
            bus: Arc::new(InMemoryMessageBus::new()),
            config,
        }
    }
}
