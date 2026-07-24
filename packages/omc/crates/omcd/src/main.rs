use clap::Parser;
use omc_core::config::OmcConfig;
use omc_server::DaemonState;
use omc_storage::memory::MemoryStorage;
use omc_storage::surreal::SurrealStorage;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "omcd", about = "oh-my-codes daemon")]
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

    #[arg(long)]
    auth_token: Option<String>,
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
            auth_token: args.auth_token,
        },
        repos: Vec::new(),
    };
    config.merge(&overrides);

    let resolved = config.resolve_daemon();

    let storage = Arc::new(MemoryStorage::new());
    let state = Arc::new(DaemonState::new(config, storage));

    let data_path = PathBuf::from(&resolved.data_dir);
    std::fs::create_dir_all(&data_path)?;
    let surreal = SurrealStorage::new_rocksdb(&data_path.join("omc.db")).await?;
    let surreal_store: Arc<dyn omc_storage::message_store::MessageStore> = Arc::new(surreal);
    state
        .add_message_store("default".to_string(), surreal_store)
        .await;

    let pid_path = omc_core::config::paths::default_pid_path();
    let _pid_file = PidFile::new(&pid_path);

    tracing::info!("oh-my-codes daemon starting");
    tracing::info!("Data directory: {}", resolved.data_dir);
    tracing::info!("Listening on {}:{}", resolved.bind_addr, resolved.bind_port);

    omc_server::start_server(state).await?;

    Ok(())
}
