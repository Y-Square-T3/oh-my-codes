use axum::Json;
use axum::extract::State;
use omc_api::types::*;
use std::sync::Arc;

use crate::DaemonState;

pub async fn list_handler(State(state): State<Arc<DaemonState>>) -> Json<RepoListResponse> {
    let config = state.config().await;
    Json(RepoListResponse {
        repos: config.repos,
    })
}

pub async fn add_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<RepoAddRequest>,
) -> Json<RepoAddResponse> {
    let _ = state.add_repo(req.path.clone()).await;
    Json(RepoAddResponse {
        status: "ok".to_string(),
        path: req.path,
    })
}

pub async fn remove_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<RepoRemoveRequest>,
) -> Json<RepoRemoveResponse> {
    let _ = state.remove_repo(req.path.clone()).await;
    Json(RepoRemoveResponse {
        status: "ok".to_string(),
        path: req.path,
    })
}
