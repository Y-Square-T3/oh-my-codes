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
        let (encoded, _, _) = encoding_rs::UTF_16LE.encode(&xml);
        let mut bytes = vec![0xFF, 0xFE];
        bytes.extend_from_slice(&encoded);
        std::fs::write(&xml_path, &bytes)?;
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
            return Err(ServiceError::Other(decode_oem_output(&output.stderr)));
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
            return Err(ServiceError::Other(decode_oem_output(&output.stderr)));
        }
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        let output = std::process::Command::new("schtasks")
            .args(["/End", "/TN", &self.task_name])
            .output()?;
        if !output.status.success() {
            return Err(ServiceError::Other(decode_oem_output(&output.stderr)));
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
        let stdout = decode_oem_output(&output.stdout);
        if !stdout.contains("Running") {
            return Ok(ServiceStatus::Stopped);
        }
        let tasklist = std::process::Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq omcd.exe", "/FO", "CSV", "/NH"])
            .output()?;
        let tasklist_stdout = decode_oem_output(&tasklist.stdout);
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

fn decode_oem_output(bytes: &[u8]) -> String {
    let codepage = unsafe { GetOEMCP() };
    let label = codepage_to_label(codepage);
    match label.and_then(|l| encoding_rs::Encoding::for_label(l.as_bytes())) {
        Some(encoding) => {
            let (decoded, _, had_errors) = encoding.decode(bytes);
            if had_errors {
                String::from_utf8_lossy(bytes).into_owned()
            } else {
                decoded.into_owned()
            }
        }
        None => String::from_utf8_lossy(bytes).into_owned(),
    }
}

fn codepage_to_label(codepage: u32) -> Option<&'static str> {
    match codepage {
        437 => Some("ibm866"),
        850 => Some("ibm866"),
        866 => Some("ibm866"),
        932 => Some("shift_jis"),
        936 => Some("gbk"),
        949 => Some("euc-kr"),
        950 => Some("big5"),
        1200 => Some("utf-16le"),
        1201 => Some("utf-16be"),
        1250 => Some("windows-1250"),
        1251 => Some("windows-1251"),
        1252 => Some("windows-1252"),
        1253 => Some("windows-1253"),
        1254 => Some("windows-1254"),
        1255 => Some("windows-1255"),
        1256 => Some("windows-1256"),
        1257 => Some("windows-1257"),
        1258 => Some("windows-1258"),
        65001 => Some("utf-8"),
        _ => None,
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetOEMCP() -> u32;
}
