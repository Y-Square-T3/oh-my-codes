#[cfg(target_os = "macos")]
pub mod launchd;
#[cfg(target_os = "linux")]
pub mod systemd;
#[cfg(target_os = "windows")]
pub mod windows;

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Service not installed")]
    NotInstalled,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ServiceError>;

pub struct ServiceConfig {
    pub binary_path: PathBuf,
    pub data_dir: Option<String>,
    pub config: Option<String>,
}

pub enum ServiceStatus {
    Running { pid: Option<u32> },
    Stopped,
    NotInstalled,
    Unknown(String),
}

pub trait ServiceManager {
    fn install(&self, config: &ServiceConfig) -> Result<()>;
    fn uninstall(&self) -> Result<()>;
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn status(&self) -> Result<ServiceStatus>;
    fn restart(&self) -> Result<()> {
        self.stop()?;
        self.start()?;
        Ok(())
    }
}

pub fn create_service_manager() -> Box<dyn ServiceManager> {
    #[cfg(target_os = "macos")]
    return Box::new(launchd::LaunchdManager::new());
    #[cfg(target_os = "linux")]
    return Box::new(systemd::SystemdManager::new());
    #[cfg(target_os = "windows")]
    return Box::new(windows::WindowsServiceManager::new());
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    compile_error!("Unsupported platform for service management");
}

pub fn find_omcd_binary() -> Result<PathBuf> {
    let current = std::env::current_exe()?;
    let dir = current
        .parent()
        .ok_or_else(|| ServiceError::Other("Cannot determine binary directory".to_string()))?;
    let omcd = dir.join("omcd");
    if omcd.exists() {
        return Ok(omcd);
    }
    let omcd_exe = dir.join("omcd.exe");
    if omcd_exe.exists() {
        return Ok(omcd_exe);
    }
    Err(ServiceError::Other("omcd binary not found".to_string()))
}

pub fn create_symlink(source: &Path, dest: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        if dest.exists() {
            std::fs::remove_file(dest)?;
        }
        std::os::unix::fs::symlink(source, dest)?;
    }
    #[cfg(windows)]
    {
        if dest.exists() {
            std::fs::remove_file(dest)?;
        }
        std::os::windows::fs::symlink_file(source, dest)?;
    }
    Ok(())
}

pub fn remove_symlink(path: &Path) -> Result<()> {
    if path.exists() || path.symlink_metadata().is_ok() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn copy_binary(source: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(source, dest)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

pub fn remove_binary(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
