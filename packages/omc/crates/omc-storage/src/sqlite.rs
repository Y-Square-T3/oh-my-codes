use crate::account_store::AccountStore;
use crate::message_store::MessageStore;
use crate::model_store::ModelStore;
use crate::token_usage_store::TokenUsageStore;
use crate::workspace_store::WorkspaceStore;
use async_trait::async_trait;
use omc_core::account::{Account, Workspace};
use omc_core::error::{OmcError, Result};
use omc_core::model::Provider;
use omc_core::token_usage::{
    DailyUsage, HeadlineStats, TokenUsage, TokenUsageOverview, UsageGroup, UsageSummary,
};
use omc_core::types::{Channel, Message};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{FromRow, Row};
use std::path::Path;
use std::sync::Arc;

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("SQLite error: {e}"))
}

pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new(path: &Path) -> Result<Self> {
        let url = format!("sqlite:{}?mode=rwc", path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(map_err)?;
        Self::init_schema(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn new_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .map_err(map_err)?;
        Self::init_schema(&pool).await?;
        Ok(Self { pool })
    }

    async fn init_schema(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS channel (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                topic TEXT,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(pool)
        .await
        .map_err(map_err)?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS message (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL,
                author_id TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                edited_at INTEGER,
                reply_to TEXT
            )",
        )
        .execute(pool)
        .await
        .map_err(map_err)?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_message_channel_ts ON message(channel_id, timestamp)",
        )
        .execute(pool)
        .await
        .map_err(map_err)?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS account (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL,
                url TEXT NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT NOT NULL,
                token_expiry INTEGER NOT NULL,
                active_workspace_id TEXT
            )",
        )
        .execute(pool)
        .await
        .map_err(map_err)?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS workspace (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                name TEXT NOT NULL,
                is_admin INTEGER NOT NULL,
                FOREIGN KEY (account_id) REFERENCES account(id)
            )",
        )
        .execute(pool)
        .await
        .map_err(map_err)?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workspace_account ON workspace(account_id)")
            .execute(pool)
            .await
            .map_err(map_err)?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS active_account (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                account_id TEXT,
                FOREIGN KEY (account_id) REFERENCES account(id)
            )",
        )
        .execute(pool)
        .await
        .map_err(map_err)?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS provider (
                id TEXT PRIMARY KEY,
                provider_id TEXT NOT NULL,
                name TEXT NOT NULL,
                env TEXT NOT NULL,
                api TEXT,
                npm TEXT,
                doc TEXT,
                models TEXT NOT NULL,
                account_id TEXT NOT NULL,
                last_fetched_at INTEGER NOT NULL,
                FOREIGN KEY (account_id) REFERENCES account(id)
            )",
        )
        .execute(pool)
        .await
        .map_err(map_err)?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_provider_account ON provider(account_id)")
            .execute(pool)
            .await
            .map_err(map_err)?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS token_usage (
                id TEXT PRIMARY KEY,
                client TEXT NOT NULL,
                session_id TEXT NOT NULL,
                message_id TEXT NOT NULL UNIQUE,
                agent TEXT,
                provider_id TEXT NOT NULL,
                model_id TEXT NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                reasoning_tokens INTEGER NOT NULL,
                cache_read_tokens INTEGER NOT NULL,
                cache_write_tokens INTEGER NOT NULL,
                pushed INTEGER NOT NULL DEFAULT 0,
                recorded_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(pool)
        .await
        .map_err(map_err)?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_token_usage_pushed ON token_usage(pushed, recorded_at)",
        )
        .execute(pool)
        .await
        .map_err(map_err)?;

        Ok(())
    }

    pub fn pool(&self) -> Arc<SqlitePool> {
        Arc::new(self.pool.clone())
    }
}

#[derive(Debug, Clone, FromRow)]
struct ChannelRow {
    id: String,
    name: String,
    topic: Option<String>,
    created_at: i64,
}

#[async_trait]
impl MessageStore for SqliteStorage {
    async fn create_channel(&self, name: &str) -> Result<Channel> {
        let now = chrono::Utc::now().timestamp();
        let id = ulid::Ulid::new().to_string();
        sqlx::query("INSERT INTO channel (id, name, topic, created_at) VALUES (?, ?, NULL, ?)")
            .bind(&id)
            .bind(name)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;
        Ok(Channel {
            id,
            name: name.to_string(),
            topic: None,
            created_at: now,
        })
    }

    async fn list_channels(&self) -> Result<Vec<Channel>> {
        let rows: Vec<ChannelRow> =
            sqlx::query_as("SELECT id, name, topic, created_at FROM channel ORDER BY created_at")
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?;
        Ok(rows
            .into_iter()
            .map(|r| Channel {
                id: r.id,
                name: r.name,
                topic: r.topic,
                created_at: r.created_at,
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
        let id = ulid::Ulid::new().to_string();
        sqlx::query(
            "INSERT INTO message (id, channel_id, author_id, content, timestamp, edited_at, reply_to) VALUES (?, ?, ?, ?, ?, NULL, NULL)",
        )
        .bind(&id)
        .bind(channel_id)
        .bind(author_id)
        .bind(content)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
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
        let rows: Vec<MessageRow> = if let Some(before_id) = before {
            sqlx::query_as(
                "SELECT id, channel_id, author_id, content, timestamp, edited_at, reply_to
                 FROM message
                 WHERE channel_id = ? AND timestamp < (SELECT timestamp FROM message WHERE id = ?)
                 ORDER BY timestamp DESC
                 LIMIT ?",
            )
            .bind(channel_id)
            .bind(&before_id)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(map_err)?
        } else {
            sqlx::query_as(
                "SELECT id, channel_id, author_id, content, timestamp, edited_at, reply_to
                 FROM message
                 WHERE channel_id = ?
                 ORDER BY timestamp DESC
                 LIMIT ?",
            )
            .bind(channel_id)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(map_err)?
        };
        Ok(rows
            .into_iter()
            .map(|r| Message {
                id: r.id,
                channel_id: r.channel_id,
                author_id: r.author_id,
                content: r.content,
                timestamp: r.timestamp,
                edited_at: r.edited_at,
                reply_to: r.reply_to,
            })
            .collect())
    }
}

#[derive(Debug, Clone, FromRow)]
struct MessageRow {
    id: String,
    channel_id: String,
    author_id: String,
    content: String,
    timestamp: i64,
    edited_at: Option<i64>,
    reply_to: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
struct AccountRow {
    id: String,
    email: String,
    url: String,
    access_token: String,
    refresh_token: String,
    token_expiry: i64,
    active_workspace_id: Option<String>,
}

pub struct SqliteAccountStore {
    pool: SqlitePool,
}

impl SqliteAccountStore {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self {
            pool: (*pool).clone(),
        }
    }
}

#[async_trait]
impl AccountStore for SqliteAccountStore {
    async fn get_account(&self, id: &str) -> Result<Option<Account>> {
        let row: Option<AccountRow> =
            sqlx::query_as("SELECT id, email, url, access_token, refresh_token, token_expiry, active_workspace_id FROM account WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_err)?;
        Ok(row.map(|r| Account {
            id: r.id,
            email: r.email,
            url: r.url,
            access_token: r.access_token,
            refresh_token: r.refresh_token,
            token_expiry: r.token_expiry,
            active_workspace_id: r.active_workspace_id,
        }))
    }

    async fn list_accounts(&self) -> Result<Vec<Account>> {
        let rows: Vec<AccountRow> = sqlx::query_as(
            "SELECT id, email, url, access_token, refresh_token, token_expiry, active_workspace_id FROM account",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(rows
            .into_iter()
            .map(|r| Account {
                id: r.id,
                email: r.email,
                url: r.url,
                access_token: r.access_token,
                refresh_token: r.refresh_token,
                token_expiry: r.token_expiry,
                active_workspace_id: r.active_workspace_id,
            })
            .collect())
    }

    async fn upsert_account(&self, account: &Account) -> Result<()> {
        sqlx::query(
            "INSERT INTO account (id, email, url, access_token, refresh_token, token_expiry, active_workspace_id)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                email = excluded.email,
                url = excluded.url,
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                token_expiry = excluded.token_expiry,
                active_workspace_id = excluded.active_workspace_id",
        )
        .bind(&account.id)
        .bind(&account.email)
        .bind(&account.url)
        .bind(&account.access_token)
        .bind(&account.refresh_token)
        .bind(account.token_expiry)
        .bind(&account.active_workspace_id)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn delete_account(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM account WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn get_active_account_id(&self) -> Result<Option<String>> {
        let row: Option<SqliteRow> =
            sqlx::query("SELECT account_id FROM active_account WHERE id = 1")
                .fetch_optional(&self.pool)
                .await
                .map_err(map_err)?;
        Ok(row.and_then(|r| r.get::<Option<String>, _>("account_id")))
    }

    async fn set_active_account(&self, id: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO active_account (id, account_id) VALUES (1, ?)
             ON CONFLICT(id) DO UPDATE SET account_id = excluded.account_id",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn clear_active_account(&self) -> Result<()> {
        sqlx::query("DELETE FROM active_account")
            .execute(&self.pool)
            .await
            .map_err(map_err)?;
        Ok(())
    }

    async fn set_active_workspace(&self, account_id: &str, workspace_id: &str) -> Result<()> {
        let result = sqlx::query("UPDATE account SET active_workspace_id = ? WHERE id = ?")
            .bind(workspace_id)
            .bind(account_id)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;
        if result.rows_affected() == 0 {
            return Err(OmcError::Storage(format!(
                "Account '{account_id}' not found"
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, FromRow)]
struct WorkspaceRow {
    id: String,
    account_id: String,
    name: String,
    is_admin: i32,
}

pub struct SqliteWorkspaceStore {
    pool: SqlitePool,
}

impl SqliteWorkspaceStore {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self {
            pool: (*pool).clone(),
        }
    }
}

#[async_trait]
impl WorkspaceStore for SqliteWorkspaceStore {
    async fn list_workspaces(&self, account_id: &str) -> Result<Vec<Workspace>> {
        let rows: Vec<WorkspaceRow> = sqlx::query_as(
            "SELECT id, account_id, name, is_admin FROM workspace WHERE account_id = ?",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(rows
            .into_iter()
            .map(|r| Workspace {
                id: r.id,
                account_id: r.account_id,
                name: r.name,
                is_admin: r.is_admin != 0,
            })
            .collect())
    }

    async fn upsert_workspaces(&self, workspaces: &[Workspace]) -> Result<()> {
        for w in workspaces {
            let is_admin = if w.is_admin { 1 } else { 0 };
            sqlx::query(
                "INSERT INTO workspace (id, account_id, name, is_admin)
                 VALUES (?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                    account_id = excluded.account_id,
                    name = excluded.name,
                    is_admin = excluded.is_admin",
            )
            .bind(&w.id)
            .bind(&w.account_id)
            .bind(&w.name)
            .bind(is_admin)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;
        }
        Ok(())
    }

    async fn clear_workspaces(&self, account_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM workspace WHERE account_id = ?")
            .bind(account_id)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;
        Ok(())
    }
}

#[derive(Debug, Clone, FromRow)]
struct ProviderRow {
    provider_id: String,
    name: String,
    env: String,
    api: Option<String>,
    npm: Option<String>,
    doc: Option<String>,
    models: String,
    account_id: String,
    last_fetched_at: i64,
}

pub struct SqliteModelStore {
    pool: SqlitePool,
}

impl SqliteModelStore {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self {
            pool: (*pool).clone(),
        }
    }
}

#[async_trait]
impl ModelStore for SqliteModelStore {
    async fn list_providers(&self, account_id: &str) -> Result<Vec<Provider>> {
        let rows: Vec<ProviderRow> =
            sqlx::query_as("SELECT provider_id, name, env, api, npm, doc, models, account_id, last_fetched_at FROM provider WHERE account_id = ?")
                .bind(account_id)
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?;
        rows.into_iter()
            .map(|r| {
                let env: Vec<String> = serde_json::from_str(&r.env)
                    .map_err(|e| OmcError::Storage(format!("Failed to deserialize env: {e}")))?;
                let models: Vec<omc_core::model::Model> = serde_json::from_str(&r.models)
                    .map_err(|e| OmcError::Storage(format!("Failed to deserialize models: {e}")))?;
                Ok(Provider {
                    id: r.provider_id,
                    name: r.name,
                    env,
                    api: r.api,
                    npm: r.npm,
                    doc: r.doc,
                    models,
                    account_id: r.account_id,
                    last_fetched_at: r.last_fetched_at,
                })
            })
            .collect()
    }

    async fn replace_providers(&self, account_id: &str, providers: Vec<Provider>) -> Result<()> {
        self.delete_providers(account_id).await?;
        for p in providers {
            let id = format!("{}:{}", p.id, account_id);
            let env_json = serde_json::to_string(&p.env)
                .map_err(|e| OmcError::Storage(format!("Failed to serialize env: {e}")))?;
            let models_json = serde_json::to_string(&p.models)
                .map_err(|e| OmcError::Storage(format!("Failed to serialize models: {e}")))?;
            sqlx::query(
                "INSERT INTO provider (id, provider_id, name, env, api, npm, doc, models, account_id, last_fetched_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(&p.id)
            .bind(&p.name)
            .bind(&env_json)
            .bind(&p.api)
            .bind(&p.npm)
            .bind(&p.doc)
            .bind(&models_json)
            .bind(&p.account_id)
            .bind(p.last_fetched_at)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;
        }
        Ok(())
    }

    async fn delete_providers(&self, account_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM provider WHERE account_id = ?")
            .bind(account_id)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;
        Ok(())
    }
}

#[derive(Debug, Clone, FromRow)]
struct TokenUsageRow {
    id: String,
    client: String,
    session_id: String,
    message_id: String,
    agent: Option<String>,
    provider_id: String,
    model_id: String,
    input_tokens: i64,
    output_tokens: i64,
    reasoning_tokens: i64,
    cache_read_tokens: i64,
    cache_write_tokens: i64,
    pushed: i32,
    recorded_at: i64,
    created_at: i64,
}

fn row_to_token_usage(r: TokenUsageRow) -> TokenUsage {
    TokenUsage {
        id: r.id,
        client: r.client,
        session_id: r.session_id,
        message_id: r.message_id,
        agent: r.agent,
        provider_id: r.provider_id,
        model_id: r.model_id,
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        reasoning_tokens: r.reasoning_tokens,
        cache_read_tokens: r.cache_read_tokens,
        cache_write_tokens: r.cache_write_tokens,
        pushed: r.pushed != 0,
        recorded_at: r.recorded_at,
        created_at: r.created_at,
    }
}

pub struct SqliteTokenUsageStore {
    pool: SqlitePool,
}

impl SqliteTokenUsageStore {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self {
            pool: (*pool).clone(),
        }
    }
}

#[async_trait]
impl TokenUsageStore for SqliteTokenUsageStore {
    async fn upsert(&self, usage: &TokenUsage) -> Result<()> {
        let pushed = if usage.pushed { 1 } else { 0 };
        sqlx::query(
            "INSERT INTO token_usage (id, client, session_id, message_id, agent, provider_id, model_id, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, pushed, recorded_at, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(message_id) DO UPDATE SET
                client = excluded.client,
                session_id = excluded.session_id,
                agent = excluded.agent,
                provider_id = excluded.provider_id,
                model_id = excluded.model_id,
                input_tokens = excluded.input_tokens,
                output_tokens = excluded.output_tokens,
                reasoning_tokens = excluded.reasoning_tokens,
                cache_read_tokens = excluded.cache_read_tokens,
                cache_write_tokens = excluded.cache_write_tokens,
                pushed = excluded.pushed,
                recorded_at = excluded.recorded_at,
                created_at = excluded.created_at",
        )
        .bind(&usage.id)
        .bind(&usage.client)
        .bind(&usage.session_id)
        .bind(&usage.message_id)
        .bind(&usage.agent)
        .bind(&usage.provider_id)
        .bind(&usage.model_id)
        .bind(usage.input_tokens)
        .bind(usage.output_tokens)
        .bind(usage.reasoning_tokens)
        .bind(usage.cache_read_tokens)
        .bind(usage.cache_write_tokens)
        .bind(pushed)
        .bind(usage.recorded_at)
        .bind(usage.created_at)
        .execute(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(())
    }

    async fn find_unpushed(&self, limit: usize) -> Result<Vec<TokenUsage>> {
        let rows: Vec<TokenUsageRow> = sqlx::query_as(
            "SELECT id, client, session_id, message_id, agent, provider_id, model_id, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, pushed, recorded_at, created_at
             FROM token_usage WHERE pushed = 0 ORDER BY recorded_at ASC LIMIT ?",
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(rows.into_iter().map(row_to_token_usage).collect())
    }

    async fn count_unpushed(&self) -> Result<usize> {
        let row: SqliteRow =
            sqlx::query("SELECT COUNT(*) as count FROM token_usage WHERE pushed = 0")
                .fetch_one(&self.pool)
                .await
                .map_err(map_err)?;
        let count: i64 = row.get("count");
        Ok(count as usize)
    }

    async fn mark_pushed(&self, ids: &[String]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let query = format!(
            "UPDATE token_usage SET pushed = 1 WHERE id IN ({})",
            placeholders.join(",")
        );
        let mut q = sqlx::query(&query);
        for id in ids {
            q = q.bind(id);
        }
        q.execute(&self.pool).await.map_err(map_err)?;
        Ok(())
    }

    async fn list_recent(
        &self,
        limit: usize,
        offset: usize,
        pushed: Option<bool>,
    ) -> Result<Vec<TokenUsage>> {
        let where_clause = match pushed {
            Some(true) => " WHERE pushed = 1",
            Some(false) => " WHERE pushed = 0",
            None => "",
        };
        let sql = format!(
            "SELECT id, client, session_id, message_id, agent, provider_id, model_id, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, pushed, recorded_at, created_at
             FROM token_usage{where_clause} ORDER BY recorded_at DESC LIMIT ? OFFSET ?",
        );
        let rows: Vec<TokenUsageRow> = sqlx::query_as(&sql)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(map_err)?;
        Ok(rows.into_iter().map(row_to_token_usage).collect())
    }

    async fn count_all(&self, pushed: Option<bool>) -> Result<usize> {
        let sql = match pushed {
            Some(true) => "SELECT COUNT(*) as count FROM token_usage WHERE pushed = 1",
            Some(false) => "SELECT COUNT(*) as count FROM token_usage WHERE pushed = 0",
            None => "SELECT COUNT(*) as count FROM token_usage",
        };
        let row: SqliteRow = sqlx::query(sql)
            .fetch_one(&self.pool)
            .await
            .map_err(map_err)?;
        let count: i64 = row.get("count");
        Ok(count as usize)
    }

    async fn cleanup_old_pushed(&self, retention_days: i64) -> Result<usize> {
        let cutoff = chrono::Utc::now().timestamp_millis() - (retention_days * 86_400_000);
        let count_row: SqliteRow = sqlx::query(
            "SELECT COUNT(*) as count FROM token_usage WHERE pushed = 1 AND recorded_at < ?",
        )
        .bind(cutoff)
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;
        let count: i64 = count_row.get("count");

        sqlx::query("DELETE FROM token_usage WHERE pushed = 1 AND recorded_at < ?")
            .bind(cutoff)
            .execute(&self.pool)
            .await
            .map_err(map_err)?;

        Ok(count as usize)
    }

    async fn summary(&self, days: Option<i64>) -> Result<Vec<UsageSummary>> {
        let cutoff = days.map(|d| chrono::Utc::now().timestamp_millis() - (d * 86_400_000));
        let query = if cutoff.is_some() {
            "SELECT provider_id, model_id,
                SUM(input_tokens) as total_input,
                SUM(output_tokens) as total_output,
                SUM(reasoning_tokens) as total_reasoning,
                SUM(cache_read_tokens) as total_cache_read,
                SUM(cache_write_tokens) as total_cache_write,
                COUNT(*) as request_count
             FROM token_usage WHERE recorded_at >= ?
             GROUP BY provider_id, model_id"
                .to_string()
        } else {
            "SELECT provider_id, model_id,
                SUM(input_tokens) as total_input,
                SUM(output_tokens) as total_output,
                SUM(reasoning_tokens) as total_reasoning,
                SUM(cache_read_tokens) as total_cache_read,
                SUM(cache_write_tokens) as total_cache_write,
                COUNT(*) as request_count
             FROM token_usage
             GROUP BY provider_id, model_id"
                .to_string()
        };

        let rows: Vec<SqliteRow> = if let Some(c) = cutoff {
            sqlx::query(&query)
                .bind(c)
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?
        } else {
            sqlx::query(&query)
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?
        };

        Ok(rows
            .into_iter()
            .map(|r| UsageSummary {
                provider_id: r.get("provider_id"),
                model_id: r.get("model_id"),
                total_input: r.get("total_input"),
                total_output: r.get("total_output"),
                total_reasoning: r.get("total_reasoning"),
                total_cache_read: r.get("total_cache_read"),
                total_cache_write: r.get("total_cache_write"),
                request_count: r.get("request_count"),
            })
            .collect())
    }

    async fn overview(&self, days: Option<i64>) -> Result<TokenUsageOverview> {
        let cutoff = days.map(|d| chrono::Utc::now().timestamp_millis() - (d * 86_400_000));
        let seven_days_ago = chrono::Utc::now().timestamp_millis() - (7 * 86_400_000);

        let headline = self.headline_stats(cutoff).await?;
        let top_models = self.top_models(cutoff).await?;
        let top_agents = self.top_agents(cutoff).await?;
        let top_clients = self.top_clients(cutoff).await?;
        let trend = self.daily_trend(seven_days_ago).await?;

        Ok(TokenUsageOverview {
            headline,
            top_models,
            top_agents,
            top_clients,
            trend,
        })
    }
}

impl SqliteTokenUsageStore {
    async fn headline_stats(&self, cutoff: Option<i64>) -> Result<HeadlineStats> {
        let (query, bind_cutoff) = if cutoff.is_some() {
            (
                "SELECT
                    COUNT(*) as requests,
                    SUM(input_tokens) as input_tokens,
                    SUM(output_tokens) as output_tokens,
                    SUM(reasoning_tokens) as reasoning_tokens,
                    SUM(cache_read_tokens) as cache_read_tokens,
                    SUM(cache_write_tokens) as cache_write_tokens
                 FROM token_usage WHERE recorded_at >= ?",
                true,
            )
        } else {
            (
                "SELECT
                    COUNT(*) as requests,
                    SUM(input_tokens) as input_tokens,
                    SUM(output_tokens) as output_tokens,
                    SUM(reasoning_tokens) as reasoning_tokens,
                    SUM(cache_read_tokens) as cache_read_tokens,
                    SUM(cache_write_tokens) as cache_write_tokens
                 FROM token_usage",
                false,
            )
        };

        let row: SqliteRow = if bind_cutoff {
            sqlx::query(query)
                .bind(cutoff.unwrap())
                .fetch_one(&self.pool)
                .await
                .map_err(map_err)?
        } else {
            sqlx::query(query)
                .fetch_one(&self.pool)
                .await
                .map_err(map_err)?
        };

        let unpushed_row: SqliteRow = sqlx::query(
            "SELECT COUNT(*) as count, SUM(input_tokens + output_tokens + reasoning_tokens) as tokens FROM token_usage WHERE pushed = 0",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(HeadlineStats {
            requests: row.get("requests"),
            input_tokens: row.get("input_tokens"),
            output_tokens: row.get("output_tokens"),
            reasoning_tokens: row.get("reasoning_tokens"),
            cache_read_tokens: row.get("cache_read_tokens"),
            cache_write_tokens: row.get("cache_write_tokens"),
            unpushed_records: unpushed_row.get::<i64, _>("count") as usize,
            unpushed_tokens: unpushed_row.get::<Option<i64>, _>("tokens").unwrap_or(0),
        })
    }

    async fn top_models(&self, cutoff: Option<i64>) -> Result<Vec<UsageSummary>> {
        let query = if cutoff.is_some() {
            "SELECT provider_id, model_id,
                SUM(input_tokens) as total_input,
                SUM(output_tokens) as total_output,
                SUM(reasoning_tokens) as total_reasoning,
                SUM(cache_read_tokens) as total_cache_read,
                SUM(cache_write_tokens) as total_cache_write,
                COUNT(*) as request_count
             FROM token_usage WHERE recorded_at >= ?
             GROUP BY provider_id, model_id
             ORDER BY (SUM(input_tokens) + SUM(output_tokens) + SUM(reasoning_tokens)) DESC
             LIMIT 3"
        } else {
            "SELECT provider_id, model_id,
                SUM(input_tokens) as total_input,
                SUM(output_tokens) as total_output,
                SUM(reasoning_tokens) as total_reasoning,
                SUM(cache_read_tokens) as total_cache_read,
                SUM(cache_write_tokens) as total_cache_write,
                COUNT(*) as request_count
             FROM token_usage
             GROUP BY provider_id, model_id
             ORDER BY (SUM(input_tokens) + SUM(output_tokens) + SUM(reasoning_tokens)) DESC
             LIMIT 3"
        };

        let rows: Vec<SqliteRow> = if let Some(c) = cutoff {
            sqlx::query(query)
                .bind(c)
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?
        } else {
            sqlx::query(query)
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?
        };

        Ok(rows
            .into_iter()
            .map(|r| UsageSummary {
                provider_id: r.get("provider_id"),
                model_id: r.get("model_id"),
                total_input: r.get("total_input"),
                total_output: r.get("total_output"),
                total_reasoning: r.get("total_reasoning"),
                total_cache_read: r.get("total_cache_read"),
                total_cache_write: r.get("total_cache_write"),
                request_count: r.get("request_count"),
            })
            .collect())
    }

    async fn top_agents(&self, cutoff: Option<i64>) -> Result<Vec<UsageGroup>> {
        let query = if cutoff.is_some() {
            "SELECT COALESCE(agent, 'unknown') as label,
                SUM(input_tokens) as total_input,
                SUM(output_tokens) as total_output,
                SUM(reasoning_tokens) as total_reasoning,
                SUM(cache_read_tokens) as total_cache_read,
                SUM(cache_write_tokens) as total_cache_write,
                COUNT(*) as request_count
             FROM token_usage WHERE recorded_at >= ?
             GROUP BY label
             ORDER BY (SUM(input_tokens) + SUM(output_tokens) + SUM(reasoning_tokens)) DESC
             LIMIT 3"
        } else {
            "SELECT COALESCE(agent, 'unknown') as label,
                SUM(input_tokens) as total_input,
                SUM(output_tokens) as total_output,
                SUM(reasoning_tokens) as total_reasoning,
                SUM(cache_read_tokens) as total_cache_read,
                SUM(cache_write_tokens) as total_cache_write,
                COUNT(*) as request_count
             FROM token_usage
             GROUP BY label
             ORDER BY (SUM(input_tokens) + SUM(output_tokens) + SUM(reasoning_tokens)) DESC
             LIMIT 3"
        };

        let rows: Vec<SqliteRow> = if let Some(c) = cutoff {
            sqlx::query(query)
                .bind(c)
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?
        } else {
            sqlx::query(query)
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?
        };

        Ok(rows
            .into_iter()
            .map(|r| UsageGroup {
                label: r.get("label"),
                total_input: r.get("total_input"),
                total_output: r.get("total_output"),
                total_reasoning: r.get("total_reasoning"),
                total_cache_read: r.get("total_cache_read"),
                total_cache_write: r.get("total_cache_write"),
                request_count: r.get("request_count"),
            })
            .collect())
    }

    async fn top_clients(&self, cutoff: Option<i64>) -> Result<Vec<UsageGroup>> {
        let query = if cutoff.is_some() {
            "SELECT client as label,
                SUM(input_tokens) as total_input,
                SUM(output_tokens) as total_output,
                SUM(reasoning_tokens) as total_reasoning,
                SUM(cache_read_tokens) as total_cache_read,
                SUM(cache_write_tokens) as total_cache_write,
                COUNT(*) as request_count
             FROM token_usage WHERE recorded_at >= ?
             GROUP BY client
             ORDER BY (SUM(input_tokens) + SUM(output_tokens) + SUM(reasoning_tokens)) DESC
             LIMIT 3"
        } else {
            "SELECT client as label,
                SUM(input_tokens) as total_input,
                SUM(output_tokens) as total_output,
                SUM(reasoning_tokens) as total_reasoning,
                SUM(cache_read_tokens) as total_cache_read,
                SUM(cache_write_tokens) as total_cache_write,
                COUNT(*) as request_count
             FROM token_usage
             GROUP BY client
             ORDER BY (SUM(input_tokens) + SUM(output_tokens) + SUM(reasoning_tokens)) DESC
             LIMIT 3"
        };

        let rows: Vec<SqliteRow> = if let Some(c) = cutoff {
            sqlx::query(query)
                .bind(c)
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?
        } else {
            sqlx::query(query)
                .fetch_all(&self.pool)
                .await
                .map_err(map_err)?
        };

        Ok(rows
            .into_iter()
            .map(|r| UsageGroup {
                label: r.get("label"),
                total_input: r.get("total_input"),
                total_output: r.get("total_output"),
                total_reasoning: r.get("total_reasoning"),
                total_cache_read: r.get("total_cache_read"),
                total_cache_write: r.get("total_cache_write"),
                request_count: r.get("request_count"),
            })
            .collect())
    }

    async fn daily_trend(&self, cutoff: i64) -> Result<Vec<DailyUsage>> {
        let rows: Vec<SqliteRow> = sqlx::query(
            "SELECT
                DATE(recorded_at / 1000, 'unixepoch') as date,
                COUNT(*) as requests,
                SUM(input_tokens + output_tokens + reasoning_tokens) as total_tokens
             FROM token_usage
             WHERE recorded_at >= ?
             GROUP BY date
             ORDER BY date",
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;

        Ok(rows
            .into_iter()
            .map(|r| DailyUsage {
                date: r.get("date"),
                requests: r.get("requests"),
                total_tokens: r.get("total_tokens"),
            })
            .collect())
    }
}
