pub mod account;
pub mod config;
pub mod daemon;
pub mod model;
pub mod opencode;
pub mod self_cmd;
pub mod token_usage;
pub mod ui;

use std::path::PathBuf;

use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "omc",
    about = "oh-my-codes CLI",
    version,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long)]
    pub remote: Option<String>,

    #[arg(long, hide = true)]
    pub elevated: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    Account(AccountCommand),
    Model(ModelCommand),
    #[command(visible_alias = "tu")]
    TokenUsage(TokenUsageCommand),
    Opencode(OpencodeCommand),
    Config(ConfigCommand),
    Daemon(DaemonCommand),
    #[command(name = "self")]
    SelfCmd(SelfCmd),
    Health,
}

#[derive(Parser)]
pub struct DaemonCommand {
    #[command(subcommand)]
    pub action: DaemonAction,
}

#[derive(Subcommand)]
pub enum DaemonAction {
    Install {
        #[arg(long)]
        bin: Option<PathBuf>,
    },
    Uninstall,
    Start,
    Stop,
    Status,
}

#[derive(Parser)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    Show,
    Path,
}

#[derive(Parser)]
pub struct AccountCommand {
    #[command(subcommand)]
    pub action: AccountAction,
}

#[derive(Subcommand)]
pub enum AccountAction {
    Login { url: String },
    Logout { email: Option<String> },
    Switch,
    List,
    Show,
}

#[derive(Parser)]
pub struct ModelCommand {
    #[command(subcommand)]
    pub action: ModelAction,
}

#[derive(Subcommand)]
pub enum ModelAction {
    List {
        #[arg(long)]
        provider: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Sync {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Parser)]
pub struct TokenUsageCommand {
    #[command(subcommand)]
    pub action: Option<TokenUsageAction>,
}

#[derive(Subcommand)]
pub enum TokenUsageAction {
    Status {
        #[arg(long)]
        json: bool,
    },
    Push {
        #[arg(long)]
        json: bool,
    },
    List {
        #[arg(long, default_value_t = 20)]
        limit: usize,
        #[arg(long, default_value_t = 1)]
        page: usize,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        detail: bool,
        #[arg(long)]
        json: bool,
    },
    Summary {
        #[arg(long)]
        days: Option<i64>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Parser)]
pub struct OpencodeCommand {
    #[command(subcommand)]
    pub action: OpencodeAction,
}

#[derive(Subcommand)]
pub enum OpencodeAction {
    Install,
    Uninstall,
}

#[derive(Parser)]
#[command(name = "self")]
pub struct SelfCmd {
    #[command(subcommand)]
    pub action: SelfAction,
}

#[derive(Subcommand)]
pub enum SelfAction {
    #[command(alias = "update")]
    Upgrade {
        #[arg(long)]
        check: bool,
    },
}

impl Cli {
    pub fn print_help(&mut self) -> std::io::Result<()> {
        Cli::command().print_help()
    }
}
