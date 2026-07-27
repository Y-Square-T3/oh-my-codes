use crate::{Result, ServiceConfig, ServiceError, ServiceManager, ServiceStatus};
use std::path::PathBuf;

pub struct TaskSchedulerManager {
    task_name: String,
    binary_dest: PathBuf,
}

impl Default for TaskSchedulerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskSchedulerManager {
    pub fn new() -> Self {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
        let binary_dest = PathBuf::from(local_app_data)
            .join("oh-my-codes")
            .join("omcd.exe");
        Self {
            task_name: r"\oh-my-codes\omcd".to_string(),
            binary_dest,
        }
    }

    fn generate_task_xml(&self, binary_path: &std::path::Path) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <Triggers>
    <LogonTrigger />
  </Triggers>
  <Actions>
    <Exec>
      <Command>{}</Command>
    </Exec>
  </Actions>
</Task>"#,
            binary_path.display()
        )
    }
}

impl ServiceManager for TaskSchedulerManager {
    fn install(&self, config: &ServiceConfig) -> Result<()> {
        crate::copy_binary(&config.binary_path, &self.binary_dest)?;
        let xml = self.generate_task_xml(&self.binary_dest);
        let xml_path = self.binary_dest.with_extension("xml");
        std::fs::write(&xml_path, xml)?;
        let output = std::process::Command::new("schtasks")
            .args([
                "/Create",
                "/TN",
                &self.task_name,
                "/XML",
                &xml_path.to_string_lossy(),
                "/F",
            ])
            .output()?;
        if !output.status.success() {
            return Err(ServiceError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        let _ = std::fs::remove_file(&xml_path);
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let _ = std::process::Command::new("schtasks")
            .args(["/Delete", "/TN", &self.task_name, "/F"])
            .output();
        crate::remove_binary(&self.binary_dest)?;
        Ok(())
    }

    fn start(&self) -> Result<()> {
        let output = std::process::Command::new("schtasks")
            .args(["/Run", "/TN", &self.task_name])
            .output()?;
        if !output.status.success() {
            return Err(ServiceError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        let output = std::process::Command::new("schtasks")
            .args(["/End", "/TN", &self.task_name])
            .output()?;
        if !output.status.success() {
            return Err(ServiceError::Other(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }
        Ok(())
    }

    fn status(&self) -> Result<ServiceStatus> {
        let output = std::process::Command::new("schtasks")
            .args(["/Query", "/TN", &self.task_name, "/FO", "LIST"])
            .output()?;
        if !output.status.success() {
            return Ok(ServiceStatus::NotInstalled);
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.contains("Running") {
            return Ok(ServiceStatus::Stopped);
        }
        let tasklist = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq omcd.exe", "/FO", "CSV", "/NH"])
            .output()?;
        let tasklist_stdout = String::from_utf8_lossy(&tasklist.stdout);
        let pid = tasklist_stdout.lines().next().and_then(|line| {
            line.split(',')
                .nth(1)
                .and_then(|p| p.trim_matches('"').parse::<u32>().ok())
        });
        match pid {
            Some(p) => Ok(ServiceStatus::Running { pid: Some(p) }),
            None => Ok(ServiceStatus::Stopped),
        }
    }
}
