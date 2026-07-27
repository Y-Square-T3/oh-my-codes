use async_trait::async_trait;
use omc_core::account::Workspace;
use omc_core::error::Result;

#[async_trait]
pub trait WorkspaceStore: Send + Sync {
    async fn list_workspaces(&self, account_id: &str) -> Result<Vec<Workspace>>;
    async fn upsert_workspaces(&self, workspaces: &[Workspace]) -> Result<()>;
    async fn clear_workspaces(&self, account_id: &str) -> Result<()>;
}
