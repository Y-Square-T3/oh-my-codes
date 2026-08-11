use thiserror::Error;

#[derive(Debug, Error)]
pub enum OmcError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Auth error: {0}")]
    Auth(String),
    #[error("Token expired")]
    TokenExpired,
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("{}", port_in_use_message(.address, .config_path))]
    PortInUse {
        address: String,
        config_path: String,
    },
}

fn port_in_use_message(address: &str, config_path: &str) -> String {
    format!(
        "Address {address} is already in use. Another process is listening on this port. \
         To use a different port, edit the config file at {config_path} and set daemon.bind_port \
         to a free port, then restart the daemon."
    )
}

pub type Result<T> = std::result::Result<T, OmcError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_in_use_message_contains_address() {
        let msg = port_in_use_message("127.0.0.1:9823", "/home/user/.config/omc/omc.json");
        assert!(msg.contains("127.0.0.1:9823"));
    }

    #[test]
    fn test_port_in_use_message_contains_config_path() {
        let msg = port_in_use_message("127.0.0.1:9823", "/home/user/.config/omc/omc.json");
        assert!(msg.contains("/home/user/.config/omc/omc.json"));
    }

    #[test]
    fn test_port_in_use_message_mentions_bind_port() {
        let msg = port_in_use_message("127.0.0.1:9823", "/home/user/.config/omc/omc.json");
        assert!(msg.contains("daemon.bind_port"));
    }

    #[test]
    fn test_port_in_use_message_mentions_restart() {
        let msg = port_in_use_message("127.0.0.1:9823", "/home/user/.config/omc/omc.json");
        assert!(msg.contains("restart the daemon"));
    }

    #[test]
    fn test_port_in_use_error_display() {
        let err = OmcError::PortInUse {
            address: "127.0.0.1:9823".to_string(),
            config_path: "/home/user/.config/omc/omc.json".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("127.0.0.1:9823"));
        assert!(msg.contains("/home/user/.config/omc/omc.json"));
        assert!(msg.contains("daemon.bind_port"));
        assert!(msg.contains("restart the daemon"));
    }
}
