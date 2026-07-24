use std::path::{Path, PathBuf};

const CONFIG_DIR_NAME: &str = "omc";
const CONFIG_FILE_NAME: &str = "omc.json";
const PROJECT_CONFIG_DIR_NAME: &str = ".omc";
const SOCKET_FILE_NAME: &str = "omc.sock";

pub fn user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join(CONFIG_DIR_NAME))
}

pub fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_DIR_NAME)
        .join("data")
}

pub fn default_socket_path() -> String {
    let dir = user_config_path().unwrap_or_else(|| PathBuf::from("."));
    dir.join(SOCKET_FILE_NAME).to_string_lossy().to_string()
}

pub fn default_pid_path() -> String {
    let dir = user_config_path().unwrap_or_else(|| PathBuf::from("."));
    dir.join("omcd.pid").to_string_lossy().to_string()
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
