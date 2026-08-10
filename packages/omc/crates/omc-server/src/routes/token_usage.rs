use crate::DaemonState;
use crate::error::AppError;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use omc_core::token_usage::{TokenUsage, TokenUsageOverview, UsageSummary, generate_id};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordTokenUsageRequest {
    pub agent: String,
    pub model: String,
    pub session_id: String,
    pub message_id: String,
    #[serde(default)]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    #[serde(default)]
    pub reasoning_tokens: i64,
    #[serde(default)]
    pub cache_read_tokens: i64,
    #[serde(default)]
    pub cache_write_tokens: i64,
    #[serde(default)]
    pub audio_input_tokens: i64,
    #[serde(default)]
    pub video_input_tokens: i64,
    #[serde(default)]
    pub image_input_tokens: i64,
    #[serde(default)]
    pub recorded_at: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageStatusResponse {
    pub unpushed_count: usize,
    pub has_active_account: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsagePushResponse {
    pub pushed_count: usize,
    pub failed_count: usize,
    pub total_batches: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageRecordResponse {
    pub id: String,
    pub workspace_id: Option<String>,
    pub session_id: String,
    pub agent: String,
    pub model: String,
    pub metadata: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub audio_input_tokens: i64,
    pub video_input_tokens: i64,
    pub image_input_tokens: i64,
    pub total_tokens: i64,
    pub pushed: bool,
    pub recorded_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageListResponse {
    pub records: Vec<TokenUsageRecordResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummaryResponse {
    pub model: String,
    pub total_input: i64,
    pub total_output: i64,
    pub total_reasoning: i64,
    pub total_cache_read: i64,
    pub total_cache_write: i64,
    pub request_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageSummaryResponse {
    pub items: Vec<UsageSummaryResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub pushed: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryQuery {
    pub days: Option<i64>,
}

pub async fn record_handler(
    State(state): State<Arc<DaemonState>>,
    Json(body): Json<RecordTokenUsageRequest>,
) -> std::result::Result<impl IntoResponse, AppError> {
    let now_ms = chrono::Utc::now().timestamp_millis();
    let id = generate_id(&body.agent, &body.message_id);
    let total_tokens = body.input_tokens
        + body.output_tokens
        + body.reasoning_tokens
        + body.cache_read_tokens
        + body.cache_write_tokens
        + body.audio_input_tokens
        + body.video_input_tokens
        + body.image_input_tokens;
    let metadata = body
        .metadata
        .unwrap_or_else(|| serde_json::json!({ "messageId": body.message_id }).to_string());
    let usage = TokenUsage {
        id,
        workspace_id: body.workspace_id,
        session_id: body.session_id,
        agent: body.agent,
        model: body.model,
        metadata: Some(metadata),
        input_tokens: body.input_tokens,
        output_tokens: body.output_tokens,
        reasoning_tokens: body.reasoning_tokens,
        cache_read_tokens: body.cache_read_tokens,
        cache_write_tokens: body.cache_write_tokens,
        audio_input_tokens: body.audio_input_tokens,
        video_input_tokens: body.video_input_tokens,
        image_input_tokens: body.image_input_tokens,
        total_tokens,
        pushed: false,
        recorded_at: body.recorded_at.unwrap_or(now_ms),
        created_at: now_ms,
    };
    state.token_usage_service.record(&usage).await?;
    Ok(StatusCode::CREATED)
}

pub async fn status_handler(
    State(state): State<Arc<DaemonState>>,
) -> std::result::Result<Json<TokenUsageStatusResponse>, AppError> {
    let result = state.token_usage_service.status().await?;
    Ok(Json(TokenUsageStatusResponse {
        unpushed_count: result.unpushed_count,
        has_active_account: result.has_active_account,
    }))
}

pub async fn push_handler(
    State(state): State<Arc<DaemonState>>,
) -> std::result::Result<Json<TokenUsagePushResponse>, AppError> {
    let result = state.token_usage_service.push_batch(None).await?;
    Ok(Json(TokenUsagePushResponse {
        pushed_count: result.pushed_count,
        failed_count: result.failed_count,
        total_batches: result.total_batches,
    }))
}

pub async fn list_handler(
    State(state): State<Arc<DaemonState>>,
    Query(query): Query<ListQuery>,
) -> std::result::Result<Json<TokenUsageListResponse>, AppError> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let pushed = query.pushed;
    let records = state
        .token_usage_service
        .list_recent(limit, offset, pushed)
        .await?;
    let total = state.token_usage_service.count_all(pushed).await?;
    let response_records = records.into_iter().map(to_record_response).collect();
    Ok(Json(TokenUsageListResponse {
        records: response_records,
        total,
    }))
}

pub async fn summary_handler(
    State(state): State<Arc<DaemonState>>,
    Query(query): Query<SummaryQuery>,
) -> std::result::Result<Json<TokenUsageSummaryResponse>, AppError> {
    let items = state.token_usage_service.summary(query.days).await?;
    let response_items = items.into_iter().map(to_summary_response).collect();
    Ok(Json(TokenUsageSummaryResponse {
        items: response_items,
    }))
}

pub async fn overview_handler(
    State(state): State<Arc<DaemonState>>,
    Query(query): Query<SummaryQuery>,
) -> std::result::Result<Json<TokenUsageOverview>, AppError> {
    let overview = state.token_usage_service.overview(query.days).await?;
    Ok(Json(overview))
}

fn to_record_response(u: TokenUsage) -> TokenUsageRecordResponse {
    TokenUsageRecordResponse {
        id: u.id,
        workspace_id: u.workspace_id,
        session_id: u.session_id,
        agent: u.agent,
        model: u.model,
        metadata: u.metadata,
        input_tokens: u.input_tokens,
        output_tokens: u.output_tokens,
        reasoning_tokens: u.reasoning_tokens,
        cache_read_tokens: u.cache_read_tokens,
        cache_write_tokens: u.cache_write_tokens,
        audio_input_tokens: u.audio_input_tokens,
        video_input_tokens: u.video_input_tokens,
        image_input_tokens: u.image_input_tokens,
        total_tokens: u.total_tokens,
        pushed: u.pushed,
        recorded_at: u.recorded_at,
        created_at: u.created_at,
    }
}

fn to_summary_response(s: UsageSummary) -> UsageSummaryResponse {
    UsageSummaryResponse {
        model: s.model,
        total_input: s.total_input,
        total_output: s.total_output,
        total_reasoning: s.total_reasoning,
        total_cache_read: s.total_cache_read,
        total_cache_write: s.total_cache_write,
        request_count: s.request_count,
    }
}
