use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub api_host: String,
    pub api_port: u16,
    pub worker_concurrency: usize,
    pub scheduler_interval_secs: u64,
    pub heartbeat_interval_secs: u64,
    pub heartbeat_timeout_secs: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://flowforge:flowforge@localhost:5432/flowforge".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            api_host: "0.0.0.0".to_string(),
            api_port: 8080,
            worker_concurrency: 4,
            scheduler_interval_secs: 5,
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> Self {
        let def = Self::default();
        Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or(def.database_url),
            redis_url: std::env::var("REDIS_URL").unwrap_or(def.redis_url),
            api_host: std::env::var("API_HOST").unwrap_or(def.api_host),
            api_port: std::env::var("API_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(def.api_port),
            worker_concurrency: std::env::var("WORKER_CONCURRENCY").ok().and_then(|c| c.parse().ok()).unwrap_or(def.worker_concurrency),
            scheduler_interval_secs: std::env::var("SCHEDULER_INTERVAL_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(def.scheduler_interval_secs),
            heartbeat_interval_secs: def.heartbeat_interval_secs,
            heartbeat_timeout_secs: def.heartbeat_timeout_secs,
        }
    }
}
