use crate::{Result, ServiceConfig, ServiceError, ServiceManager, ServiceStatus};
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{debug, warn};
use windows_service::service::{
    ServiceAccess, ServiceAction, ServiceActionType, ServiceErrorControl, ServiceFailureActions,
    ServiceFailureResetPeriod, ServiceInfo, ServiceStartType, ServiceState, ServiceType,
};
use windows_service::service_manager::{ServiceManager as Scm, ServiceManagerAccess};

const SERVICE_NAME: &str = "omcd";
const SERVICE_DISPLAY_NAME: &str = "OMC Daemon";

fn quote_path_if_needed(path: &std::path::Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if path_str.contains(' ') && !path_str.starts_with('"') {
        PathBuf::from(format!("\"{}\"", path_str))
    } else {
        path.to_path_buf()
    }
}

fn quote_string_if_needed(s: &str) -> OsString {
    if s.contains(' ') && !s.starts_with('"') {
        OsString::from(format!("\"{}\"", s))
    } else {
        OsString::from(s)
    }
}

fn format_windows_service_error(context: &str, e: &windows_service::Error) -> String {
    if let windows_service::Error::Winapi(io_err) = e {
        let code = io_err.raw_os_error();
        let mut msg = format!("{context}: {io_err}");
        if code == Some(5) {
            msg.push_str(
                ". Administrator privileges are required; run the command as Administrator",
            );
        }
        msg
    } else {
        format!("{context}: {e}")
    }
}
pub struct WindowsServiceManager;

impl Default for WindowsServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowsServiceManager {
    pub fn new() -> Self {
        Self
    }

    fn open_service_with_access(
        &self,
        access: ServiceAccess,
    ) -> Result<windows_service::service::Service> {
        debug!("Connecting to Service Control Manager");
        let manager_access = ServiceManagerAccess::CONNECT;
        let manager = Scm::local_computer(None::<&str>, manager_access).map_err(|e| {
            let err_msg = format_windows_service_error("Failed to connect to SCM", &e);
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;
        debug!(
            "Opening service '{}' with access {:?}",
            SERVICE_NAME, access
        );
        manager.open_service(SERVICE_NAME, access).map_err(|e| {
            if is_service_not_found_error(&e) {
                debug!("Service '{}' is not installed", SERVICE_NAME);
                ServiceError::NotInstalled
            } else {
                let err_msg = format_windows_service_error(
                    &format!("Failed to open service '{}'", SERVICE_NAME),
                    &e,
                );
                debug!("{err_msg}");
                ServiceError::Other(err_msg)
            }
        })
    }

    fn open_service(&self) -> Result<windows_service::service::Service> {
        self.open_service_with_access(
            ServiceAccess::QUERY_STATUS | ServiceAccess::START | ServiceAccess::STOP,
        )
    }
}

impl ServiceManager for WindowsServiceManager {
    fn install(&self, config: &ServiceConfig) -> Result<()> {
        debug!("Starting service installation");
        debug!("Binary path: {}", config.binary_path.display());

        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        debug!("Connecting to SCM with CREATE_SERVICE access");
        let manager = Scm::local_computer(None::<&str>, manager_access).map_err(|e| {
            let err_msg = format_windows_service_error("Failed to connect to SCM", &e);
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;

        let system_data_dir = omc_core::config::paths::service_data_dir();
        let system_config_dir = omc_core::config::paths::service_config_path()
            .parent()
            .unwrap()
            .to_path_buf();

        debug!(
            "Creating system data directory: {}",
            system_data_dir.display()
        );
        std::fs::create_dir_all(&system_data_dir).map_err(|e| {
            let err_msg = format!(
                "Failed to create data directory {}: {e}",
                system_data_dir.display()
            );
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;

        debug!(
            "Creating system config directory: {}",
            system_config_dir.display()
        );
        std::fs::create_dir_all(&system_config_dir).map_err(|e| {
            let err_msg = format!(
                "Failed to create config directory {}: {e}",
                system_config_dir.display()
            );
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;

        let system_config_path = omc_core::config::paths::service_config_path();
        if !system_config_path.exists() {
            debug!(
                "Creating default config file: {}",
                system_config_path.display()
            );
            std::fs::write(&system_config_path, "{}").map_err(|e| {
                let err_msg = format!(
                    "Failed to create default config file {}: {e}",
                    system_config_path.display()
                );
                debug!("{err_msg}");
                ServiceError::Other(err_msg)
            })?;
        }

        let mut launch_arguments: Vec<OsString> = vec![OsString::from("--service")];

        launch_arguments.push(OsString::from("--data-dir"));
        launch_arguments.push(quote_string_if_needed(&system_data_dir.to_string_lossy()));

        launch_arguments.push(OsString::from("--config"));
        launch_arguments.push(quote_string_if_needed(
            &omc_core::config::paths::service_config_path().to_string_lossy(),
        ));

        debug!("Launch arguments: {:?}", launch_arguments);

        let quoted_binary_path = quote_path_if_needed(&config.binary_path);
        let service_info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_DISPLAY_NAME),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: quoted_binary_path,
            launch_arguments,
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };

        debug!("Creating service with CHANGE_CONFIG access");
        let service = match manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG) {
            Ok(s) => {
                debug!("Service created successfully");
                s
            }
            Err(e) if is_service_exists_error(&e) => {
                debug!("Service already exists (ERROR_SERVICE_EXISTS), opening existing service");
                manager
                    .open_service(SERVICE_NAME, ServiceAccess::CHANGE_CONFIG)
                    .map_err(|e| {
                        let err_msg =
                            format_windows_service_error("Failed to open existing service", &e);
                        debug!("{err_msg}");
                        ServiceError::Other(err_msg)
                    })?
            }
            Err(e) => {
                let err_msg = format_windows_service_error("Failed to create service", &e);
                debug!("{err_msg}");
                return Err(ServiceError::Other(err_msg));
            }
        };

        debug!("Updating service configuration");
        service.change_config(&service_info).map_err(|e| {
            let err_msg =
                format_windows_service_error("Failed to update service configuration", &e);
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;
        debug!("Service configuration updated");

        debug!("Setting service description");
        service
            .set_description("oh-my-codes daemon service")
            .map_err(|e| {
                let err_msg = format_windows_service_error("Failed to set description", &e);
                debug!("{err_msg}");
                ServiceError::Other(err_msg)
            })?;
        debug!("Service description set");

        debug!("Configuring failure recovery actions");
        let failure_actions = ServiceFailureActions {
            reset_period: ServiceFailureResetPeriod::After(Duration::from_secs(86400)),
            reboot_msg: None,
            command: None,
            actions: Some(vec![
                ServiceAction {
                    action_type: ServiceActionType::Restart,
                    delay: Duration::from_secs(5),
                },
                ServiceAction {
                    action_type: ServiceActionType::Restart,
                    delay: Duration::from_secs(10),
                },
                ServiceAction {
                    action_type: ServiceActionType::Restart,
                    delay: Duration::from_secs(30),
                },
            ]),
        };

        match service.update_failure_actions(failure_actions) {
            Ok(()) => debug!("Failure recovery actions configured successfully"),
            Err(e) => {
                let err_msg =
                    format_windows_service_error("Failed to set failure recovery actions", &e);
                warn!(
                    "{err_msg}. \
                     Service installed successfully but automatic restart on crash is disabled. \
                     You can configure recovery manually via services.msc."
                );
            }
        }

        debug!("Service installation complete");
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        debug!("Starting service uninstallation");
        let service = self.open_service_with_access(
            ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE,
        )?;

        let status = service.query_status().map_err(|e| {
            let err_msg = format_windows_service_error("Failed to query service status", &e);
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;

        match status.current_state {
            ServiceState::Running | ServiceState::StartPending => {
                debug!(
                    "Service is {:?}, stopping before deletion",
                    status.current_state
                );
                service.stop().map_err(|e| {
                    let err_msg = format_windows_service_error("Failed to stop service", &e);
                    debug!("{err_msg}");
                    ServiceError::Other(err_msg)
                })?;
                wait_for_service_state(&service, ServiceState::Stopped, Duration::from_secs(30))?;
                debug!("Service stopped");
            }
            ServiceState::StopPending => {
                debug!("Service is already stopping, waiting");
                wait_for_service_state(&service, ServiceState::Stopped, Duration::from_secs(30))?;
                debug!("Service stopped");
            }
            _ => {}
        }

        debug!("Deleting service");
        match service.delete() {
            Ok(()) => {
                debug!("Service marked for deletion successfully");
                Ok(())
            }
            Err(e) if is_service_marked_for_deletion_error(&e) => {
                debug!("Service already marked for deletion");
                Ok(())
            }
            Err(e) => {
                let err_msg = format_windows_service_error("Failed to delete service", &e);
                debug!("{err_msg}");
                Err(ServiceError::Other(err_msg))
            }
        }
    }

    fn start(&self) -> Result<()> {
        debug!("Starting service");
        let service = self.open_service()?;
        service.start(&[] as &[OsString]).map_err(|e| {
            let err_msg = format_windows_service_error("Failed to start service", &e);
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;
        debug!("Service start command issued");
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        debug!("Stopping service");
        let service = self.open_service()?;
        service.stop().map_err(|e| {
            let err_msg = format_windows_service_error("Failed to stop service", &e);
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;
        debug!("Service stop command issued");
        Ok(())
    }

    fn status(&self) -> Result<ServiceStatus> {
        debug!("Querying service status");
        let service = match self.open_service_with_access(ServiceAccess::QUERY_STATUS) {
            Ok(s) => s,
            Err(ServiceError::NotInstalled) => {
                debug!("Service not installed");
                return Ok(ServiceStatus::NotInstalled);
            }
            Err(e) => return Err(e),
        };
        let status = service.query_status().map_err(|e| {
            let err_msg = format_windows_service_error("Failed to query status", &e);
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;
        debug!("Service state: {:?}", status.current_state);
        match status.current_state {
            ServiceState::Running => Ok(ServiceStatus::Running {
                pid: status.process_id,
            }),
            ServiceState::Stopped | ServiceState::StopPending | ServiceState::StartPending => {
                Ok(ServiceStatus::Stopped)
            }
            _ => Ok(ServiceStatus::Unknown(format!(
                "{:?}",
                status.current_state
            ))),
        }
    }
}

fn is_service_exists_error(e: &windows_service::Error) -> bool {
    if let windows_service::Error::Winapi(io_err) = e {
        io_err.raw_os_error() == Some(1073) // ERROR_SERVICE_EXISTS
    } else {
        false
    }
}

fn is_service_not_found_error(e: &windows_service::Error) -> bool {
    if let windows_service::Error::Winapi(io_err) = e {
        io_err.raw_os_error() == Some(1060) // ERROR_SERVICE_DOES_NOT_EXIST
    } else {
        false
    }
}

fn is_service_marked_for_deletion_error(e: &windows_service::Error) -> bool {
    if let windows_service::Error::Winapi(io_err) = e {
        io_err.raw_os_error() == Some(1072) // ERROR_SERVICE_MARKED_FOR_DELETION
    } else {
        false
    }
}

fn wait_for_service_state(
    service: &windows_service::service::Service,
    target: ServiceState,
    timeout: Duration,
) -> Result<()> {
    let start = Instant::now();
    let poll_interval = Duration::from_millis(500);
    loop {
        let status = service.query_status().map_err(|e| {
            let err_msg = format_windows_service_error("Failed to query service status", &e);
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;
        if status.current_state == target {
            return Ok(());
        }
        if start.elapsed() >= timeout {
            return Err(ServiceError::Other(format!(
                "Timed out waiting for service to reach {:?} state (current: {:?})",
                target, status.current_state
            )));
        }
        std::thread::sleep(poll_interval);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_path_without_spaces() {
        let path = PathBuf::from(r"C:\Users\test\omcd.exe");
        let quoted = quote_path_if_needed(&path);
        assert_eq!(quoted, PathBuf::from(r"C:\Users\test\omcd.exe"));
    }

    #[test]
    fn test_quote_path_with_spaces() {
        let path = PathBuf::from(r"C:\Program Files\omcd.exe");
        let quoted = quote_path_if_needed(&path);
        assert_eq!(quoted, PathBuf::from(r#""C:\Program Files\omcd.exe""#));
    }

    #[test]
    fn test_quote_path_already_quoted() {
        let path = PathBuf::from(r#""C:\Program Files\omcd.exe""#);
        let quoted = quote_path_if_needed(&path);
        assert_eq!(quoted, PathBuf::from(r#""C:\Program Files\omcd.exe""#));
    }

    #[test]
    fn test_quote_string_without_spaces() {
        let s = "simple";
        let quoted = quote_string_if_needed(s);
        assert_eq!(quoted, OsString::from("simple"));
    }

    #[test]
    fn test_quote_string_with_spaces() {
        let s = r"C:\Program Files\omcd.exe";
        let quoted = quote_string_if_needed(s);
        assert_eq!(quoted, OsString::from(r#""C:\Program Files\omcd.exe""#));
    }

    #[test]
    fn test_quote_string_already_quoted() {
        let s = r#""C:\Program Files\omcd.exe""#;
        let quoted = quote_string_if_needed(s);
        assert_eq!(quoted, OsString::from(r#""C:\Program Files\omcd.exe""#));
    }

    #[test]
    fn test_quote_path_with_user_appdata_spaces() {
        let path = PathBuf::from(r"C:\Users\John Doe\AppData\Roaming\omc\data");
        let quoted = quote_path_if_needed(&path);
        assert_eq!(
            quoted,
            PathBuf::from(r#""C:\Users\John Doe\AppData\Roaming\omc\data""#)
        );
    }
}
