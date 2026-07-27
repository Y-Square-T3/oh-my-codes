use axum::extract::State;
use omc_api::types::*;
use omc_core::config::paths;
use std::sync::Arc;

use crate::DaemonState;

pub async fn handler(State(state): State<Arc<DaemonState>>) -> axum::Json<ConfigResponse> {
    let config = state.config().await;
    let resolved = config.resolve_daemon();
    axum::Json(ConfigResponse {
        daemon: ResolvedDaemonConfigJson {
            bind_addr: resolved.bind_addr,
            bind_port: resolved.bind_port,
            socket_path: resolved.socket_path,
            data_dir: resolved.data_dir,
        },
    })
}

pub async fn path_handler() -> axum::Json<ConfigPathResponse> {
    let user = paths::user_config_path()
        .map(|p| {
            p.join(paths::config_file_name())
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_default();
    let project = paths::find_project_config().map(|p| p.to_string_lossy().to_string());
    axum::Json(ConfigPathResponse { user, project })
}
