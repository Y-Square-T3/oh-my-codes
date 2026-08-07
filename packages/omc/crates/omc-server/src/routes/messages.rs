use axum::Json;
use axum::extract::{Path, Query, State};
use omc_api::types::*;
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
    Path(channel_id): Path<String>,
    Query(query): Query<MessagesQuery>,
) -> Json<MessagesResponse> {
    let messages = state
        .backend
        .get_messages(&channel_id, query.limit.unwrap_or(50), query.before)
        .await
        .unwrap_or_default();
    Json(MessagesResponse { messages })
}

pub async fn send_handler(
    State(state): State<Arc<DaemonState>>,
    Path(channel_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> Json<SendMessageResponse> {
    let message = state
        .backend
        .send_message(&channel_id, "anonymous", &req.content)
        .await
        .unwrap_or(omc_core::types::Message {
            id: String::new(),
            channel_id,
            author_id: "anonymous".to_string(),
            content: req.content,
            timestamp: 0,
            edited_at: None,
            reply_to: None,
        });
    Json(SendMessageResponse { message })
}
