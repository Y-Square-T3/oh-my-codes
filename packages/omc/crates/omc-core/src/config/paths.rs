use std::path::{Path, PathBuf};

const CONFIG_DIR_NAME: &str = "omc";
const CONFIG_FILE_NAME: &str = "omc.json";
const PROJECT_CONFIG_DIR_NAME: &str = ".omc";
const SOCKET_FILE_NAME: &str = "omc.sock";

#[cfg(windows)]
const PROGRAM_DATA_DIR: &str = "ProgramData";

#[cfg(windows)]
pub fn is_windows_service() -> bool {
    false
}

#[cfg(not(windows))]
pub fn is_windows_service() -> bool {
    false
}

#[cfg(windows)]
pub fn service_data_dir() -> PathBuf {
    PathBuf::from("C:\\")
        .join(PROGRAM_DATA_DIR)
        .join(CONFIG_DIR_NAME)
        .join("data")
}

#[cfg(windows)]
pub fn service_config_path() -> PathBuf {
    PathBuf::from("C:\\")
        .join(PROGRAM_DATA_DIR)
        .join(CONFIG_DIR_NAME)
        .join(CONFIG_FILE_NAME)
}

#[cfg(windows)]
pub fn service_pid_path() -> PathBuf {
    PathBuf::from("C:\\")
        .join(PROGRAM_DATA_DIR)
        .join(CONFIG_DIR_NAME)
        .join("omcd.pid")
}

pub fn user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(CONFIG_DIR_NAME))
}

#[allow(unused_variables)]
pub fn default_data_dir_with_mode(service_mode: bool) -> PathBuf {
    #[cfg(windows)]
    {
        if service_mode {
            return service_data_dir();
        }
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_DIR_NAME)
        .join("data")
}

pub fn default_data_dir() -> PathBuf {
    default_data_dir_with_mode(false)
}

#[allow(unused_variables)]
pub fn default_config_path_with_mode(service_mode: bool) -> PathBuf {
    #[cfg(windows)]
    {
        if service_mode {
            return service_config_path();
        }
    }
    user_config_path()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_FILE_NAME)
}

pub fn default_config_path() -> PathBuf {
    default_config_path_with_mode(false)
}

pub fn default_socket_path() -> String {
    let dir = user_config_path().unwrap_or_else(|| PathBuf::from("."));
    dir.join(SOCKET_FILE_NAME).to_string_lossy().to_string()
}

#[allow(unused_variables)]
pub fn default_pid_path_with_mode(service_mode: bool) -> String {
    #[cfg(windows)]
    {
        if service_mode {
            return service_pid_path().to_string_lossy().to_string();
        }
    }
    let dir = user_config_path().unwrap_or_else(|| PathBuf::from("."));
    dir.join("omcd.pid").to_string_lossy().to_string()
}

pub fn default_pid_path() -> String {
    default_pid_path_with_mode(false)
}

pub fn find_project_config() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        let candidate = current.join(PROJECT_CONFIG_DIR_NAME).join(CONFIG_FILE_NAME);
        if candidate.exists() {
            return Some(candidate);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

pub fn config_dir_name() -> &'static str {
    CONFIG_DIR_NAME
}

pub fn config_file_name() -> &'static str {
    CONFIG_FILE_NAME
}

pub fn project_config_dir_name() -> &'static str {
    PROJECT_CONFIG_DIR_NAME
}

pub fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let git_dir = current.join(".git");
        if git_dir.exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}
