use crate::account_store::AccountStore;
use crate::message_store::MessageStore;
use crate::workspace_store::WorkspaceStore;
use async_trait::async_trait;
use omc_core::account::{Account, Workspace};
use omc_core::error::{OmcError, Result};
use omc_core::types::{Channel, Message};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, Mem, RocksDb};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealChannel {
    id: String,
    name: String,
    topic: Option<String>,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealMessage {
    id: String,
    channel_id: String,
    author_id: String,
    content: String,
    timestamp: i64,
    edited_at: Option<i64>,
    reply_to: Option<String>,
}

pub struct SurrealStorage {
    db: Arc<Surreal<Db>>,
}

impl SurrealStorage {
    pub async fn new_memory() -> Result<Self> {
        let db = Surreal::new::<Mem>(())
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create in-memory SurrealDB: {e}")))?;
        db.use_ns("omc")
            .use_db("main")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to set namespace: {e}")))?;
        Self::init_schema(&db).await?;
        Ok(Self { db: Arc::new(db) })
    }

    pub async fn new_rocksdb(path: &Path) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(path)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create RocksDB SurrealDB: {e}")))?;
        db.use_ns("omc")
            .use_db("main")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to set namespace: {e}")))?;
        Self::init_schema(&db).await?;
        Ok(Self { db: Arc::new(db) })
    }

    async fn init_schema(db: &Surreal<Db>) -> Result<()> {
        let _result = db
            .query("DEFINE TABLE IF NOT EXISTS channel SCHEMALESS;")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to define channel table: {e}")))?;
        let _result = db
            .query("DEFINE TABLE IF NOT EXISTS message SCHEMALESS;")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to define message table: {e}")))?;
        let _result = db
            .query("DEFINE TABLE IF NOT EXISTS account SCHEMALESS;")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to define account table: {e}")))?;
        let _result = db
            .query("DEFINE TABLE IF NOT EXISTS workspace SCHEMALESS;")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to define workspace table: {e}")))?;
        let _result = db
            .query("DEFINE TABLE IF NOT EXISTS active_account SCHEMALESS;")
            .await
            .map_err(|e| {
                OmcError::Storage(format!("Failed to define active_account table: {e}"))
            })?;
        Ok(())
    }

    pub fn db(&self) -> Arc<Surreal<Db>> {
        self.db.clone()
    }
}

#[async_trait]
impl MessageStore for SurrealStorage {
    async fn create_channel(&self, name: &str) -> Result<Channel> {
        let now = chrono::Utc::now().timestamp();
        let channel = SurrealChannel {
            id: ulid::Ulid::new().to_string(),
            name: name.to_string(),
            topic: None,
            created_at: now,
        };
        let _: Option<SurrealChannel> = self
            .db
            .create("channel")
            .content(channel.clone())
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create channel: {e}")))?;
        Ok(Channel {
            id: channel.id,
            name: channel.name,
            topic: channel.topic,
            created_at: channel.created_at,
        })
    }

    async fn list_channels(&self) -> Result<Vec<Channel>> {
        let channels: Vec<SurrealChannel> = self
            .db
            .select("channel")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to list channels: {e}")))?;
        Ok(channels
            .into_iter()
            .map(|c| Channel {
                id: c.id,
                name: c.name,
                topic: c.topic,
                created_at: c.created_at,
            })
            .collect())
    }

    async fn send_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<Message> {
        let now = chrono::Utc::now().timestamp();
        let msg = SurrealMessage {
            id: ulid::Ulid::new().to_string(),
            channel_id: channel_id.to_string(),
            author_id: author_id.to_string(),
            content: content.to_string(),
            timestamp: now,
            edited_at: None,
            reply_to: None,
        };
        let _: Option<SurrealMessage> = self
            .db
            .create("message")
            .content(msg.clone())
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create message: {e}")))?;
        Ok(Message {
            id: msg.id,
            channel_id: msg.channel_id,
            author_id: msg.author_id,
            content: msg.content,
            timestamp: msg.timestamp,
            edited_at: msg.edited_at,
            reply_to: msg.reply_to,
        })
    }

    async fn get_messages(
        &self,
        channel_id: &str,
        limit: usize,
        before: Option<String>,
    ) -> Result<Vec<Message>> {
        let query = if let Some(before_id) = before {
            format!(
                "SELECT * FROM message WHERE channel_id = '{channel_id}' AND timestamp < (SELECT timestamp FROM message WHERE id = '{before_id}')[0].timestamp ORDER BY timestamp DESC LIMIT {limit};"
            )
        } else {
            format!(
                "SELECT * FROM message WHERE channel_id = '{channel_id}' ORDER BY timestamp DESC LIMIT {limit};"
            )
        };
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to query messages: {e}")))?;
        let messages: Vec<SurrealMessage> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract messages: {e}")))?;
        Ok(messages
            .into_iter()
            .map(|m| Message {
                id: m.id,
                channel_id: m.channel_id,
                author_id: m.author_id,
                content: m.content,
                timestamp: m.timestamp,
                edited_at: m.edited_at,
                reply_to: m.reply_to,
            })
            .collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealAccount {
    id: String,
    email: String,
    url: String,
    access_token: String,
    refresh_token: String,
    token_expiry: i64,
    active_workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealWorkspace {
    id: String,
    account_id: String,
    name: String,
    is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealActiveAccount {
    id: String,
    account_id: Option<String>,
}

pub struct SurrealAccountStore {
    db: Arc<Surreal<Db>>,
}

impl SurrealAccountStore {
    pub fn new(db: Arc<Surreal<Db>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl AccountStore for SurrealAccountStore {
    async fn get_account(&self, id: &str) -> Result<Option<Account>> {
        let query = format!("SELECT * FROM account WHERE id = '{id}';");
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to get account: {e}")))?;
        let accounts: Vec<SurrealAccount> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract account: {e}")))?;
        Ok(accounts.into_iter().next().map(|a| Account {
            id: a.id,
            email: a.email,
            url: a.url,
            access_token: a.access_token,
            refresh_token: a.refresh_token,
            token_expiry: a.token_expiry,
            active_workspace_id: a.active_workspace_id,
        }))
    }

    async fn list_accounts(&self) -> Result<Vec<Account>> {
        let accounts: Vec<SurrealAccount> = self
            .db
            .select("account")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to list accounts: {e}")))?;
        Ok(accounts
            .into_iter()
            .map(|a| Account {
                id: a.id,
                email: a.email,
                url: a.url,
                access_token: a.access_token,
                refresh_token: a.refresh_token,
                token_expiry: a.token_expiry,
                active_workspace_id: a.active_workspace_id,
            })
            .collect())
    }

    async fn upsert_account(&self, account: &Account) -> Result<()> {
        let dto = SurrealAccount {
            id: account.id.clone(),
            email: account.email.clone(),
            url: account.url.clone(),
            access_token: account.access_token.clone(),
            refresh_token: account.refresh_token.clone(),
            token_expiry: account.token_expiry,
            active_workspace_id: account.active_workspace_id.clone(),
        };
        let _: Option<SurrealAccount> = self
            .db
            .update(("account", &account.id))
            .content(dto)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to upsert account: {e}")))?;
        Ok(())
    }

    async fn delete_account(&self, id: &str) -> Result<()> {
        let _: Option<SurrealAccount> = self
            .db
            .delete(("account", id))
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to delete account: {e}")))?;
        Ok(())
    }

    async fn get_active_account_id(&self) -> Result<Option<String>> {
        let mut result = self
            .db
            .query("SELECT * FROM active_account WHERE id = 'active';")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to get active account: {e}")))?;
        let records: Vec<SurrealActiveAccount> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract active account: {e}")))?;
        Ok(records.into_iter().next().and_then(|r| r.account_id))
    }

    async fn set_active_account(&self, id: &str) -> Result<()> {
        let dto = SurrealActiveAccount {
            id: "active".to_string(),
            account_id: Some(id.to_string()),
        };
        let _: Option<SurrealActiveAccount> = self
            .db
            .upsert(("active_account", "active"))
            .content(dto)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to set active account: {e}")))?;
        Ok(())
    }

    async fn clear_active_account(&self) -> Result<()> {
        let dto = SurrealActiveAccount {
            id: "active".to_string(),
            account_id: None,
        };
        let _: Option<SurrealActiveAccount> = self
            .db
            .upsert(("active_account", "active"))
            .content(dto)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to clear active account: {e}")))?;
        Ok(())
    }

    async fn set_active_workspace(&self, account_id: &str, workspace_id: &str) -> Result<()> {
        let query = format!(
            "UPDATE account SET active_workspace_id = '{workspace_id}' WHERE id = '{account_id}';"
        );
        let _result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to set active workspace: {e}")))?;
        Ok(())
    }
}

pub struct SurrealWorkspaceStore {
    db: Arc<Surreal<Db>>,
}

impl SurrealWorkspaceStore {
    pub fn new(db: Arc<Surreal<Db>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl WorkspaceStore for SurrealWorkspaceStore {
    async fn list_workspaces(&self, account_id: &str) -> Result<Vec<Workspace>> {
        let query = format!("SELECT * FROM workspace WHERE account_id = '{account_id}';");
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to list workspaces: {e}")))?;
        let workspaces: Vec<SurrealWorkspace> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract workspaces: {e}")))?;
        Ok(workspaces
            .into_iter()
            .map(|w| Workspace {
                id: w.id,
                account_id: w.account_id,
                name: w.name,
                is_admin: w.is_admin,
            })
            .collect())
    }

    async fn upsert_workspaces(&self, workspaces: &[Workspace]) -> Result<()> {
        for w in workspaces {
            let dto = SurrealWorkspace {
                id: w.id.clone(),
                account_id: w.account_id.clone(),
                name: w.name.clone(),
                is_admin: w.is_admin,
            };
            let _: Option<SurrealWorkspace> = self
                .db
                .update(("workspace", &w.id))
                .content(dto)
                .await
                .map_err(|e| OmcError::Storage(format!("Failed to upsert workspace: {e}")))?;
        }
        Ok(())
    }

    async fn clear_workspaces(&self, account_id: &str) -> Result<()> {
        let query = format!("DELETE FROM workspace WHERE account_id = '{account_id}';");
        let _result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to clear workspaces: {e}")))?;
        Ok(())
    }
}
