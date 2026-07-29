use crate::{Result, ServiceConfig, ServiceError, ServiceManager, ServiceStatus};
use std::ffi::OsString;
use std::time::Duration;
use tracing::{debug, warn};
use windows_service::service::{
    ServiceAccess, ServiceAction, ServiceActionType, ServiceErrorControl, ServiceFailureActions,
    ServiceFailureResetPeriod, ServiceInfo, ServiceStartType, ServiceState, ServiceType,
};
use windows_service::service_manager::{ServiceManager as Scm, ServiceManagerAccess};

const SERVICE_NAME: &str = "omcd";
const SERVICE_DISPLAY_NAME: &str = "OMC Daemon";

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
            let err_msg = format!("Failed to connect to SCM: {e}");
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;
        debug!(
            "Opening service '{}' with access {:?}",
            SERVICE_NAME, access
        );
        manager.open_service(SERVICE_NAME, access).map_err(|e| {
            debug!("Service not found or access denied: {e}");
            ServiceError::NotInstalled
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
        if let Some(ref data_dir) = config.data_dir {
            debug!("Data dir: {data_dir}");
        }
        if let Some(ref config_path) = config.config {
            debug!("Config path: {config_path}");
        }

        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        debug!("Connecting to SCM with CREATE_SERVICE access");
        let manager = Scm::local_computer(None::<&str>, manager_access).map_err(|e| {
            let err_msg = format!("Failed to connect to SCM: {e}");
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;

        let mut launch_arguments: Vec<OsString> = vec![OsString::from("--service")];

        if let Some(ref data_dir) = config.data_dir {
            launch_arguments.push(OsString::from("--data-dir"));
            launch_arguments.push(OsString::from(data_dir));
        }

        if let Some(ref config_path) = config.config {
            launch_arguments.push(OsString::from("--config"));
            launch_arguments.push(OsString::from(config_path));
        }

        debug!("Launch arguments: {:?}", launch_arguments);

        let service_info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_DISPLAY_NAME),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: config.binary_path.clone(),
            launch_arguments,
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };

        debug!("Creating service with CHANGE_CONFIG access");
        let service = manager
            .create_service(&service_info, ServiceAccess::CHANGE_CONFIG)
            .map_err(|e| {
                let err_msg = format!("Failed to create service: {e}");
                debug!("{err_msg}");
                ServiceError::Other(err_msg)
            })?;
        debug!("Service created successfully");

        debug!("Setting service description");
        service
            .set_description("oh-my-codes daemon service")
            .map_err(|e| {
                let err_msg = format!("Failed to set description: {e}");
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
                warn!(
                    "Failed to set failure recovery actions (non-fatal): {e}. \
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

        debug!("Deleting service");
        service.delete().map_err(|e| {
            let err_msg = format!("Failed to delete service: {e}");
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;
        debug!("Service deleted successfully");
        Ok(())
    }

    fn start(&self) -> Result<()> {
        debug!("Starting service");
        let service = self.open_service()?;
        service.start(&[] as &[OsString]).map_err(|e| {
            let err_msg = format!("Failed to start service: {e}");
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
            let err_msg = format!("Failed to stop service: {e}");
            debug!("{err_msg}");
            ServiceError::Other(err_msg)
        })?;
        debug!("Service stop command issued");
        Ok(())
    }

    fn status(&self) -> Result<ServiceStatus> {
        debug!("Querying service status");
        let service = match self.open_service() {
            Ok(s) => s,
            Err(ServiceError::NotInstalled) => {
                debug!("Service not installed");
                return Ok(ServiceStatus::NotInstalled);
            }
            Err(e) => return Err(e),
        };
        let status = service.query_status().map_err(|e| {
            let err_msg = format!("Failed to query status: {e}");
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
