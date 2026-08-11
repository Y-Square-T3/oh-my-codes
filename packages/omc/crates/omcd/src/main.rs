#[cfg(windows)]
mod service;

use clap::Parser;
use omc_core::config::OmcConfig;
use omc_server::DaemonState;
use omc_server::account_service::AccountService;
use omc_server::model_service::ModelService;
use omc_server::server_client::OmcServerClient;
use omc_server::token_usage_service::TokenUsageService;
use omc_storage::create_backend;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::watch;

#[derive(Parser, Clone)]
#[command(name = "omcd", about = "oh-my-codes daemon", version)]
pub(crate) struct Args {
    #[arg(long)]
    pub(crate) config: Option<PathBuf>,

    #[arg(long)]
    pub(crate) data_dir: Option<String>,

    #[arg(long)]
    pub(crate) database_url: Option<String>,

    #[arg(long)]
    pub(crate) bind_addr: Option<String>,

    #[arg(long)]
    pub(crate) bind_port: Option<u16>,

    #[arg(long)]
    pub(crate) socket_path: Option<String>,

    #[cfg(windows)]
    #[arg(long)]
    pub(crate) service: bool,
}

struct PidFile(PathBuf);

impl PidFile {
    fn new(path: &str) -> Self {
        let p = PathBuf::from(path);
        if let Some(parent) = p.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let pid = std::process::id();
        let _ = std::fs::write(&p, pid.to_string());
        Self(p)
    }
}

impl Drop for PidFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    #[cfg(windows)]
    {
        if args.service {
            return service::run().map_err(|e| format!("Service error: {e}").into());
        }
    }

    init_tracing();
    run_daemon(args, None, false).await
}

pub(crate) async fn run_daemon(
    args: Args,
    external_shutdown: Option<watch::Receiver<()>>,
    service_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = OmcConfig::load(args.config.as_deref(), service_mode)?;

    let overrides = OmcConfig {
        daemon: omc_core::config::DaemonConfig {
            bind_addr: args.bind_addr,
            bind_port: args.bind_port,
            socket_path: args.socket_path,
            data_dir: args.data_dir,
            database_url: args.database_url,
        },
    };
    config.merge(&overrides);

    let resolved = config.resolve_daemon_with_mode(service_mode);

    let data_path = PathBuf::from(&resolved.data_dir);
    std::fs::create_dir_all(&data_path)?;

    let db_url = omc_storage::database_url::DatabaseUrl::parse(&resolved.database_url);
    let backend = create_backend(&db_url).await?;

    let server_client = OmcServerClient::new();
    let account_service = Arc::new(AccountService::new(backend.clone(), server_client.clone()));
    let model_service = Arc::new(ModelService::new(
        backend.clone(),
        account_service.clone(),
        server_client.clone(),
    ));
    let token_usage_service = Arc::new(TokenUsageService::new(
        backend.clone(),
        account_service.clone(),
        server_client,
    ));

    let state = Arc::new(DaemonState::new(
        config,
        backend,
        account_service,
        model_service,
        token_usage_service.clone(),
    ));

    let auto_push_stop = token_usage_service.start_auto_push(300, 20);

    let pid_path = omc_core::config::paths::default_pid_path_with_mode(service_mode);
    let _pid_file = PidFile::new(&pid_path);

    tracing::info!("oh-my-codes daemon starting");
    tracing::info!("Data directory: {}", resolved.data_dir);

    let shutdown_rx = match external_shutdown {
        Some(rx) => rx,
        None => {
            let (shutdown_tx, shutdown_rx) = watch::channel(());
            tokio::spawn(async move {
                shutdown_signal().await;
                let _ = shutdown_tx.send(());
            });
            shutdown_rx
        }
    };

    omc_server::start_server(state, shutdown_rx).await?;

    auto_push_stop.notify_waiters();

    Ok(())
}

pub(crate) fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
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
