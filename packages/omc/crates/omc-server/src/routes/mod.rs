pub mod account;
pub mod channels;
pub mod config;
pub mod health;
pub mod messages;
pub mod models;
pub mod token_usage;

use crate::DaemonState;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn create_router(daemon_state: Arc<DaemonState>) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health", get(health::handler))
        .route("/config", get(config::handler))
        .route("/config/path", get(config::path_handler))
        .route(
            "/channels",
            get(channels::list_handler).post(channels::create_handler),
        )
        .route(
            "/channels/{channel_id}/messages",
            get(messages::list_handler).post(messages::send_handler),
        )
        .route("/account/login", post(account::login_handler))
        .route("/account/poll", post(account::poll_handler))
        .route("/account/active", get(account::active_handler))
        .route("/account/list", get(account::list_handler))
        .route("/account/switch", post(account::switch_handler))
        .route("/account/remove", post(account::remove_handler))
        .route("/account/workspaces", get(account::workspaces_handler))
        .route("/models", get(models::list_handler))
        .route("/models/sync", post(models::sync_handler))
        .route("/token-usage", post(token_usage::record_handler))
        .route("/token-usage/status", get(token_usage::status_handler))
        .route("/token-usage/push", post(token_usage::push_handler))
        .route("/token-usage/list", get(token_usage::list_handler))
        .route("/token-usage/summary", get(token_usage::summary_handler))
        .with_state(daemon_state)
}

async fn root() -> &'static str {
    "oh-my-codes daemon"
}
