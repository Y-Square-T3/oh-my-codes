use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub url: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_expiry: i64,
    pub active_workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub id: String,
    pub email: String,
    pub url: String,
    pub active_workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub is_admin: bool,
}

impl Account {
    pub fn to_info(&self) -> AccountInfo {
        AccountInfo {
            id: self.id.clone(),
            email: self.email.clone(),
            url: self.url.clone(),
            active_workspace_id: self.active_workspace_id.clone(),
        }
    }
}
