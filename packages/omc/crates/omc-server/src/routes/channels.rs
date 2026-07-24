use axum::Json;
use axum::extract::State;
use omc_api::types::*;
use std::sync::Arc;

use crate::DaemonState;

pub async fn list_handler(State(state): State<Arc<DaemonState>>) -> Json<ChannelsResponse> {
    let channels = state
        .message_store
        .list_channels()
        .await
        .unwrap_or_default();
    Json(ChannelsResponse { channels })
}

pub async fn create_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<CreateChannelRequest>,
) -> Json<CreateChannelResponse> {
    let channel = state
        .message_store
        .create_channel(&req.name)
        .await
        .unwrap_or(omc_core::types::Channel {
            id: String::new(),
            name: req.name,
            topic: None,
            created_at: 0,
        });
    Json(CreateChannelResponse { channel })
}
