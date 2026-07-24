pub mod channels;
pub mod config;
pub mod health;
pub mod messages;

use crate::DaemonState;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;

pub fn create_router(_auth_token: Option<String>, daemon_state: Arc<DaemonState>) -> Router {
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
        .with_state(daemon_state)
}

async fn root() -> &'static str {
    "oh-my-codes daemon"
}
