use clap::{Parser, Subcommand};
use omc_api::client::OmcClient;
use omc_core::config::OmcConfig;
use omc_service::{create_service_manager, find_omcd_binary};

#[derive(Parser)]
#[command(name = "omc", about = "oh-my-codes CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(long)]
    remote: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Daemon(DaemonCommand),
    Config(ConfigCommand),
    Repo(RepoCommand),
    Health,
}

#[derive(Parser)]
struct DaemonCommand {
    #[command(subcommand)]
    action: DaemonAction,
}

#[derive(Subcommand)]
enum DaemonAction {
    Install,
    Uninstall,
    Start,
    Stop,
    Status,
}

#[derive(Parser)]
struct ConfigCommand {
    #[command(subcommand)]
    action: ConfigAction,
}

#[derive(Subcommand)]
enum ConfigAction {
    Show,
    Path,
}

#[derive(Parser)]
struct RepoCommand {
    #[command(subcommand)]
    action: RepoAction,
}

#[derive(Subcommand)]
enum RepoAction {
    Add { path: String },
    List,
    Remove { path: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = OmcConfig::load(None)?;
    let resolved = config.resolve_daemon();

    let client = if let Some(ref remote) = cli.remote {
        OmcClient::connect_http(remote)
    } else {
        #[cfg(unix)]
        {
            OmcClient::connect_unix(&resolved.socket_path)
        }
        #[cfg(not(unix))]
        {
            OmcClient::connect_http(&format!(
                "http://{}:{}",
                resolved.bind_addr, resolved.bind_port
            ))
        }
    };

    match cli.command {
        None => {
            omc_tui::run(client).await?;
        }
        Some(Commands::Health) => {
            let resp = client.health().await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        Some(Commands::Config(cmd)) => match cmd.action {
            ConfigAction::Show => {
                let resp = client.config().await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            ConfigAction::Path => {
                let resp = client.config_path().await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        },
        Some(Commands::Repo(cmd)) => match cmd.action {
            RepoAction::Add { path } => {
                let resp = client.repo_add(&path).await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            RepoAction::List => {
                let resp = client.repo_list().await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
            RepoAction::Remove { path } => {
                let resp = client.repo_remove(&path).await?;
                println!("{}", serde_json::to_string_pretty(&resp)?);
            }
        },
        Some(Commands::Daemon(cmd)) => {
            let manager = create_service_manager();
            match cmd.action {
                DaemonAction::Install => {
                    let binary_path = find_omcd_binary().map_err(|e| e.to_string())?;
                    let config = omc_service::ServiceConfig { binary_path };
                    manager.install(&config).map_err(|e| e.to_string())?;
                    println!("Daemon installed successfully");
                }
                DaemonAction::Uninstall => {
                    manager.uninstall().map_err(|e| e.to_string())?;
                    println!("Daemon uninstalled successfully");
                }
                DaemonAction::Start => {
                    manager.start().map_err(|e| e.to_string())?;
                    println!("Daemon started");
                }
                DaemonAction::Stop => {
                    manager.stop().map_err(|e| e.to_string())?;
                    println!("Daemon stopped");
                }
                DaemonAction::Status => {
                    let status = manager.status().map_err(|e| e.to_string())?;
                    match status {
                        omc_service::ServiceStatus::Running { pid } => {
                            println!("Daemon is running (pid: {:?})", pid)
                        }
                        omc_service::ServiceStatus::Stopped => println!("Daemon is stopped"),
                        omc_service::ServiceStatus::NotInstalled => {
                            println!("Daemon is not installed")
                        }
                        omc_service::ServiceStatus::Unknown(s) => println!("Daemon status: {s}"),
                    }
                }
            }
        }
    }

    Ok(())
}
