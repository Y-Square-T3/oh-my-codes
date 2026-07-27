use omc_core::account::Workspace;
use omc_core::error::{OmcError, Result};
use serde::{Deserialize, Serialize};

const CLIENT_ID: &str = "oh-my-codes";

#[derive(Debug, Clone, Serialize)]
struct DeviceCodeRequest {
    client_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: i64,
    pub interval: i64,
}

#[derive(Debug, Clone, Serialize)]
struct DeviceTokenRequest {
    grant_type: String,
    device_code: String,
    client_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct RefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceTokenSuccess {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct DeviceTokenError {
    error: String,
    error_description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PollResult {
    Success {
        user_id: String,
        access_token: String,
        refresh_token: String,
        expires_in: i64,
        email: String,
    },
    Pending,
    Slow,
    Expired,
    Denied,
    Error(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
struct WorkspaceResponse {
    id: String,
    name: String,
    is_admin: bool,
}

pub struct OmcServerClient {
    client: reqwest::Client,
}

impl Default for OmcServerClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OmcServerClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn request_device_code(&self, server_url: &str) -> Result<DeviceCodeResponse> {
        let url = format!("{server_url}/auth/device/code");
        let body = DeviceCodeRequest {
            client_id: CLIENT_ID.to_string(),
        };
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to request device code: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(OmcError::Auth(format!(
                "Device code request failed ({status}): {text}"
            )));
        }
        resp.json()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to parse device code response: {e}")))
    }

    pub async fn poll_device_token(
        &self,
        server_url: &str,
        device_code: &str,
    ) -> Result<PollResult> {
        let url = format!("{server_url}/auth/device/token");
        let body = DeviceTokenRequest {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            device_code: device_code.to_string(),
            client_id: CLIENT_ID.to_string(),
        };
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to poll device token: {e}")))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to read poll response: {e}")))?;
        if !status.is_success() {
            let err: DeviceTokenError = serde_json::from_str(&text)
                .map_err(|e| OmcError::Auth(format!("Failed to parse poll error response: {e}")))?;
            return match err.error.as_str() {
                "authorization_pending" => Ok(PollResult::Pending),
                "slow_down" => Ok(PollResult::Slow),
                "expired_token" => Ok(PollResult::Expired),
                "access_denied" => Ok(PollResult::Denied),
                _ => Ok(PollResult::Error(
                    err.error_description.unwrap_or(err.error),
                )),
            };
        }
        let token: DeviceTokenSuccess = serde_json::from_str(&text)
            .map_err(|e| OmcError::Auth(format!("Failed to parse token response: {e}")))?;
        let user = self.fetch_user(server_url, &token.access_token).await?;
        Ok(PollResult::Success {
            user_id: user.id,
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_in: token.expires_in,
            email: user.email,
        })
    }

    pub async fn refresh_token(
        &self,
        server_url: &str,
        refresh_token: &str,
    ) -> Result<DeviceTokenSuccess> {
        let url = format!("{server_url}/auth/device/token");
        let body = RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            client_id: CLIENT_ID.to_string(),
        };
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to refresh token: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(OmcError::Auth(format!(
                "Token refresh failed ({status}): {text}"
            )));
        }
        resp.json()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to parse refresh token response: {e}")))
    }

    pub async fn fetch_user(&self, server_url: &str, access_token: &str) -> Result<UserInfo> {
        let url = format!("{server_url}/api/me");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to fetch user: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(OmcError::Auth(format!(
                "Fetch user failed ({status}): {text}"
            )));
        }
        resp.json()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to parse user response: {e}")))
    }

    pub async fn fetch_workspaces(
        &self,
        server_url: &str,
        access_token: &str,
    ) -> Result<Vec<Workspace>> {
        let url = format!("{server_url}/api/workspaces");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to fetch workspaces: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(OmcError::Auth(format!(
                "Fetch workspaces failed ({status}): {text}"
            )));
        }
        let workspaces: Vec<WorkspaceResponse> = resp
            .json()
            .await
            .map_err(|e| OmcError::Auth(format!("Failed to parse workspaces response: {e}")))?;
        Ok(workspaces
            .into_iter()
            .map(|w| Workspace {
                id: w.id,
                account_id: String::new(),
                name: w.name,
                is_admin: w.is_admin,
            })
            .collect())
    }
}
