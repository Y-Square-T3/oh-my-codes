use axum::Json;
use axum::extract::{Path, State};
use omc_api::types::*;
use omc_core::path_encode::decode_repo_path;
use std::sync::Arc;

use crate::DaemonState;

pub async fn list_handler(
    State(state): State<Arc<DaemonState>>,
    Path(repo_path): Path<String>,
) -> Json<ChannelsResponse> {
    let decoded = decode_repo_path(&repo_path);
    let store = state.message_store_for(&decoded).await;
    let channels = if let Some(store) = store {
        store.list_channels().await.unwrap_or_default()
    } else {
        Vec::new()
    };
    Json(ChannelsResponse { channels })
}

pub async fn create_handler(
    State(state): State<Arc<DaemonState>>,
    Path(repo_path): Path<String>,
    Json(req): Json<CreateChannelRequest>,
) -> Json<CreateChannelResponse> {
    let decoded = decode_repo_path(&repo_path);
    let store = state.message_store_for(&decoded).await;
    let channel = if let Some(store) = store {
        store.create_channel(&req.name).await.ok()
    } else {
        None
    };
    Json(CreateChannelResponse {
        channel: channel.unwrap_or(omc_core::types::Channel {
            id: String::new(),
            name: req.name,
            topic: None,
            created_at: 0,
        }),
    })
}
