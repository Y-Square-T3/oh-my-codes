use crate::config::paths;
use crate::config::types::DaemonConfig;
use crate::error::OmcError;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OmcConfig {
    #[serde(default)]
    pub daemon: DaemonConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedDaemonConfig {
    pub bind_addr: String,
    pub bind_port: u16,
    pub socket_path: String,
    pub data_dir: String,
}

impl OmcConfig {
    pub fn load(custom_path: Option<&Path>) -> Result<Self, OmcError> {
        if let Some(path) = custom_path {
            return Self::load_from_file(path);
        }

        if let Ok(env_config) = std::env::var("OMC_CONFIG") {
            let path = Path::new(&env_config);
            if path.exists() {
                return Self::load_from_file(path);
            }
        }

        if let Some(project_config) = paths::find_project_config() {
            return Self::load_from_file(&project_config);
        }

        if let Some(user_config) = paths::user_config_path() {
            let config_file = user_config.join(paths::config_file_name());
            if config_file.exists() {
                return Self::load_from_file(&config_file);
            }
        }

        Ok(Self::default())
    }

    pub fn load_from_file(path: &Path) -> Result<Self, OmcError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| OmcError::Config(format!("Failed to read config file: {e}")))?;
        serde_json::from_str(&content)
            .map_err(|e| OmcError::Config(format!("Failed to parse config file: {e}")))
    }

    pub fn load_from_env() -> Self {
        Self::default()
    }

    pub fn merge(&mut self, other: &Self) {
        if other.daemon.bind_addr.is_some() {
            self.daemon.bind_addr = other.daemon.bind_addr.clone();
        }
        if other.daemon.bind_port.is_some() {
            self.daemon.bind_port = other.daemon.bind_port;
        }
        if other.daemon.socket_path.is_some() {
            self.daemon.socket_path = other.daemon.socket_path.clone();
        }
        if other.daemon.data_dir.is_some() {
            self.daemon.data_dir = other.daemon.data_dir.clone();
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<(), OmcError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| OmcError::Config(format!("Failed to create config directory: {e}")))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| OmcError::Config(format!("Failed to serialize config: {e}")))?;
        std::fs::write(path, content)
            .map_err(|e| OmcError::Config(format!("Failed to write config file: {e}")))
    }

    pub fn resolve_daemon(&self) -> ResolvedDaemonConfig {
        ResolvedDaemonConfig {
            bind_addr: self
                .daemon
                .bind_addr
                .clone()
                .unwrap_or_else(|| "127.0.0.1".to_string()),
            bind_port: self.daemon.bind_port.unwrap_or(9823),
            socket_path: self
                .daemon
                .socket_path
                .clone()
                .unwrap_or_else(paths::default_socket_path),
            data_dir: self
                .daemon
                .data_dir
                .clone()
                .unwrap_or_else(|| paths::default_data_dir().to_string_lossy().to_string()),
        }
    }
}
