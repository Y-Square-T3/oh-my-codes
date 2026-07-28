mod cli;

use clap::Parser;
use cli::{Cli, Commands};
use omc_api::client::OmcClient;
use omc_core::config::OmcConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = Cli::parse();

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
            cli.print_help()?;
            println!();
        }
        Some(Commands::Health) => {
            let resp = client.health().await?;
            println!("{}", serde_json::to_string_pretty(&resp)?);
        }
        Some(Commands::Config(cmd)) => cli::config::run(&client, cmd).await?,
        Some(Commands::Account(cmd)) => cli::account::run(&client, cmd).await?,
        Some(Commands::Model(cmd)) => cli::model::run(&client, cmd).await?,
        Some(Commands::TokenUsage(cmd)) => cli::token_usage::run(&client, cmd).await?,
        Some(Commands::Daemon(cmd)) => cli::daemon::run(cmd)?,
        Some(Commands::Opencode(cmd)) => cli::opencode::run(cmd)?,
    }

    Ok(())
}
