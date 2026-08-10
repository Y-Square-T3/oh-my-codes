use async_trait::async_trait;
use omc_core::account::{Account, Workspace};
use omc_core::error::Result;
use omc_core::model::Provider;
use omc_core::token_usage::{TokenCost, TokenUsage, TokenUsageOverview, UsageSummary};
use omc_core::types::{Channel, Message};

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn get_account(&self, id: &str) -> Result<Option<Account>>;
    async fn list_accounts(&self) -> Result<Vec<Account>>;
    async fn upsert_account(&self, account: &Account) -> Result<()>;
    async fn delete_account(&self, id: &str) -> Result<()>;
    async fn get_active_account_id(&self) -> Result<Option<String>>;
    async fn set_active_account(&self, id: &str) -> Result<()>;
    async fn clear_active_account(&self) -> Result<()>;
    async fn set_active_workspace(&self, account_id: &str, workspace_id: &str) -> Result<()>;

    async fn list_workspaces(&self, account_id: &str) -> Result<Vec<Workspace>>;
    async fn upsert_workspaces(&self, workspaces: &[Workspace]) -> Result<()>;
    async fn clear_workspaces(&self, account_id: &str) -> Result<()>;

    async fn create_channel(&self, name: &str) -> Result<Channel>;
    async fn list_channels(&self) -> Result<Vec<Channel>>;
    async fn send_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<Message>;
    async fn get_messages(
        &self,
        channel_id: &str,
        limit: usize,
        before: Option<String>,
    ) -> Result<Vec<Message>>;

    async fn list_providers(&self, account_id: &str) -> Result<Vec<Provider>>;
    async fn replace_providers(&self, account_id: &str, providers: Vec<Provider>) -> Result<()>;
    async fn delete_providers(&self, account_id: &str) -> Result<()>;

    async fn upsert_usage(&self, usage: &TokenUsage) -> Result<()>;
    async fn find_unpushed(&self, limit: usize) -> Result<Vec<TokenUsage>>;
    async fn count_unpushed(&self) -> Result<usize>;
    async fn mark_pushed(&self, ids: &[String]) -> Result<()>;
    async fn list_recent(
        &self,
        limit: usize,
        offset: usize,
        pushed: Option<bool>,
    ) -> Result<Vec<TokenUsage>>;
    async fn count_all(&self, pushed: Option<bool>) -> Result<usize>;
    async fn cleanup_old_pushed(&self, retention_days: i64) -> Result<usize>;
    async fn usage_summary(&self, days: Option<i64>) -> Result<Vec<UsageSummary>>;
    async fn usage_overview(&self, days: Option<i64>) -> Result<TokenUsageOverview>;

    async fn upsert_token_cost(&self, cost: &TokenCost) -> Result<()>;
    async fn get_token_cost(&self, usage_id: &str) -> Result<Option<TokenCost>>;
}
