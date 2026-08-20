use crate::DaemonState;
use crate::error::AppError;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_at: i64,
    pub interval: i64,
}

pub async fn login_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<LoginRequest>,
) -> std::result::Result<Json<LoginResponse>, AppError> {
    let session = state.account_service.login(&req.url).await?;
    Ok(Json(LoginResponse {
        device_code: session.device_code,
        user_code: session.user_code,
        verification_uri: session.verification_uri,
        verification_uri_complete: session.verification_uri_complete,
        expires_at: session.expires_at,
        interval: session.interval,
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollRequest {
    pub device_code: String,
    pub server_url: String,
    pub expires_at: i64,
    pub interval: i64,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum PollResponse {
    #[serde(rename = "success")]
    Success { email: String },
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "slow")]
    Slow,
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "denied")]
    Denied,
    #[serde(rename = "error")]
    Error { message: String },
}

pub async fn poll_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<PollRequest>,
) -> std::result::Result<Json<PollResponse>, AppError> {
    let session = crate::account_service::LoginSession {
        device_code: req.device_code,
        user_code: String::new(),
        verification_uri: String::new(),
        verification_uri_complete: String::new(),
        server_url: req.server_url,
        expires_at: req.expires_at,
        interval: req.interval,
    };
    let result = state.account_service.poll(&session).await?;
    Ok(Json(match result {
        crate::server_client::PollResult::Success { email, .. } => PollResponse::Success { email },
        crate::server_client::PollResult::Pending => PollResponse::Pending,
        crate::server_client::PollResult::Slow => PollResponse::Slow,
        crate::server_client::PollResult::Expired => PollResponse::Expired,
        crate::server_client::PollResult::Denied => PollResponse::Denied,
        crate::server_client::PollResult::Error(msg) => PollResponse::Error { message: msg },
    }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfoResponse {
    pub id: String,
    pub email: String,
    pub url: String,
    pub active_workspace_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ActiveResponse {
    pub account: Option<AccountInfoResponse>,
}

pub async fn active_handler(
    State(state): State<Arc<DaemonState>>,
) -> std::result::Result<Json<ActiveResponse>, AppError> {
    let account = state
        .account_service
        .active()
        .await?
        .map(|a| AccountInfoResponse {
            id: a.id,
            email: a.email,
            url: a.url,
            active_workspace_id: a.active_workspace_id,
        });
    Ok(Json(ActiveResponse { account }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceResponse {
    pub id: String,
    pub name: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize)]
pub struct AccountWithWorkspaces {
    pub account: AccountInfoResponse,
    pub workspaces: Vec<WorkspaceResponse>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub accounts: Vec<AccountWithWorkspaces>,
}

pub async fn list_handler(
    State(state): State<Arc<DaemonState>>,
) -> std::result::Result<Json<ListResponse>, AppError> {
    let accounts = state.account_service.list().await?;
    Ok(Json(ListResponse {
        accounts: accounts
            .into_iter()
            .map(|(a, ws)| AccountWithWorkspaces {
                account: AccountInfoResponse {
                    id: a.id,
                    email: a.email,
                    url: a.url,
                    active_workspace_id: a.active_workspace_id,
                },
                workspaces: ws
                    .into_iter()
                    .map(|w| WorkspaceResponse {
                        id: w.id,
                        name: w.name,
                        is_admin: w.is_admin,
                    })
                    .collect(),
            })
            .collect(),
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchRequest {
    pub account_id: String,
    pub workspace_id: String,
}

pub async fn switch_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<SwitchRequest>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    state
        .account_service
        .switch(&req.account_id, &req.workspace_id)
        .await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveRequest {
    pub account_id: String,
}

pub async fn remove_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<RemoveRequest>,
) -> std::result::Result<Json<serde_json::Value>, AppError> {
    state.account_service.remove(&req.account_id).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesQuery {
    pub account_id: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspacesResponse {
    pub workspaces: Vec<WorkspaceResponse>,
}

pub async fn workspaces_handler(
    State(state): State<Arc<DaemonState>>,
    axum::extract::Query(query): axum::extract::Query<WorkspacesQuery>,
) -> std::result::Result<Json<WorkspacesResponse>, AppError> {
    let workspaces = state.account_service.workspaces(&query.account_id).await?;
    Ok(Json(WorkspacesResponse {
        workspaces: workspaces
            .into_iter()
            .map(|w| WorkspaceResponse {
                id: w.id,
                name: w.name,
                is_admin: w.is_admin,
            })
            .collect(),
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequest {
    pub account_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenResponse {
    pub email: String,
    pub token_expiry: i64,
}

pub async fn refresh_token_handler(
    State(state): State<Arc<DaemonState>>,
    Json(req): Json<RefreshTokenRequest>,
) -> std::result::Result<Json<RefreshTokenResponse>, AppError> {
    let account_id = match req.account_id {
        Some(id) => id,
        None => state
            .backend
            .get_active_account_id()
            .await?
            .ok_or_else(|| {
                omc_core::error::OmcError::Auth(
                    "No active account. Run `omc account login` first.".into(),
                )
            })?,
    };
    let updated = state.account_service.refresh_token(&account_id).await?;
    Ok(Json(RefreshTokenResponse {
        email: updated.email,
        token_expiry: updated.token_expiry,
    }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialsResponse {
    pub api_key: String,
    pub base_url: String,
    pub workspace_id: Option<String>,
}

pub async fn credentials_handler(
    State(state): State<Arc<DaemonState>>,
) -> std::result::Result<Json<CredentialsResponse>, AppError> {
    let (account, token) = state
        .account_service
        .active_with_token()
        .await?
        .ok_or_else(|| {
            omc_core::error::OmcError::Auth(
                "No active account. Run `omc account login` first.".into(),
            )
        })?;
    let base_url = format!("{}/api/v2", account.url.trim_end_matches('/'));
    Ok(Json(CredentialsResponse {
        api_key: token,
        base_url,
        workspace_id: account.active_workspace_id,
    }))
}
