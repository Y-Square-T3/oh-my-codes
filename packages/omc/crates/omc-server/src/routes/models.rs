use crate::DaemonState;
use crate::error::AppError;
use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ModelsListQuery {
    pub provider: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfoResponse {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub family: Option<String>,
    pub reasoning: Option<bool>,
    pub tool_call: Option<bool>,
    pub attachment: Option<bool>,
    pub temperature: Option<bool>,
    pub open_weights: Option<bool>,
    pub modalities_input: Vec<String>,
    pub modalities_output: Vec<String>,
    pub cost_input: f64,
    pub cost_output: f64,
    pub limit_context: Option<i64>,
    pub limit_output: Option<i64>,
    pub release_date: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfoResponse {
    pub id: String,
    pub name: String,
    pub api: Option<String>,
    pub npm: Option<String>,
    pub env: Vec<String>,
    pub model_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsListResponse {
    pub providers: Vec<ProviderInfoResponse>,
    pub models: Vec<ModelInfoResponse>,
    pub account_email: Option<String>,
    pub account_url: Option<String>,
}

pub async fn list_handler(
    State(state): State<Arc<DaemonState>>,
    Query(query): Query<ModelsListQuery>,
) -> std::result::Result<Json<ModelsListResponse>, AppError> {
    let result = state.model_service.list(query.provider.as_deref()).await?;
    Ok(Json(ModelsListResponse {
        providers: result
            .providers
            .into_iter()
            .map(|p| ProviderInfoResponse {
                id: p.id,
                name: p.name,
                api: p.api,
                npm: p.npm,
                env: p.env,
                model_count: p.model_count,
            })
            .collect(),
        models: result
            .models
            .into_iter()
            .map(|m| ModelInfoResponse {
                id: m.id,
                provider_id: m.provider_id,
                name: m.name,
                family: m.family,
                reasoning: m.reasoning,
                tool_call: m.tool_call,
                attachment: m.attachment,
                temperature: m.temperature,
                open_weights: m.open_weights,
                modalities_input: m.modalities_input,
                modalities_output: m.modalities_output,
                cost_input: m.cost_input,
                cost_output: m.cost_output,
                limit_context: m.limit_context,
                limit_output: m.limit_output,
                release_date: m.release_date,
            })
            .collect(),
        account_email: result.account_email,
        account_url: result.account_url,
    }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsSyncResponse {
    pub providers: usize,
    pub models: usize,
}

pub async fn sync_handler(
    State(state): State<Arc<DaemonState>>,
) -> std::result::Result<Json<ModelsSyncResponse>, AppError> {
    let result = state.model_service.sync().await?;
    Ok(Json(ModelsSyncResponse {
        providers: result.providers,
        models: result.models,
    }))
}
