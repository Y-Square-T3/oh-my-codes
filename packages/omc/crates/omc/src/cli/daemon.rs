use super::{DaemonAction, DaemonCommand};
use omc_service::{create_service_manager, find_omcd_binary};

pub fn run(cmd: DaemonCommand) -> Result<(), Box<dyn std::error::Error>> {
    let manager = create_service_manager();
    match cmd.action {
        DaemonAction::Install { bin } => {
            let binary_path = match bin {
                Some(p) => p,
                None => find_omcd_binary().map_err(|e| e.to_string())?,
            };
            let data_dir = omc_core::config::paths::default_data_dir()
                .to_string_lossy()
                .into_owned();
            let config_path = dirs::config_dir()
                .map(|d| d.join("omc").join("omc.json"))
                .and_then(|p| p.exists().then(|| p.to_string_lossy().into_owned()));
            let config = omc_service::ServiceConfig {
                binary_path,
                data_dir: Some(data_dir),
                config: config_path,
            };
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
    Ok(())
}
