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
    pub database_url: String,
}

impl OmcConfig {
    #[allow(unused_variables)]
    pub fn load(custom_path: Option<&Path>, service_mode: bool) -> Result<Self, OmcError> {
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

        #[cfg(windows)]
        {
            if service_mode {
                let service_config = paths::service_config_path();
                if service_config.exists() {
                    return Self::load_from_file(&service_config);
                }
            }
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
        if other.daemon.database_url.is_some() {
            self.daemon.database_url = other.daemon.database_url.clone();
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

    pub fn resolve_daemon_with_mode(&self, service_mode: bool) -> ResolvedDaemonConfig {
        let data_dir = self.daemon.data_dir.clone().unwrap_or_else(|| {
            paths::default_data_dir_with_mode(service_mode)
                .to_string_lossy()
                .to_string()
        });
        let database_url = self
            .daemon
            .database_url
            .clone()
            .unwrap_or_else(|| format!("sqlite:{}/omc.db", data_dir));
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
            data_dir,
            database_url,
        }
    }

    pub fn resolve_daemon(&self) -> ResolvedDaemonConfig {
        self.resolve_daemon_with_mode(false)
    }
}
