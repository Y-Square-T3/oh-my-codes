pub mod account_service;
pub mod error;
pub mod model_service;
pub mod routes;
pub mod server_client;
pub mod token_usage_service;

pub use crate::account_service::AccountService;
pub use crate::model_service::ModelService;
pub use crate::token_usage_service::TokenUsageService;
use omc_core::config::OmcConfig;
use omc_core::error::OmcError;
use omc_storage::StorageBackend;
use std::sync::Arc;
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::UnixListener;
use tokio::sync::watch;

pub struct DaemonState {
    config: tokio::sync::RwLock<OmcConfig>,
    pub backend: Arc<dyn StorageBackend>,
    pub account_service: Arc<AccountService>,
    pub model_service: Arc<ModelService>,
    pub token_usage_service: Arc<TokenUsageService>,
}

impl DaemonState {
    pub fn new(
        config: OmcConfig,
        backend: Arc<dyn StorageBackend>,
        account_service: Arc<AccountService>,
        model_service: Arc<ModelService>,
        token_usage_service: Arc<TokenUsageService>,
    ) -> Self {
        Self {
            config: tokio::sync::RwLock::new(config),
            backend,
            account_service,
            model_service,
            token_usage_service,
        }
    }

    pub async fn config(&self) -> OmcConfig {
        self.config.read().await.clone()
    }
}

pub async fn start_server(
    daemon_state: Arc<DaemonState>,
    #[allow(unused_mut)] mut shutdown_rx: watch::Receiver<()>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let resolved = daemon_state.config().await.resolve_daemon();

    let router = routes::create_router(daemon_state);

    let tcp_addr = format!("{}:{}", resolved.bind_addr, resolved.bind_port);
    let tcp_listener = match TcpListener::bind(&tcp_addr).await {
        Ok(listener) => listener,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            let config_path = omc_core::config::paths::default_config_path();
            return Err(Box::new(OmcError::PortInUse {
                address: tcp_addr,
                config_path: config_path.to_string_lossy().to_string(),
            }));
        }
        Err(e) => return Err(Box::new(e)),
    };
    tracing::info!("omcd listening on http://{}", tcp_addr);

    #[cfg(unix)]
    {
        let socket_path = resolved.socket_path.clone();
        if std::path::Path::new(&socket_path).exists() {
            std::fs::remove_file(&socket_path)?;
        }
        let unix_listener = UnixListener::bind(&socket_path)?;
        tracing::info!("omcd listening on unix://{}", socket_path);

        let mut unix_rx = shutdown_rx.clone();
        let app_clone = router.clone();
        let unix_handle = tokio::spawn(async move {
            axum::serve(unix_listener, app_clone)
                .with_graceful_shutdown(async move {
                    let _ = unix_rx.changed().await;
                })
                .await
        });

        let mut tcp_rx = shutdown_rx.clone();
        let app_clone = router.clone();
        let tcp_handle = tokio::spawn(async move {
            axum::serve(tcp_listener, app_clone)
                .with_graceful_shutdown(async move {
                    let _ = tcp_rx.changed().await;
                })
                .await
        });

        let _ = tokio::join!(unix_handle, tcp_handle);
    }

    #[cfg(not(unix))]
    {
        axum::serve(tcp_listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.changed().await;
            })
            .await?;
    }

    tracing::info!("omcd shutting down");
    Ok(())
}
