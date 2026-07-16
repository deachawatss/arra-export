use std::net::SocketAddr;

use arra_export::{
    app::{AppState, app_router},
    config::AppConfig,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "arra_export=info,tower_http=info".into()),
        )
        .init();

    let config = match AppConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("configuration error: {error}");
            std::process::exit(2);
        }
    };
    let state = match AppState::from_config(&config) {
        Ok(state) => state,
        Err(error) => {
            eprintln!("HTTP client error: {error}");
            std::process::exit(2);
        }
    };
    let address = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("server bind error: {error}");
            std::process::exit(2);
        }
    };

    tracing::info!(address = %address, oracle_url = %config.oracle_url, "arra-export listening");
    if let Err(error) = axum::serve(listener, app_router(state, config.frontend_dist)).await {
        eprintln!("server error: {error}");
        std::process::exit(1);
    }
}
