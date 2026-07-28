use clap::Parser;
use omc_core::config::OmcConfig;
use omc_server::DaemonState;
use omc_server::account_service::AccountService;
use omc_server::model_service::ModelService;
use omc_server::server_client::OmcServerClient;
use omc_server::token_usage_service::TokenUsageService;
use omc_storage::account_store::AccountStore;
use omc_storage::memory::MemoryStorage;
use omc_storage::surreal::{
    SurrealAccountStore, SurrealModelStore, SurrealStorage, SurrealTokenUsageStore,
    SurrealWorkspaceStore,
};
use omc_storage::token_usage_store::TokenUsageStore;
use omc_storage::workspace_store::WorkspaceStore;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "omcd", about = "oh-my-codes daemon", version)]
struct Args {
    #[arg(long)]
    config: Option<PathBuf>,

    #[arg(long)]
    data_dir: Option<String>,

    #[arg(long)]
    bind_addr: Option<String>,

    #[arg(long)]
    bind_port: Option<u16>,

    #[arg(long)]
    socket_path: Option<String>,
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    let mut config = OmcConfig::load(args.config.as_deref())?;

    let overrides = OmcConfig {
        daemon: omc_core::config::DaemonConfig {
            bind_addr: args.bind_addr,
            bind_port: args.bind_port,
            socket_path: args.socket_path,
            data_dir: args.data_dir,
        },
    };
    config.merge(&overrides);

    let resolved = config.resolve_daemon();

    let data_path = PathBuf::from(&resolved.data_dir);
    std::fs::create_dir_all(&data_path)?;
    let surreal = SurrealStorage::new_rocksdb(&data_path.join("omc.db")).await?;
    let db = surreal.db();
    let message_store: Arc<dyn omc_storage::message_store::MessageStore> = Arc::new(surreal);
    let storage = Arc::new(MemoryStorage::new());

    let account_store: Arc<dyn AccountStore> = Arc::new(SurrealAccountStore::new(db.clone()));
    let workspace_store: Arc<dyn WorkspaceStore> = Arc::new(SurrealWorkspaceStore::new(db.clone()));
    let model_store: Arc<dyn omc_storage::model_store::ModelStore> =
        Arc::new(SurrealModelStore::new(db.clone()));
    let token_usage_store: Arc<dyn TokenUsageStore> = Arc::new(SurrealTokenUsageStore::new(db));
    let server_client = OmcServerClient::new();
    let account_service = Arc::new(AccountService::new(
        account_store,
        workspace_store,
        server_client.clone(),
    ));
    let model_service = Arc::new(ModelService::new(
        model_store,
        account_service.clone(),
        server_client.clone(),
    ));
    let token_usage_service = Arc::new(TokenUsageService::new(
        token_usage_store,
        account_service.clone(),
        server_client,
    ));

    let state = Arc::new(DaemonState::new(
        config,
        storage,
        message_store,
        account_service,
        model_service,
        token_usage_service.clone(),
    ));

    let _auto_push_stop = token_usage_service.start_auto_push(30, 20);

    let pid_path = omc_core::config::paths::default_pid_path();
    let _pid_file = PidFile::new(&pid_path);

    tracing::info!("oh-my-codes daemon starting");
    tracing::info!("Data directory: {}", resolved.data_dir);
    tracing::info!("Listening on {}:{}", resolved.bind_addr, resolved.bind_port);

    omc_server::start_server(state).await?;

    Ok(())
}
