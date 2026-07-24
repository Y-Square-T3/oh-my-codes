use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct DaemonConfig {
    pub bind_addr: Option<String>,
    pub bind_port: Option<u16>,
    pub socket_path: Option<String>,
    pub data_dir: Option<String>,
    pub auth_token: Option<String>,
}
