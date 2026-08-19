use crate::server_client::{DeviceCodeResponse, OmcServerClient, PollResult};
use omc_core::account::{Account, AccountInfo, Workspace};
use omc_core::error::{OmcError, Result};
use omc_storage::StorageBackend;
use std::sync::Arc;

const EAGER_REFRESH_SECS: i64 = 300;

pub struct LoginSession {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub server_url: String,
    pub expires_at: i64,
    pub interval: i64,
}

pub struct AccountService {
    backend: Arc<dyn StorageBackend>,
    server_client: OmcServerClient,
}

impl AccountService {
    pub fn new(backend: Arc<dyn StorageBackend>, server_client: OmcServerClient) -> Self {
        Self {
            backend,
            server_client,
        }
    }

    pub async fn login(&self, server_url: &str) -> Result<LoginSession> {
        let normalized = omc_core::url::normalize_server_url(server_url)?;
        let resp: DeviceCodeResponse = self.server_client.request_device_code(&normalized).await?;
        let now = chrono::Utc::now().timestamp();
        Ok(LoginSession {
            device_code: resp.device_code,
            user_code: resp.user_code,
            verification_uri: resp.verification_uri,
            verification_uri_complete: resp.verification_uri_complete,
            server_url: normalized,
            expires_at: now + resp.expires_in,
            interval: resp.interval,
        })
    }

    pub async fn poll(&self, session: &LoginSession) -> Result<PollResult> {
        let now = chrono::Utc::now().timestamp();
        if now >= session.expires_at {
            return Ok(PollResult::Expired);
        }
        let result = self
            .server_client
            .poll_device_token(&session.server_url, &session.device_code)
            .await?;
        if let PollResult::Success {
            ref user_id,
            ref access_token,
            ref refresh_token,
            expires_in,
            ref email,
        } = result
        {
            let account = Account {
                id: user_id.clone(),
                email: email.clone(),
                url: session.server_url.clone(),
                access_token: access_token.clone(),
                refresh_token: refresh_token.clone(),
                token_expiry: now + expires_in,
                active_workspace_id: None,
            };
            self.backend.upsert_account(&account).await?;
            let mut workspaces = self
                .server_client
                .fetch_workspaces(&session.server_url, access_token)
                .await?;
            for w in &mut workspaces {
                w.account_id = account.id.clone();
            }
            self.backend.upsert_workspaces(&workspaces).await?;
            if workspaces.len() == 1 {
                self.backend
                    .set_active_workspace(&account.id, &workspaces[0].id)
                    .await?;
            }
            self.backend.set_active_account(&account.id).await?;
        }
        Ok(result)
    }

    pub async fn refresh_token(&self, account_id: &str) -> Result<String> {
        let account = self
            .backend
            .get_account(account_id)
            .await?
            .ok_or_else(|| OmcError::NotFound(format!("Account {account_id} not found")))?;
        let resp = self
            .server_client
            .refresh_token(&account.url, &account.refresh_token)
            .await?;
        let now = chrono::Utc::now().timestamp();
        let updated = Account {
            id: account.id.clone(),
            email: account.email,
            url: account.url,
            access_token: resp.access_token.clone(),
            refresh_token: resp.refresh_token,
            token_expiry: now + resp.expires_in,
            active_workspace_id: account.active_workspace_id,
        };
        self.backend.upsert_account(&updated).await?;
        Ok(resp.access_token)
    }

    pub async fn resolve_token(&self, account_id: &str) -> Result<String> {
        let account = self
            .backend
            .get_account(account_id)
            .await?
            .ok_or_else(|| OmcError::NotFound(format!("Account {account_id} not found")))?;
        let now = chrono::Utc::now().timestamp();
        if account.token_expiry - now < EAGER_REFRESH_SECS {
            return self.refresh_token(account_id).await;
        }
        Ok(account.access_token)
    }

    pub async fn active(&self) -> Result<Option<AccountInfo>> {
        let Some(id) = self.backend.get_active_account_id().await? else {
            return Ok(None);
        };
        let account = self.backend.get_account(&id).await?;
        Ok(account.map(|a| a.to_info()))
    }

    pub async fn active_with_token(&self) -> Result<Option<(AccountInfo, String)>> {
        let Some(id) = self.backend.get_active_account_id().await? else {
            return Ok(None);
        };
        let account = self
            .backend
            .get_account(&id)
            .await?
            .ok_or_else(|| OmcError::NotFound(format!("Account {id} not found")))?;
        let now = chrono::Utc::now().timestamp();
        let token = if account.token_expiry - now < EAGER_REFRESH_SECS {
            self.refresh_token(&id).await?
        } else {
            account.access_token.clone()
        };
        Ok(Some((account.to_info(), token)))
    }

    pub async fn list(&self) -> Result<Vec<(AccountInfo, Vec<Workspace>)>> {
        let accounts = self.backend.list_accounts().await?;
        let mut result = Vec::new();
        for account in accounts {
            let workspaces = self.backend.list_workspaces(&account.id).await?;
            result.push((account.to_info(), workspaces));
        }
        Ok(result)
    }

    pub async fn switch(&self, account_id: &str, workspace_id: &str) -> Result<()> {
        let account = self
            .backend
            .get_account(account_id)
            .await?
            .ok_or_else(|| OmcError::NotFound(format!("Account '{account_id}' not found")))?;

        let workspaces = self.backend.list_workspaces(account_id).await?;
        let workspace_exists = workspaces.iter().any(|w| w.id == workspace_id);
        if !workspace_exists {
            return Err(OmcError::NotFound(format!(
                "Workspace '{workspace_id}' not found for account '{}'",
                account.email
            )));
        }

        self.backend
            .set_active_workspace(account_id, workspace_id)
            .await?;
        self.backend.set_active_account(account_id).await?;
        Ok(())
    }

    pub async fn remove(&self, account_id: &str) -> Result<()> {
        let active_id = self.backend.get_active_account_id().await?;
        self.backend.clear_workspaces(account_id).await?;
        self.backend.delete_providers(account_id).await?;
        self.backend.delete_account(account_id).await?;
        if active_id.as_deref() == Some(account_id) {
            self.backend.clear_active_account().await?;
        }
        Ok(())
    }

    pub async fn workspaces(&self, account_id: &str) -> Result<Vec<Workspace>> {
        self.backend.list_workspaces(account_id).await
    }
}
