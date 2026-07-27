pub mod account_service;
pub mod middleware;
pub mod routes;
pub mod server_client;

use crate::account_service::AccountService;
use omc_core::config::OmcConfig;
use omc_storage::Storage;
use omc_storage::message_store::MessageStore;
use std::sync::Arc;

pub struct DaemonState {
    config: tokio::sync::RwLock<OmcConfig>,
    pub storage: Arc<dyn Storage>,
    pub message_store: Arc<dyn MessageStore>,
    pub account_service: Arc<AccountService>,
}

impl DaemonState {
    pub fn new(
        config: OmcConfig,
        storage: Arc<dyn Storage>,
        message_store: Arc<dyn MessageStore>,
        account_service: Arc<AccountService>,
    ) -> Self {
        Self {
            config: tokio::sync::RwLock::new(config),
            storage,
            message_store,
            account_service,
        }
    }

    pub async fn config(&self) -> OmcConfig {
        self.config.read().await.clone()
    }
}

pub async fn start_server(
    daemon_state: Arc<DaemonState>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = daemon_state.config().await;
    let resolved = config.resolve_daemon();

    let router = routes::create_router(resolved.auth_token.clone(), daemon_state);

    let addr = format!("{}:{}", resolved.bind_addr, resolved.bind_port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
