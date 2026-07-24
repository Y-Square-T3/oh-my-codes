use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub message_id: String,
    pub timestamp: i64,
}

pub struct Wal {
    path: PathBuf,
}

impl Wal {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub fn append(&self, entry: &WalEntry) -> omc_core::error::Result<()> {
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| omc_core::error::OmcError::Storage(format!("Failed to open WAL: {e}")))?;
        let line = serde_json::to_string(entry).map_err(|e| {
            omc_core::error::OmcError::Storage(format!("Failed to serialize WAL entry: {e}"))
        })?;
        writeln!(file, "{line}")
            .map_err(|e| omc_core::error::OmcError::Storage(format!("Failed to write WAL: {e}")))?;
        Ok(())
    }

    pub fn read_all(&self) -> omc_core::error::Result<Vec<WalEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path)
            .map_err(|e| omc_core::error::OmcError::Storage(format!("Failed to read WAL: {e}")))?;
        let entries: Vec<WalEntry> = content
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                serde_json::from_str(line).map_err(|e| {
                    omc_core::error::OmcError::Storage(format!("Failed to parse WAL entry: {e}"))
                })
            })
            .collect::<omc_core::error::Result<Vec<_>>>()?;
        Ok(entries)
    }

    pub fn truncate(&self) -> omc_core::error::Result<()> {
        if self.path.exists() {
            fs::write(&self.path, "").map_err(|e| {
                omc_core::error::OmcError::Storage(format!("Failed to truncate WAL: {e}"))
            })?;
        }
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
