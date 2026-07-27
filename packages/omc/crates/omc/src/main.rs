use clap::{CommandFactory, Parser, Subcommand};
use comfy_table::{Cell, CellAlignment, Color as TColor, Table};
use console::style;
use dialoguer::{Confirm, Select};
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
    Show,
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
            AccountAction::Show => {
                show_effect(&client).await?;
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

    let full_url = format!(
        "{}{}",
        url.trim_end_matches('/'),
        resp.verification_uri_complete
    );

    println!();
    println!(
        "  {} {}",
        style("Open this URL in your browser:").bold(),
        style(&full_url).cyan().underlined()
    );
    println!();
    println!(
        "  {} {}",
        style("Or enter code:"),
        style(&resp.user_code).green().bold()
    );
    println!();

    let _ = open::that(&full_url);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );
    pb.set_message("Waiting for authorization...");

    let mut interval = Duration::from_secs(resp.interval as u64).min(Duration::from_secs(2));
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
                pb.finish_and_clear();
                println!(
                    "  {} {} {}",
                    style("✓").green().bold(),
                    style("Logged in as").bold(),
                    style(&email).cyan()
                );

                let list = client.account_list().await?;
                if let Some(account) = list.accounts.iter().find(|a| a.account.email == email) {
                    if account.workspaces.len() > 1 {
                        println!();
                        println!(
                            "  {} {}",
                            style("Select workspace:").bold(),
                            style(format!("({} available)", account.workspaces.len())).dim()
                        );
                        println!();

                        let mut items: Vec<String> = Vec::new();
                        let mut keys: Vec<String> = Vec::new();

                        for ws in &account.workspaces {
                            let admin_badge = if ws.is_admin {
                                format!(" {}", style("admin").yellow())
                            } else {
                                String::new()
                            };
                            items.push(format!("{}{}", style(&ws.name).bold(), admin_badge));
                            keys.push(ws.id.clone());
                        }

                        let selection = Select::new()
                            .with_prompt("Choose workspace")
                            .items(&items)
                            .default(0)
                            .interact()?;

                        let workspace_id = &keys[selection];
                        client
                            .account_switch(&account.account.id, workspace_id)
                            .await?;

                        let selected_ws = &account.workspaces[selection];
                        println!(
                            "  {} {} {}",
                            style("✓").green().bold(),
                            style("Workspace set to"),
                            style(&selected_ws.name).cyan().bold()
                        );
                    } else if account.workspaces.len() == 1 {
                        let ws = &account.workspaces[0];
                        println!(
                            "  {} {} {}",
                            style("✓").green().bold(),
                            style("Workspace set to"),
                            style(&ws.name).cyan().bold()
                        );
                    }
                }

                println!();
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
                pb.finish_and_clear();
                println!(
                    "  {} {}",
                    style("✗").red().bold(),
                    style("Login expired. Please try again.").red()
                );
                return Err("Device code expired.".into());
            }
            PollResponse::Denied => {
                pb.finish_and_clear();
                println!(
                    "  {} {}",
                    style("✗").red().bold(),
                    style("Authorization was denied.").red()
                );
                return Err("Authorization was denied.".into());
            }
            PollResponse::Error { message } => {
                pb.finish_and_clear();
                println!(
                    "  {} {}",
                    style("✗").red().bold(),
                    style(format!("Error: {message}")).red()
                );
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
        println!(
            "  {} {}",
            style("!").yellow().bold(),
            style("No accounts found.").yellow()
        );
        return Ok(());
    }

    let account_id = if let Some(email) = email {
        match list.accounts.iter().find(|a| a.account.email == email) {
            Some(a) => a.account.id.clone(),
            None => {
                println!(
                    "  {} {}",
                    style("✗").red().bold(),
                    style(format!("Account with email '{email}' not found.")).red()
                );
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

    let account_email = list
        .accounts
        .iter()
        .find(|a| a.account.id == account_id)
        .map(|a| a.account.email.as_str())
        .unwrap_or("unknown");

    let confirmed = Confirm::new()
        .with_prompt(format!("Remove account {}?", style(account_email).cyan()))
        .default(false)
        .interact()?;

    if !confirmed {
        println!("  {}", style("Cancelled.").dim());
        return Ok(());
    }

    client.account_remove(&account_id).await?;
    println!(
        "  {} {} {}",
        style("✓").green().bold(),
        style("Account removed:"),
        style(account_email).cyan()
    );
    Ok(())
}

async fn switch_effect(client: &OmcClient) -> Result<(), Box<dyn std::error::Error>> {
    let list = client.account_list().await?;

    if list.accounts.is_empty() {
        println!(
            "  {} {}",
            style("!").yellow().bold(),
            style("No accounts found. Run 'omc account login <url>' first.").yellow()
        );
        return Ok(());
    }

    let mut items: Vec<String> = Vec::new();
    let mut keys: Vec<(String, String)> = Vec::new();

    for aw in &list.accounts {
        for ws in &aw.workspaces {
            let active = aw.account.active_workspace_id.as_deref() == Some(ws.id.as_str());
            items.push(format!(
                "{} {} / {} {}",
                if active { "▶" } else { " " },
                style(&aw.account.email).cyan(),
                style(&ws.name).bold(),
                if ws.is_admin {
                    style("admin").yellow().to_string()
                } else {
                    String::new()
                }
            ));
            keys.push((aw.account.id.clone(), ws.id.clone()));
        }
    }

    if items.is_empty() {
        println!(
            "  {} {}",
            style("!").yellow().bold(),
            style("No workspaces found.").yellow()
        );
        return Ok(());
    }

    let selection = Select::new()
        .with_prompt("Select workspace")
        .items(&items)
        .default(0)
        .interact()?;

    let (account_id, workspace_id) = &keys[selection];
    client.account_switch(account_id, workspace_id).await?;

    let selected_account = list
        .accounts
        .iter()
        .find(|aw| aw.account.id == *account_id)
        .unwrap();
    let selected_ws = selected_account
        .workspaces
        .iter()
        .find(|ws| ws.id == *workspace_id)
        .unwrap();

    println!(
        "  {} {} {} / {}",
        style("✓").green().bold(),
        style("Switched to"),
        style(&selected_account.account.email).cyan(),
        style(&selected_ws.name).bold()
    );
    if selected_ws.is_admin {
        println!(
            "    {} {}",
            style("★").yellow(),
            style("Admin access").yellow().dim()
        );
    }

    Ok(())
}

async fn list_effect(client: &OmcClient) -> Result<(), Box<dyn std::error::Error>> {
    let list = client.account_list().await?;

    if list.accounts.is_empty() {
        println!(
            "  {} {}",
            style("!").yellow().bold(),
            style("No accounts found.").yellow()
        );
        return Ok(());
    }

    let active = client.account_active().await?;
    let active_id = active.account.as_ref().map(|a| a.id.as_str());

    let mut table = Table::new();
    table
        .set_header(vec![
            Cell::new("").set_alignment(CellAlignment::Center),
            Cell::new("Account"),
            Cell::new("Server"),
            Cell::new("Workspaces"),
        ])
        .set_width(80);

    for aw in &list.accounts {
        let a = &aw.account;
        let is_active = active_id == Some(&a.id);

        let status_cell = if is_active {
            Cell::new("●").fg(TColor::Green)
        } else {
            Cell::new("○").fg(TColor::DarkGrey)
        };

        let email_cell = if is_active {
            Cell::new(&a.email).fg(TColor::Cyan)
        } else {
            Cell::new(&a.email)
        };

        let server_cell = Cell::new(&a.url).fg(TColor::DarkGrey);

        let ws_display: Vec<String> = aw
            .workspaces
            .iter()
            .map(|ws| {
                let is_ws_active = a.active_workspace_id.as_deref() == Some(ws.id.as_str());
                let admin_badge = if ws.is_admin { " ★" } else { "" };
                if is_ws_active {
                    format!("▶ {}{}", ws.name, admin_badge)
                } else {
                    format!("  {}{}", ws.name, admin_badge)
                }
            })
            .collect();
        let ws_cell = Cell::new(ws_display.join("\n"));

        table.add_row(vec![status_cell, email_cell, server_cell, ws_cell]);
    }

    println!();
    println!("{table}");
    println!("  {} {}", style("●").green(), style("active").dim());
    println!("  {} {}", style("★").yellow(), style("admin").dim());
    println!();

    Ok(())
}

async fn show_effect(client: &OmcClient) -> Result<(), Box<dyn std::error::Error>> {
    let active = client.account_active().await?;

    let Some(account) = active.account else {
        println!(
            "  {} {}",
            style("!").yellow().bold(),
            style("No active account. Run 'omc account login <url>' first.").yellow()
        );
        return Ok(());
    };

    let list = client.account_list().await?;
    let account_with_ws = list.accounts.iter().find(|aw| aw.account.id == account.id);

    let active_ws = account_with_ws.and_then(|aw| {
        aw.workspaces
            .iter()
            .find(|ws| aw.account.active_workspace_id.as_deref() == Some(ws.id.as_str()))
    });

    println!();
    println!("  {}", style("Active Account").bold().underlined());
    println!();
    println!(
        "  {} {}",
        style("Email:").dim(),
        style(&account.email).cyan().bold()
    );
    println!(
        "  {} {}",
        style("Server:").dim(),
        style(&account.url).cyan()
    );

    if let Some(ws) = active_ws {
        let admin_badge = if ws.is_admin {
            style(" (admin)").yellow().to_string()
        } else {
            String::new()
        };
        println!(
            "  {} {}{}",
            style("Workspace:").dim(),
            style(&ws.name).cyan(),
            admin_badge
        );
    } else {
        println!("  {} {}", style("Workspace:").dim(), style("none").dim());
    }

    println!();
    Ok(())
}
