use crate::account_store::AccountStore;
use crate::message_store::MessageStore;
use crate::model_store::ModelStore;
use crate::token_usage_store::TokenUsageStore;
use crate::workspace_store::WorkspaceStore;
use async_trait::async_trait;
use omc_core::account::{Account, Workspace};
use omc_core::error::{OmcError, Result};
use omc_core::model::Provider;
use omc_core::token_usage::{TokenUsage, UsageSummary};
use omc_core::types::{Channel, Message};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use surrealdb::engine::local::{Db, Mem, RocksDb};
use surrealdb::{RecordId, Surreal};

fn extract_id(rid: &RecordId) -> Result<String> {
    String::try_from(rid.key().clone())
        .map_err(|_| OmcError::Storage("Record ID key is not a string".into()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealChannel {
    #[serde(skip_serializing)]
    id: RecordId,
    name: String,
    topic: Option<String>,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealMessage {
    #[serde(skip_serializing)]
    id: RecordId,
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
        let _result = db
            .query("DEFINE TABLE IF NOT EXISTS provider SCHEMALESS;")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to define provider table: {e}")))?;
        let _result = db
            .query("DEFINE TABLE IF NOT EXISTS token_usage SCHEMALESS;")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to define token_usage table: {e}")))?;
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
        let id = ulid::Ulid::new().to_string();
        let dto = SurrealChannel {
            id: ("channel", id.as_str()).into(),
            name: name.to_string(),
            topic: None,
            created_at: now,
        };
        let _: Option<SurrealChannel> = self
            .db
            .create(("channel", id.as_str()))
            .content(dto)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create channel: {e}")))?;
        Ok(Channel {
            id,
            name: name.to_string(),
            topic: None,
            created_at: now,
        })
    }

    async fn list_channels(&self) -> Result<Vec<Channel>> {
        let channels: Vec<SurrealChannel> = self
            .db
            .select("channel")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to list channels: {e}")))?;
        let mut result = Vec::with_capacity(channels.len());
        for c in channels {
            result.push(Channel {
                id: extract_id(&c.id)?,
                name: c.name,
                topic: c.topic,
                created_at: c.created_at,
            });
        }
        Ok(result)
    }

    async fn send_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<Message> {
        let now = chrono::Utc::now().timestamp();
        let id = ulid::Ulid::new().to_string();
        let dto = SurrealMessage {
            id: ("message", id.as_str()).into(),
            channel_id: channel_id.to_string(),
            author_id: author_id.to_string(),
            content: content.to_string(),
            timestamp: now,
            edited_at: None,
            reply_to: None,
        };
        let _: Option<SurrealMessage> = self
            .db
            .create(("message", id.as_str()))
            .content(dto)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create message: {e}")))?;
        Ok(Message {
            id,
            channel_id: channel_id.to_string(),
            author_id: author_id.to_string(),
            content: content.to_string(),
            timestamp: now,
            edited_at: None,
            reply_to: None,
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
        let mut result = Vec::with_capacity(messages.len());
        for m in messages {
            result.push(Message {
                id: extract_id(&m.id)?,
                channel_id: m.channel_id,
                author_id: m.author_id,
                content: m.content,
                timestamp: m.timestamp,
                edited_at: m.edited_at,
                reply_to: m.reply_to,
            });
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealAccount {
    #[serde(skip_serializing)]
    id: RecordId,
    email: String,
    url: String,
    access_token: String,
    refresh_token: String,
    token_expiry: i64,
    active_workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealWorkspace {
    #[serde(skip_serializing)]
    id: RecordId,
    account_id: String,
    name: String,
    is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealActiveAccount {
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
        let account: Option<SurrealAccount> = self
            .db
            .select(("account", id))
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to get account: {e}")))?;
        account
            .map(|a| {
                Ok(Account {
                    id: extract_id(&a.id)?,
                    email: a.email,
                    url: a.url,
                    access_token: a.access_token,
                    refresh_token: a.refresh_token,
                    token_expiry: a.token_expiry,
                    active_workspace_id: a.active_workspace_id,
                })
            })
            .transpose()
    }

    async fn list_accounts(&self) -> Result<Vec<Account>> {
        let accounts: Vec<SurrealAccount> = self
            .db
            .select("account")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to list accounts: {e}")))?;
        let mut result = Vec::with_capacity(accounts.len());
        for a in accounts {
            result.push(Account {
                id: extract_id(&a.id)?,
                email: a.email,
                url: a.url,
                access_token: a.access_token,
                refresh_token: a.refresh_token,
                token_expiry: a.token_expiry,
                active_workspace_id: a.active_workspace_id,
            });
        }
        Ok(result)
    }

    async fn upsert_account(&self, account: &Account) -> Result<()> {
        let dto = SurrealAccount {
            id: ("account", account.id.as_str()).into(),
            email: account.email.clone(),
            url: account.url.clone(),
            access_token: account.access_token.clone(),
            refresh_token: account.refresh_token.clone(),
            token_expiry: account.token_expiry,
            active_workspace_id: account.active_workspace_id.clone(),
        };
        let _: Option<SurrealAccount> = self
            .db
            .upsert(("account", account.id.as_str()))
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
        let record: Option<SurrealActiveAccount> = self
            .db
            .select(("active_account", "active"))
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to get active account: {e}")))?;
        Ok(record.and_then(|r| r.account_id))
    }

    async fn set_active_account(&self, id: &str) -> Result<()> {
        let dto = SurrealActiveAccount {
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
        let dto = SurrealActiveAccount { account_id: None };
        let _: Option<SurrealActiveAccount> = self
            .db
            .upsert(("active_account", "active"))
            .content(dto)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to clear active account: {e}")))?;
        Ok(())
    }

    async fn set_active_workspace(&self, account_id: &str, workspace_id: &str) -> Result<()> {
        let account: Option<SurrealAccount> = self
            .db
            .select(("account", account_id))
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to get account: {e}")))?;

        let mut account = account
            .ok_or_else(|| OmcError::Storage(format!("Account '{account_id}' not found")))?;

        account.active_workspace_id = Some(workspace_id.to_string());

        let _: Option<SurrealAccount> = self
            .db
            .upsert(("account", account_id))
            .content(account)
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
        let mut ws_list = Vec::with_capacity(workspaces.len());
        for w in workspaces {
            ws_list.push(Workspace {
                id: extract_id(&w.id)?,
                account_id: w.account_id,
                name: w.name,
                is_admin: w.is_admin,
            });
        }
        Ok(ws_list)
    }

    async fn upsert_workspaces(&self, workspaces: &[Workspace]) -> Result<()> {
        for w in workspaces {
            let dto = SurrealWorkspace {
                id: ("workspace", w.id.as_str()).into(),
                account_id: w.account_id.clone(),
                name: w.name.clone(),
                is_admin: w.is_admin,
            };
            let _: Option<SurrealWorkspace> = self
                .db
                .upsert(("workspace", w.id.as_str()))
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealProvider {
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    id: RecordId,
    provider_id: String,
    name: String,
    env: Vec<String>,
    api: Option<String>,
    npm: Option<String>,
    doc: Option<String>,
    models: Vec<omc_core::model::Model>,
    account_id: String,
    last_fetched_at: i64,
}

fn provider_record_key(provider_id: &str, account_id: &str) -> String {
    format!("{provider_id}:{account_id}")
}

pub struct SurrealModelStore {
    db: Arc<Surreal<Db>>,
}

impl SurrealModelStore {
    pub fn new(db: Arc<Surreal<Db>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ModelStore for SurrealModelStore {
    async fn list_providers(&self, account_id: &str) -> Result<Vec<Provider>> {
        let query = format!("SELECT * FROM provider WHERE account_id = '{account_id}';");
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to list providers: {e}")))?;
        let rows: Vec<SurrealProvider> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract providers: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| Provider {
                id: r.provider_id,
                name: r.name,
                env: r.env,
                api: r.api,
                npm: r.npm,
                doc: r.doc,
                models: r.models,
                account_id: r.account_id,
                last_fetched_at: r.last_fetched_at,
            })
            .collect())
    }

    async fn replace_providers(&self, account_id: &str, providers: Vec<Provider>) -> Result<()> {
        self.delete_providers(account_id).await?;
        for p in providers {
            let record_key = provider_record_key(&p.id, account_id);
            let dto = SurrealProvider {
                id: ("provider", record_key.as_str()).into(),
                provider_id: p.id,
                name: p.name,
                env: p.env,
                api: p.api,
                npm: p.npm,
                doc: p.doc,
                models: p.models,
                account_id: p.account_id,
                last_fetched_at: p.last_fetched_at,
            };
            let _: Option<SurrealProvider> = self
                .db
                .create(("provider", record_key.as_str()))
                .content(dto)
                .await
                .map_err(|e| OmcError::Storage(format!("Failed to insert provider: {e}")))?;
        }
        Ok(())
    }

    async fn delete_providers(&self, account_id: &str) -> Result<()> {
        let query = format!("DELETE FROM provider WHERE account_id = '{account_id}';");
        let _result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to delete providers: {e}")))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealTokenUsage {
    #[serde(skip_serializing)]
    id: RecordId,
    client: String,
    session_id: String,
    message_id: String,
    provider_id: String,
    model_id: String,
    input_tokens: i64,
    output_tokens: i64,
    reasoning_tokens: i64,
    cache_read_tokens: i64,
    cache_write_tokens: i64,
    pushed: bool,
    recorded_at: i64,
    created_at: i64,
}

pub struct SurrealTokenUsageStore {
    db: Arc<Surreal<Db>>,
}

impl SurrealTokenUsageStore {
    pub fn new(db: Arc<Surreal<Db>>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TokenUsageStore for SurrealTokenUsageStore {
    async fn upsert(&self, usage: &TokenUsage) -> Result<()> {
        let query = format!(
            "SELECT * FROM token_usage WHERE message_id = '{}' LIMIT 1;",
            usage.message_id
        );
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to query token_usage: {e}")))?;
        let existing: Option<SurrealTokenUsage> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract token_usage: {e}")))?;

        if let Some(record) = existing {
            let key = extract_id(&record.id)?;
            let dto = SurrealTokenUsage {
                id: ("token_usage", key.as_str()).into(),
                client: usage.client.clone(),
                session_id: usage.session_id.clone(),
                message_id: usage.message_id.clone(),
                provider_id: usage.provider_id.clone(),
                model_id: usage.model_id.clone(),
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                reasoning_tokens: usage.reasoning_tokens,
                cache_read_tokens: usage.cache_read_tokens,
                cache_write_tokens: usage.cache_write_tokens,
                pushed: usage.pushed,
                recorded_at: usage.recorded_at,
                created_at: usage.created_at,
            };
            let _: Option<SurrealTokenUsage> = self
                .db
                .upsert(("token_usage", key.as_str()))
                .content(dto)
                .await
                .map_err(|e| OmcError::Storage(format!("Failed to update token_usage: {e}")))?;
        } else {
            let dto = SurrealTokenUsage {
                id: ("token_usage", usage.id.as_str()).into(),
                client: usage.client.clone(),
                session_id: usage.session_id.clone(),
                message_id: usage.message_id.clone(),
                provider_id: usage.provider_id.clone(),
                model_id: usage.model_id.clone(),
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                reasoning_tokens: usage.reasoning_tokens,
                cache_read_tokens: usage.cache_read_tokens,
                cache_write_tokens: usage.cache_write_tokens,
                pushed: usage.pushed,
                recorded_at: usage.recorded_at,
                created_at: usage.created_at,
            };
            let _: Option<SurrealTokenUsage> = self
                .db
                .create(("token_usage", usage.id.as_str()))
                .content(dto)
                .await
                .map_err(|e| OmcError::Storage(format!("Failed to create token_usage: {e}")))?;
        }
        Ok(())
    }

    async fn find_unpushed(&self, limit: usize) -> Result<Vec<TokenUsage>> {
        let query = format!(
            "SELECT * FROM token_usage WHERE pushed = false ORDER BY recorded_at ASC LIMIT {limit};"
        );
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to query unpushed: {e}")))?;
        let rows: Vec<SurrealTokenUsage> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract unpushed: {e}")))?;
        Ok(rows.into_iter().map(surreal_to_token_usage).collect())
    }

    async fn count_unpushed(&self) -> Result<usize> {
        let query = "SELECT count() AS count FROM token_usage WHERE pushed = false GROUP ALL;";
        let mut result = self
            .db
            .query(query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to count unpushed: {e}")))?;
        let rows: Vec<serde_json::Value> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract count: {e}")))?;
        let count = rows
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as usize;
        Ok(count)
    }

    async fn mark_pushed(&self, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let id_list = ids
            .iter()
            .map(|id| format!("'{id}'"))
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!("UPDATE token_usage SET pushed = true WHERE id IN [{id_list}];");
        let _result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to mark pushed: {e}")))?;
        Ok(())
    }

    async fn list_recent(&self, limit: usize, offset: usize) -> Result<Vec<TokenUsage>> {
        let query = format!(
            "SELECT * FROM token_usage ORDER BY recorded_at DESC LIMIT {limit} START {offset};"
        );
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to list recent: {e}")))?;
        let rows: Vec<SurrealTokenUsage> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract recent: {e}")))?;
        Ok(rows.into_iter().map(surreal_to_token_usage).collect())
    }

    async fn cleanup_old_pushed(&self, retention_days: i64) -> Result<usize> {
        let cutoff = chrono::Utc::now().timestamp_millis() - (retention_days * 86_400_000);
        let query = format!(
            "DELETE FROM token_usage WHERE pushed = true AND recorded_at < {cutoff} RETURN BEFORE;"
        );
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to cleanup: {e}")))?;
        let rows: Vec<serde_json::Value> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract cleanup result: {e}")))?;
        Ok(rows.len())
    }

    async fn summary(&self, days: Option<i64>) -> Result<Vec<UsageSummary>> {
        let where_clause = if let Some(d) = days {
            let cutoff = chrono::Utc::now().timestamp_millis() - (d * 86_400_000);
            format!("WHERE recorded_at >= {cutoff}")
        } else {
            String::new()
        };
        let query = format!(
            "SELECT provider_id, model_id, \
             math::sum(input_tokens) AS total_input, \
             math::sum(output_tokens) AS total_output, \
             math::sum(reasoning_tokens) AS total_reasoning, \
             math::sum(cache_read_tokens) AS total_cache_read, \
             math::sum(cache_write_tokens) AS total_cache_write, \
             count() AS request_count \
             FROM token_usage {where_clause} \
             GROUP BY provider_id, model_id;"
        );
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to query summary: {e}")))?;
        let rows: Vec<serde_json::Value> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract summary: {e}")))?;
        Ok(rows
            .into_iter()
            .map(|v| UsageSummary {
                provider_id: v["provider_id"].as_str().unwrap_or("").to_string(),
                model_id: v["model_id"].as_str().unwrap_or("").to_string(),
                total_input: v["total_input"].as_i64().unwrap_or(0),
                total_output: v["total_output"].as_i64().unwrap_or(0),
                total_reasoning: v["total_reasoning"].as_i64().unwrap_or(0),
                total_cache_read: v["total_cache_read"].as_i64().unwrap_or(0),
                total_cache_write: v["total_cache_write"].as_i64().unwrap_or(0),
                request_count: v["request_count"].as_i64().unwrap_or(0),
            })
            .collect())
    }
}

fn surreal_to_token_usage(s: SurrealTokenUsage) -> TokenUsage {
    TokenUsage {
        id: s.id.key().to_string(),
        client: s.client,
        session_id: s.session_id,
        message_id: s.message_id,
        provider_id: s.provider_id,
        model_id: s.model_id,
        input_tokens: s.input_tokens,
        output_tokens: s.output_tokens,
        reasoning_tokens: s.reasoning_tokens,
        cache_read_tokens: s.cache_read_tokens,
        cache_write_tokens: s.cache_write_tokens,
        pushed: s.pushed,
        recorded_at: s.recorded_at,
        created_at: s.created_at,
    }
}
