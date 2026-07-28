use crate::{Result, ServiceConfig, ServiceError, ServiceManager, ServiceStatus};
use std::path::PathBuf;

pub struct LaunchdManager {
    plist_path: PathBuf,
    symlink_path: PathBuf,
    log_dir: PathBuf,
}

impl Default for LaunchdManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LaunchdManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let plist_path = home
            .join("Library")
            .join("LaunchAgents")
            .join("com.oh-my-codes.omcd.plist");
        let symlink_path = home.join(".local").join("bin").join("omcd");
        let log_dir = home.join("Library").join("Logs").join("omcd");
        Self {
            plist_path,
            symlink_path,
            log_dir,
        }
    }

    fn generate_plist(&self, binary_path: &std::path::Path) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.oh-my-codes.omcd</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}/omcd.log</string>
    <key>StandardErrorPath</key>
    <string>{}/omcd.err.log</string>
</dict>
</plist>"#,
            binary_path.display(),
            self.log_dir.display(),
            self.log_dir.display()
        )
    }
}

impl ServiceManager for LaunchdManager {
    fn install(&self, config: &ServiceConfig) -> Result<()> {
        std::fs::create_dir_all(self.plist_path.parent().unwrap())?;
        std::fs::create_dir_all(&self.log_dir)?;
        if let Some(parent) = self.symlink_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        crate::create_symlink(&config.binary_path, &self.symlink_path)?;
        let plist = self.generate_plist(&self.symlink_path);
        std::fs::write(&self.plist_path, plist)?;
        let output = std::process::Command::new("launchctl")
            .args(["load", "-w", &self.plist_path.to_string_lossy()])
            .output()?;
        if !output.status.success() {
            return Err(ServiceError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let _ = std::process::Command::new("launchctl")
            .args(["unload", &self.plist_path.to_string_lossy()])
            .output();
        crate::remove_symlink(&self.symlink_path)?;
        if self.plist_path.exists() {
            std::fs::remove_file(&self.plist_path)?;
        }
        Ok(())
    }

    fn start(&self) -> Result<()> {
        if !self.plist_path.exists() {
            return Err(ServiceError::NotInstalled);
        }
        let output = std::process::Command::new("launchctl")
            .args(["load", "-w", &self.plist_path.to_string_lossy()])
            .output()?;
        if !output.status.success() {
            return Err(ServiceError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        if !self.plist_path.exists() {
            return Err(ServiceError::NotInstalled);
        }
        let output = std::process::Command::new("launchctl")
            .args(["unload", &self.plist_path.to_string_lossy()])
            .output()?;
        if !output.status.success() {
            return Err(ServiceError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn status(&self) -> Result<ServiceStatus> {
        if !self.plist_path.exists() {
            return Ok(ServiceStatus::NotInstalled);
        }
        let output = std::process::Command::new("launchctl")
            .args(["list", "com.oh-my-codes.omcd"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.contains("com.oh-my-codes.omcd") {
            return Ok(ServiceStatus::Stopped);
        }
        let pid = stdout.lines().find_map(|line| {
            let trimmed = line.trim();
            if trimmed.contains("\"PID\"") {
                trimmed
                    .split('=')
                    .nth(1)
                    .and_then(|v| v.trim().trim_end_matches(';').trim().parse::<u32>().ok())
            } else {
                None
            }
        });
        match pid {
            Some(p) => Ok(ServiceStatus::Running { pid: Some(p) }),
            None => Ok(ServiceStatus::Stopped),
        }
    }
}
