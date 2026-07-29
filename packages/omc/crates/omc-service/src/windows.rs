use crate::{Result, ServiceConfig, ServiceError, ServiceManager, ServiceStatus};
use std::ffi::OsString;
use windows_service::service::{
    ServiceAccess, ServiceControl, ServiceErrorControl, ServiceFailureActions,
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

    fn open_service(&self) -> Result<windows_service::service::Service> {
        let manager_access = ServiceManagerAccess::CONNECT;
        let manager = Scm::local_computer(None::<&str>, manager_access)
            .map_err(|e| ServiceError::Other(format!("Failed to connect to SCM: {e}")))?;
        manager
            .open_service(
                SERVICE_NAME,
                ServiceAccess::QUERY_STATUS | ServiceAccess::START | ServiceAccess::STOP,
            )
            .map_err(|_| ServiceError::NotInstalled)
    }
}

impl ServiceManager for WindowsServiceManager {
    fn install(&self, config: &ServiceConfig) -> Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let manager = Scm::local_computer(None::<&str>, manager_access)
            .map_err(|e| ServiceError::Other(format!("Failed to connect to SCM: {e}")))?;

        let mut launch_arguments: Vec<OsString> = vec![OsString::from("--service")];

        if let Some(ref data_dir) = config.data_dir {
            launch_arguments.push(OsString::from("--data-dir"));
            launch_arguments.push(OsString::from(data_dir));
        }

        if let Some(ref config_path) = config.config {
            launch_arguments.push(OsString::from("--config"));
            launch_arguments.push(OsString::from(config_path));
        }

        let service_info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_DISPLAY_NAME),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::Automatic,
            error_control: ServiceErrorControl::Normal,
            executable_path: config.binary_path.clone(),
            launch_arguments,
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };

        let service = manager
            .create_service(&service_info, ServiceAccess::CHANGE_CONFIG)
            .map_err(|e| ServiceError::Other(format!("Failed to create service: {e}")))?;

        service
            .set_description("oh-my-codes daemon service")
            .map_err(|e| ServiceError::Other(format!("Failed to set description: {e}")))?;

        let failure_actions = ServiceFailureActions {
            reset_period: ServiceFailureResetPeriod::Days(1),
            reboot_message: None,
            command: None,
            actions: vec![
                windows_service::service::ServiceAction {
                    type_: windows_service::service::ServiceActionType::Restart,
                    delay: std::time::Duration::from_secs(5),
                },
                windows_service::service::ServiceAction {
                    type_: windows_service::service::ServiceActionType::Restart,
                    delay: std::time::Duration::from_secs(10),
                },
                windows_service::service::ServiceAction {
                    type_: windows_service::service::ServiceActionType::Restart,
                    delay: std::time::Duration::from_secs(30),
                },
            ],
        };

        service
            .set_failure_actions(failure_actions)
            .map_err(|e| ServiceError::Other(format!("Failed to set failure actions: {e}")))?;

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let service = self.open_service()?;
        service
            .delete()
            .map_err(|e| ServiceError::Other(format!("Failed to delete service: {e}")))?;
        Ok(())
    }

    fn start(&self) -> Result<()> {
        let service = self.open_service()?;
        service
            .start(&[] as &[OsString])
            .map_err(|e| ServiceError::Other(format!("Failed to start service: {e}")))?;
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        let service = self.open_service()?;
        service
            .control(ServiceControl::Stop)
            .map_err(|e| ServiceError::Other(format!("Failed to stop service: {e}")))?;
        Ok(())
    }

    fn status(&self) -> Result<ServiceStatus> {
        let service = match self.open_service() {
            Ok(s) => s,
            Err(ServiceError::NotInstalled) => return Ok(ServiceStatus::NotInstalled),
            Err(e) => return Err(e),
        };
        let status = service
            .query_status()
            .map_err(|e| ServiceError::Other(format!("Failed to query status: {e}")))?;
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
