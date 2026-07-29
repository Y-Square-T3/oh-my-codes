#![cfg(windows)]

use clap::Parser;
use std::ffi::OsString;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use windows_service::define_windows_service;
use windows_service::service::{
    ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType,
};
use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
use windows_service::service_dispatcher;

const SERVICE_NAME: &str = "omcd";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

define_windows_service!(ffi_service_main, my_service_main);

fn my_service_main(arguments: Vec<OsString>) {
    crate::init_tracing();
    tracing::debug!("Service entry point called with arguments: {:?}", arguments);
    if let Err(e) = run_service(arguments) {
        tracing::error!("Service error: {e}");
    }
}

pub fn run() -> windows_service::Result<()> {
    tracing::debug!("Starting service dispatcher");
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

fn run_service(_arguments: Vec<OsString>) -> windows_service::Result<()> {
    tracing::debug!("Initializing service shutdown channel");
    let (shutdown_tx, shutdown_rx) = watch::channel(());
    let shutdown_tx = Arc::new(std::sync::Mutex::new(Some(shutdown_tx)));

    let event_handler = {
        let shutdown_tx = shutdown_tx.clone();
        move |control_event| -> ServiceControlHandlerResult {
            tracing::debug!("Service control event received: {:?}", control_event);
            match control_event {
                ServiceControl::Interrogate => {
                    tracing::debug!("Interrogate event - reporting no error");
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Stop => {
                    tracing::info!("Stop event received - initiating graceful shutdown");
                    if let Ok(mut guard) = shutdown_tx.lock() {
                        if let Some(tx) = guard.take() {
                            tracing::debug!("Sending shutdown signal to daemon");
                            let _ = tx.send(());
                        } else {
                            tracing::debug!("Shutdown signal already sent");
                        }
                    } else {
                        tracing::error!("Failed to acquire shutdown_tx lock");
                    }
                    ServiceControlHandlerResult::NoError
                }
                other => {
                    tracing::debug!("Unhandled control event: {:?}", other);
                    ServiceControlHandlerResult::NotImplemented
                }
            }
        }
    };

    tracing::debug!("Registering service control handler");
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;
    tracing::debug!("Service control handler registered");

    tracing::debug!("Reporting StartPending to SCM");
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(30),
        process_id: None,
    })?;
    tracing::debug!("StartPending reported");

    tracing::debug!("Creating tokio runtime");
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => {
            tracing::debug!("Tokio runtime created");
            rt
        }
        Err(e) => {
            tracing::error!("Failed to create tokio runtime: {e}");
            return Ok(());
        }
    };

    tracing::debug!("Parsing daemon arguments");
    let args = crate::Args::parse();
    tracing::debug!(
        "Arguments parsed: config={:?}, data_dir={:?}, bind_addr={:?}, bind_port={:?}",
        args.config,
        args.data_dir,
        args.bind_addr,
        args.bind_port
    );

    tracing::debug!("Reporting Running to SCM");
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;
    tracing::debug!("Running reported");

    tracing::info!("Starting daemon in service mode");
    rt.block_on(async {
        if let Err(e) = crate::run_daemon(args, Some(shutdown_rx), true).await {
            tracing::error!("Daemon error: {e}");
        }
    });
    tracing::debug!("Daemon shutdown complete");

    tracing::debug!("Reporting Stopped to SCM");
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;
    tracing::debug!("Stopped reported to SCM");

    Ok(())
}
