mod account;
mod message;
mod model;
mod token_usage;
mod workspace;

use crate::migrations::{MigrationRunner, registry};
use crate::traits::StorageBackend;
use async_trait::async_trait;
use omc_core::account::{Account, Workspace};
use omc_core::error::{OmcError, Result};
use omc_core::model::Provider;
use omc_core::token_usage::{TokenUsage, TokenUsageOverview, UsageSummary};
use omc_core::types::{Channel, Message};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::Path;

#[derive(Clone)]
pub struct SqliteBackend {
    pool: SqlitePool,
}

impl SqliteBackend {
    pub async fn new(path: &Path) -> Result<Self> {
        let url = format!("sqlite:{}?mode=rwc", path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(map_err)?;
        MigrationRunner::run_sqlite(&pool, &registry::sqlite_migrations())
            .await
            .map_err(map_migration_err)?;
        Ok(Self { pool })
    }

    pub async fn new_memory() -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .map_err(map_err)?;
        MigrationRunner::run_sqlite(&pool, &registry::sqlite_migrations())
            .await
            .map_err(map_migration_err)?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("SQLite error: {e}"))
}

fn map_migration_err(e: crate::migrations::MigrationError) -> OmcError {
    OmcError::Storage(format!("Migration error: {e}"))
}

#[async_trait]
impl StorageBackend for SqliteBackend {
    async fn get_account(&self, id: &str) -> Result<Option<Account>> {
        account::get_account(&self.pool, id).await
    }

    async fn list_accounts(&self) -> Result<Vec<Account>> {
        account::list_accounts(&self.pool).await
    }

    async fn upsert_account(&self, account: &Account) -> Result<()> {
        account::upsert_account(&self.pool, account).await
    }

    async fn delete_account(&self, id: &str) -> Result<()> {
        account::delete_account(&self.pool, id).await
    }

    async fn get_active_account_id(&self) -> Result<Option<String>> {
        account::get_active_account_id(&self.pool).await
    }

    async fn set_active_account(&self, id: &str) -> Result<()> {
        account::set_active_account(&self.pool, id).await
    }

    async fn clear_active_account(&self) -> Result<()> {
        account::clear_active_account(&self.pool).await
    }

    async fn set_active_workspace(&self, account_id: &str, workspace_id: &str) -> Result<()> {
        account::set_active_workspace(&self.pool, account_id, workspace_id).await
    }

    async fn list_workspaces(&self, account_id: &str) -> Result<Vec<Workspace>> {
        workspace::list_workspaces(&self.pool, account_id).await
    }

    async fn upsert_workspaces(&self, workspaces: &[Workspace]) -> Result<()> {
        workspace::upsert_workspaces(&self.pool, workspaces).await
    }

    async fn clear_workspaces(&self, account_id: &str) -> Result<()> {
        workspace::clear_workspaces(&self.pool, account_id).await
    }

    async fn create_channel(&self, name: &str) -> Result<Channel> {
        message::create_channel(&self.pool, name).await
    }

    async fn list_channels(&self) -> Result<Vec<Channel>> {
        message::list_channels(&self.pool).await
    }

    async fn send_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<Message> {
        message::send_message(&self.pool, channel_id, author_id, content).await
    }

    async fn get_messages(
        &self,
        channel_id: &str,
        limit: usize,
        before: Option<String>,
    ) -> Result<Vec<Message>> {
        message::get_messages(&self.pool, channel_id, limit, before).await
    }

    async fn list_providers(&self, account_id: &str) -> Result<Vec<Provider>> {
        model::list_providers(&self.pool, account_id).await
    }

    async fn replace_providers(&self, account_id: &str, providers: Vec<Provider>) -> Result<()> {
        model::replace_providers(&self.pool, account_id, providers).await
    }

    async fn delete_providers(&self, account_id: &str) -> Result<()> {
        model::delete_providers(&self.pool, account_id).await
    }

    async fn upsert_usage(&self, usage: &TokenUsage) -> Result<()> {
        token_usage::upsert_usage(&self.pool, usage).await
    }

    async fn find_unpushed(&self, limit: usize) -> Result<Vec<TokenUsage>> {
        token_usage::find_unpushed(&self.pool, limit).await
    }

    async fn count_unpushed(&self) -> Result<usize> {
        token_usage::count_unpushed(&self.pool).await
    }

    async fn mark_pushed(&self, ids: &[String]) -> Result<()> {
        token_usage::mark_pushed(&self.pool, ids).await
    }

    async fn list_recent(
        &self,
        limit: usize,
        offset: usize,
        pushed: Option<bool>,
    ) -> Result<Vec<TokenUsage>> {
        token_usage::list_recent(&self.pool, limit, offset, pushed).await
    }

    async fn count_all(&self, pushed: Option<bool>) -> Result<usize> {
        token_usage::count_all(&self.pool, pushed).await
    }

    async fn cleanup_old_pushed(&self, retention_days: i64) -> Result<usize> {
        token_usage::cleanup_old_pushed(&self.pool, retention_days).await
    }

    async fn usage_summary(&self, days: Option<i64>) -> Result<Vec<UsageSummary>> {
        token_usage::usage_summary(&self.pool, days).await
    }

    async fn usage_overview(&self, days: Option<i64>) -> Result<TokenUsageOverview> {
        token_usage::usage_overview(&self.pool, days).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_migration_err_wraps_migration_error() {
        let migration_err = crate::migrations::MigrationError::MissingDownMigration {
            version: 1,
            name: "test_migration".to_string(),
        };
        let omc_err = map_migration_err(migration_err);

        match omc_err {
            OmcError::Storage(msg) => {
                assert!(msg.contains("Migration error"));
                assert!(msg.contains("test_migration"));
            }
            _ => panic!("Expected OmcError::Storage"),
        }
    }
}
