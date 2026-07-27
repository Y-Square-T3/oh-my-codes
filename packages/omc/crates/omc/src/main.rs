use clap::{CommandFactory, Parser, Subcommand};
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};
use omc_api::client::OmcClient;
use omc_api::types::{PollRequest, PollResponse};
use omc_core::config::OmcConfig;
use omc_service::{create_service_manager, find_omcd_binary};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "omc", about = "oh-my-codes CLI", version)]
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
    Account(AccountCommand),
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
struct AccountCommand {
    #[command(subcommand)]
    action: AccountAction,
}

#[derive(Subcommand)]
enum AccountAction {
    Login { url: String },
    Logout { email: Option<String> },
    Switch,
    List,
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
            let mut cmd = Cli::command();
            cmd.print_help()?;
            println!();
            return Ok(());
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
        Some(Commands::Account(cmd)) => match cmd.action {
            AccountAction::Login { url } => {
                login_effect(&client, &url).await?;
            }
            AccountAction::Logout { email } => {
                logout_effect(&client, email.as_deref()).await?;
            }
            AccountAction::Switch => {
                switch_effect(&client).await?;
            }
            AccountAction::List => {
                list_effect(&client).await?;
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
                        omc_service::ServiceStatus::Running { pid } => match pid {
                            Some(p) => println!("Daemon is running (pid: {p})"),
                            None => println!("Daemon is running"),
                        },
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

async fn login_effect(client: &OmcClient, url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let resp = client.account_login(url).await?;

    let full_url = format!("{}{}", url.trim_end_matches('/'), resp.verification_uri_complete);

    println!();
    println!("Open this URL in your browser:");
    println!("  {}", full_url);
    println!();
    println!("Or enter code: {}", resp.user_code);
    println!();

    let _ = open::that(&full_url);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );
    pb.set_message("Waiting for authorization...");

    let mut interval = Duration::from_secs(resp.interval as u64);
    loop {
        tokio::time::sleep(interval).await;

        let poll_req = PollRequest {
            device_code: resp.device_code.clone(),
            server_url: url.to_string(),
            expires_at: resp.expires_at,
            interval: resp.interval,
        };
        let poll_resp = client.account_poll(&poll_req).await?;

        match poll_resp {
            PollResponse::Success { email } => {
                pb.finish_with_message(format!("Logged in as {email}"));
                println!();
                println!("Login successful!");
                return Ok(());
            }
            PollResponse::Pending => {
                pb.set_message("Waiting for authorization...");
            }
            PollResponse::Slow => {
                interval += Duration::from_secs(5);
                pb.set_message("Slowing down...");
            }
            PollResponse::Expired => {
                pb.finish_with_message("Login expired");
                return Err("Device code expired. Please try again.".into());
            }
            PollResponse::Denied => {
                pb.finish_with_message("Login denied");
                return Err("Authorization was denied.".into());
            }
            PollResponse::Error { message } => {
                pb.finish_with_message(format!("Error: {message}"));
                return Err(format!("Login error: {message}").into());
            }
        }
    }
}

async fn logout_effect(
    client: &OmcClient,
    email: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let list = client.account_list().await?;

    if list.accounts.is_empty() {
        println!("No accounts found.");
        return Ok(());
    }

    let account_id = if let Some(email) = email {
        match list.accounts.iter().find(|a| a.account.email == email) {
            Some(a) => a.account.id.clone(),
            None => {
                println!("Account with email '{email}' not found.");
                return Ok(());
            }
        }
    } else {
        let items: Vec<String> = list
            .accounts
            .iter()
            .map(|a| {
                let active = if a.account.active_workspace_id.is_some() {
                    " (active)"
                } else {
                    ""
                };
                format!("{}{}", a.account.email, active)
            })
            .collect();
        let selection = Select::new()
            .with_prompt("Select account to remove")
            .items(&items)
            .interact()?;
        list.accounts[selection].account.id.clone()
    };

    client.account_remove(&account_id).await?;
    println!("Account removed.");
    Ok(())
}

async fn switch_effect(client: &OmcClient) -> Result<(), Box<dyn std::error::Error>> {
    let list = client.account_list().await?;

    if list.accounts.is_empty() {
        println!("No accounts found. Run 'omc account login <url>' first.");
        return Ok(());
    }

    let mut items: Vec<String> = Vec::new();
    let mut keys: Vec<(String, String)> = Vec::new();

    for aw in &list.accounts {
        for ws in &aw.workspaces {
            let active = aw.account.active_workspace_id.as_deref() == Some(&ws.id);
            let marker = if active { "* " } else { "  " };
            items.push(format!("{}{} / {}", marker, aw.account.email, ws.name));
            keys.push((aw.account.id.clone(), ws.id.clone()));
        }
    }

    if items.is_empty() {
        println!("No workspaces found.");
        return Ok(());
    }

    let selection = Select::new()
        .with_prompt("Select workspace")
        .items(&items)
        .interact()?;

    let (account_id, workspace_id) = &keys[selection];
    client.account_switch(account_id, workspace_id).await?;
    println!("Switched workspace.");
    Ok(())
}

async fn list_effect(client: &OmcClient) -> Result<(), Box<dyn std::error::Error>> {
    let list = client.account_list().await?;

    if list.accounts.is_empty() {
        println!("No accounts found.");
        return Ok(());
    }

    let active = client.account_active().await?;
    let active_id = active.account.as_ref().map(|a| a.id.as_str());

    for aw in &list.accounts {
        let a = &aw.account;
        let marker = if active_id == Some(&a.id) { "*" } else { " " };
        println!("{marker} {} ({})", a.email, a.url);
        for ws in &aw.workspaces {
            let ws_marker = if a.active_workspace_id.as_deref() == Some(&ws.id) {
                "  * "
            } else {
                "    "
            };
            println!("{}{}", ws_marker, ws.name);
        }
        println!();
    }

    Ok(())
}
