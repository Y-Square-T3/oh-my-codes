use axum::extract::State;
use omc_api::types::HealthResponse;
use std::sync::Arc;

use crate::DaemonState;

pub async fn handler(State(_state): State<Arc<DaemonState>>) -> axum::Json<HealthResponse> {
    axum::Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
