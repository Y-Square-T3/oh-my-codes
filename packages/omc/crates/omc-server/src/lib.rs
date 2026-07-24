pub mod middleware;
pub mod routes;

use omc_core::config::OmcConfig;
use omc_core::error::Result;
use omc_storage::Storage;
use omc_storage::message_store::MessageStore;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DaemonState {
    config: tokio::sync::RwLock<OmcConfig>,
    pub storage: Arc<dyn Storage>,
    message_stores: tokio::sync::RwLock<HashMap<String, Arc<dyn MessageStore>>>,
}

impl DaemonState {
    pub fn new(config: OmcConfig, storage: Arc<dyn Storage>) -> Self {
        Self {
            config: tokio::sync::RwLock::new(config),
            storage,
            message_stores: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    pub async fn message_store_for(&self, repo_path: &str) -> Option<Arc<dyn MessageStore>> {
        let stores = self.message_stores.read().await;
        stores.get(repo_path).cloned()
    }

    pub async fn add_message_store(&self, repo_path: String, store: Arc<dyn MessageStore>) {
        let mut stores = self.message_stores.write().await;
        stores.insert(repo_path, store);
    }

    pub async fn remove_message_store(&self, repo_path: &str) {
        let mut stores = self.message_stores.write().await;
        stores.remove(repo_path);
    }

    pub async fn config(&self) -> OmcConfig {
        self.config.read().await.clone()
    }

    pub async fn add_repo(&self, path: String) -> Result<()> {
        let mut config = self.config.write().await;
        if !config.repos.iter().any(|r| r.path == path) {
            config.repos.push(omc_core::config::RepoConfig { path });
        }
        Ok(())
    }

    pub async fn remove_repo(&self, path: String) -> Result<()> {
        let mut config = self.config.write().await;
        config.repos.retain(|r| r.path != path);
        drop(config);
        self.remove_message_store(&path).await;
        Ok(())
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
