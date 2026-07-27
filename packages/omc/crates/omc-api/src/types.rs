use omc_core::types::{Channel, Message};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedDaemonConfigJson {
    pub bind_addr: String,
    pub bind_port: u16,
    pub socket_path: String,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub daemon: ResolvedDaemonConfigJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPathResponse {
    pub user: String,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChannelRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChannelResponse {
    pub channel: Channel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelsResponse {
    pub channels: Vec<Channel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesResponse {
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_at: i64,
    pub interval: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollRequest {
    pub device_code: String,
    pub server_url: String,
    pub expires_at: i64,
    pub interval: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfoResponse {
    pub id: String,
    pub email: String,
    pub url: String,
    pub active_workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveResponse {
    pub account: Option<AccountInfoResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceResponse {
    pub id: String,
    pub name: String,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithWorkspaces {
    pub account: AccountInfoResponse,
    pub workspaces: Vec<WorkspaceResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResponse {
    pub accounts: Vec<AccountWithWorkspaces>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchRequest {
    pub account_id: String,
    pub workspace_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveRequest {
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacesResponse {
    pub workspaces: Vec<WorkspaceResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsListQuery {
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfoResponse {
    pub id: String,
    pub name: String,
    pub api: Option<String>,
    pub npm: Option<String>,
    pub model_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsListResponse {
    pub providers: Vec<ProviderInfoResponse>,
    pub models: Vec<ModelInfoResponse>,
    pub account_email: Option<String>,
    pub account_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsSyncResponse {
    pub providers: usize,
    pub models: usize,
}
