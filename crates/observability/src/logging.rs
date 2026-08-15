use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(service_name: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,flowforge=debug"));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();

    tracing::info!(service = service_name, "Logging and tracing initialized");
}
