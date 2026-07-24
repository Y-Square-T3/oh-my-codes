use axum::Json;
use axum::extract::{Path, Query, State};
use omc_api::types::*;
use omc_core::path_encode::decode_repo_path;
use serde::Deserialize;
use std::sync::Arc;

use crate::DaemonState;

#[derive(Debug, Deserialize)]
pub struct MessagesQuery {
    pub limit: Option<usize>,
    pub before: Option<String>,
}

pub async fn list_handler(
    State(state): State<Arc<DaemonState>>,
    Path((repo_path, channel_id)): Path<(String, String)>,
    Query(query): Query<MessagesQuery>,
) -> Json<MessagesResponse> {
    let decoded = decode_repo_path(&repo_path);
    let store = state.message_store_for(&decoded).await;
    let messages = if let Some(store) = store {
        store
            .get_messages(&channel_id, query.limit.unwrap_or(50), query.before)
            .await
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    Json(MessagesResponse { messages })
}

pub async fn send_handler(
    State(state): State<Arc<DaemonState>>,
    Path((repo_path, channel_id)): Path<(String, String)>,
    Json(req): Json<SendMessageRequest>,
) -> Json<SendMessageResponse> {
    let decoded = decode_repo_path(&repo_path);
    let store = state.message_store_for(&decoded).await;
    let message = if let Some(store) = store {
        store
            .send_message(&channel_id, "anonymous", &req.content)
            .await
            .ok()
    } else {
        None
    };
    Json(SendMessageResponse {
        message: message.unwrap_or(omc_core::types::Message {
            id: String::new(),
            channel_id,
            author_id: "anonymous".to_string(),
            content: req.content,
            timestamp: 0,
            edited_at: None,
            reply_to: None,
        }),
    })
}
