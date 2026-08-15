use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub database_url: String,
    pub nats_url: String,
    pub redis_url: Option<String>,
    pub api_host: String,
    pub api_port: u16,
    pub scheduler_interval_secs: u64,
    pub worker_concurrency: usize,
    pub worker_id: String,
    pub lease_duration_secs: u64,
    pub heartbeat_interval_secs: u64,
    pub otel_endpoint: Option<String>,
    pub object_storage_endpoint: Option<String>,
    pub object_storage_bucket: Option<String>,
    pub oidc_issuer: Option<String>,
    pub jwt_secret: String,
    pub environment: String,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("FLOWFORGE_DATABASE_URL")
                .or_else(|_| std::env::var("DATABASE_URL"))
                .unwrap_or_else(|_| "postgres://flowforge:flowforge@localhost:5432/flowforge".to_string()),
            nats_url: std::env::var("FLOWFORGE_NATS_URL")
                .or_else(|_| std::env::var("NATS_URL"))
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            redis_url: std::env::var("REDIS_URL").ok(),
            api_host: std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            api_port: std::env::var("API_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            scheduler_interval_secs: std::env::var("SCHEDULER_INTERVAL_SECS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(2),
            worker_concurrency: std::env::var("WORKER_CONCURRENCY")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8),
            worker_id: std::env::var("WORKER_ID")
                .unwrap_or_else(|_| format!("worker-{}", &uuid::Uuid::new_v4().to_string()[..8])),
            lease_duration_secs: 30,
            heartbeat_interval_secs: 5,
            otel_endpoint: std::env::var("FLOWFORGE_OTEL_ENDPOINT").ok(),
            object_storage_endpoint: std::env::var("FLOWFORGE_OBJECT_STORAGE_ENDPOINT").ok(),
            object_storage_bucket: std::env::var("FLOWFORGE_OBJECT_STORAGE_BUCKET").ok(),
            oidc_issuer: std::env::var("FLOWFORGE_OIDC_ISSUER").ok(),
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "flowforge-dev-secret-key-super-secure".to_string()),
            environment: std::env::var("FLOWFORGE_ENV").unwrap_or_else(|_| "development".to_string()),
        }
    }
}
