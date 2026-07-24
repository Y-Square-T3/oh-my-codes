use crate::{Result, ServiceConfig, ServiceError, ServiceManager, ServiceStatus};
use std::path::PathBuf;

pub struct SystemdManager {
    unit_path: PathBuf,
    symlink_path: PathBuf,
}

impl Default for SystemdManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemdManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let unit_path = home
            .join(".config")
            .join("systemd")
            .join("user")
            .join("omcd.service");
        let symlink_path = home.join(".local").join("bin").join("omcd");
        Self {
            unit_path,
            symlink_path,
        }
    }

    fn generate_unit(&self, binary_path: &std::path::Path) -> String {
        format!(
            r#"[Unit]
Description=oh-my-codes daemon
After=network.target

[Service]
ExecStart={}
Restart=on-failure

[Install]
WantedBy=default.target
"#,
            binary_path.display()
        )
    }
}

impl ServiceManager for SystemdManager {
    fn install(&self, config: &ServiceConfig) -> Result<()> {
        std::fs::create_dir_all(self.unit_path.parent().unwrap())?;
        if let Some(parent) = self.symlink_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        crate::create_symlink(&config.binary_path, &self.symlink_path)?;
        let unit = self.generate_unit(&self.symlink_path);
        std::fs::write(&self.unit_path, unit)?;
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "disable", "omcd.service"])
            .output();
        crate::remove_symlink(&self.symlink_path)?;
        if self.unit_path.exists() {
            std::fs::remove_file(&self.unit_path)?;
        }
        Ok(())
    }

    fn start(&self) -> Result<()> {
        if !self.unit_path.exists() {
            return Err(ServiceError::NotInstalled);
        }
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .output();
        let output = std::process::Command::new("systemctl")
            .args(["--user", "start", "omcd.service"])
            .output()?;
        if !output.status.success() {
            return Err(ServiceError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        if !self.unit_path.exists() {
            return Err(ServiceError::NotInstalled);
        }
        let output = std::process::Command::new("systemctl")
            .args(["--user", "stop", "omcd.service"])
            .output()?;
        if !output.status.success() {
            return Err(ServiceError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn status(&self) -> Result<ServiceStatus> {
        if !self.unit_path.exists() {
            return Ok(ServiceStatus::NotInstalled);
        }
        let output = std::process::Command::new("systemctl")
            .args(["--user", "is-active", "omcd.service"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        match stdout.as_str() {
            "active" => Ok(ServiceStatus::Running { pid: None }),
            "inactive" | "failed" => Ok(ServiceStatus::Stopped),
            other => Ok(ServiceStatus::Unknown(other.to_string())),
        }
    }
}
