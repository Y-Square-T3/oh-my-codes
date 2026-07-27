use async_trait::async_trait;
use omc_core::account::Account;
use omc_core::error::Result;

#[async_trait]
pub trait AccountStore: Send + Sync {
    async fn get_account(&self, id: &str) -> Result<Option<Account>>;
    async fn list_accounts(&self) -> Result<Vec<Account>>;
    async fn upsert_account(&self, account: &Account) -> Result<()>;
    async fn delete_account(&self, id: &str) -> Result<()>;
    async fn get_active_account_id(&self) -> Result<Option<String>>;
    async fn set_active_account(&self, id: &str) -> Result<()>;
    async fn clear_active_account(&self) -> Result<()>;
    async fn set_active_workspace(&self, account_id: &str, workspace_id: &str) -> Result<()>;
}
