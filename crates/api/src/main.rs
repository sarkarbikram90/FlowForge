use flowforge_api::{create_router, AppState};
use flowforge_common::PlatformConfig;
use flowforge_observability::{init_tracing, MetricsRegistry};
use flowforge_persistence::PostgresDatabase;
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing("flowforge-api");
    let _ = MetricsRegistry::init();

    let config = PlatformConfig::default();
    let addr: SocketAddr = format!("{}:{}", config.api_host, config.api_port).parse()?;

    info!(
        "Connecting to PostgreSQL database at: {}",
        config.database_url
    );
    let app_state = match PostgresDatabase::connect(&config.database_url).await {
        Ok(db) => {
            info!("Connected to PostgreSQL successfully");
            AppState::new_with_db(db, config)
        }
        Err(e) => {
            tracing::warn!("PostgreSQL connection failed ({}), falling back to in-memory database mode for development", e);
            AppState::new_in_memory()
        }
    };

    let router = create_router(app_state);

    info!("FlowForge API Gateway listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
