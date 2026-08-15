use flowforge_auth::AuthContext;
use flowforge_common::PlatformConfig;
use flowforge_execution_engine::ExecutorRegistry;
use flowforge_messaging::{InMemoryMessageBus, MessageBus};
use flowforge_persistence::{InMemoryDatabase, PostgresDatabase, Repository};
use std::sync::Arc;
use tokio::sync::broadcast;

pub type DynRepository = Arc<dyn Repository>;
pub type DynMessageBus = Arc<dyn MessageBus>;

#[derive(Clone)]
pub struct AppState {
    pub repo: DynRepository,
    pub bus: DynMessageBus,
    pub executors: Arc<ExecutorRegistry>,
    pub config: Arc<PlatformConfig>,
    pub event_tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new_in_memory() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            repo: Arc::new(InMemoryDatabase::new()),
            bus: Arc::new(InMemoryMessageBus::new()),
            executors: Arc::new(ExecutorRegistry::default()),
            config: Arc::new(PlatformConfig::default()),
            event_tx: tx,
        }
    }

    pub fn new_with_db(db: PostgresDatabase, config: PlatformConfig) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            repo: Arc::new(db),
            bus: Arc::new(InMemoryMessageBus::new()),
            executors: Arc::new(ExecutorRegistry::default()),
            config: Arc::new(config),
            event_tx: tx,
        }
    }

    pub fn broadcast_event(&self, event: &str) {
        let _ = self.event_tx.send(event.to_string());
    }
}
