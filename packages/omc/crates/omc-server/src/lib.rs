pub mod account_service;
pub mod error;
pub mod model_service;
pub mod routes;
pub mod server_client;

use crate::account_service::AccountService;
use crate::model_service::ModelService;
use omc_core::config::OmcConfig;
use omc_storage::Storage;
use omc_storage::message_store::MessageStore;
use std::sync::Arc;
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::UnixListener;

pub struct DaemonState {
    config: tokio::sync::RwLock<OmcConfig>,
    pub storage: Arc<dyn Storage>,
    pub message_store: Arc<dyn MessageStore>,
    pub account_service: Arc<AccountService>,
    pub model_service: Arc<ModelService>,
}

impl DaemonState {
    pub fn new(
        config: OmcConfig,
        storage: Arc<dyn Storage>,
        message_store: Arc<dyn MessageStore>,
        account_service: Arc<AccountService>,
        model_service: Arc<ModelService>,
    ) -> Self {
        Self {
            config: tokio::sync::RwLock::new(config),
            storage,
            message_store,
            account_service,
            model_service,
        }
    }

    pub async fn config(&self) -> OmcConfig {
        self.config.read().await.clone()
    }
}

pub async fn start_server(
    daemon_state: Arc<DaemonState>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let resolved = daemon_state.config().await.resolve_daemon();

    let router = routes::create_router(daemon_state);

    let tcp_addr = format!("{}:{}", resolved.bind_addr, resolved.bind_port);
    let tcp_listener = TcpListener::bind(&tcp_addr).await?;
    tracing::info!("omcd listening on http://{}", tcp_addr);

    #[cfg(unix)]
    {
        let socket_path = resolved.socket_path.clone();
        if std::path::Path::new(&socket_path).exists() {
            std::fs::remove_file(&socket_path)?;
        }
        let unix_listener = UnixListener::bind(&socket_path)?;
        tracing::info!("omcd listening on unix://{}", socket_path);

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(());

        let mut unix_rx = shutdown_rx.clone();
        let app_clone = router.clone();
        let unix_handle = tokio::spawn(async move {
            axum::serve(unix_listener, app_clone)
                .with_graceful_shutdown(async move {
                    let _ = unix_rx.changed().await;
                })
                .await
                .ok();
        });

        let mut tcp_rx = shutdown_rx.clone();
        let tcp_handle = tokio::spawn(async move {
            axum::serve(tcp_listener, router)
                .with_graceful_shutdown(async move {
                    let _ = tcp_rx.changed().await;
                })
                .await
                .ok();
        });

        drop(shutdown_rx);

        shutdown_signal().await;
        drop(shutdown_tx);

        let _ = tokio::join!(tcp_handle, unix_handle);
    }

    #[cfg(not(unix))]
    {
        axum::serve(tcp_listener, router)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received");
}
