use async_trait::async_trait;
use omc_core::error::Result;
use omc_core::model::Provider;

#[async_trait]
pub trait ModelStore: Send + Sync {
    async fn list_providers(&self, account_id: &str) -> Result<Vec<Provider>>;
    async fn replace_providers(&self, account_id: &str, providers: Vec<Provider>) -> Result<()>;
    async fn delete_providers(&self, account_id: &str) -> Result<()>;
}
