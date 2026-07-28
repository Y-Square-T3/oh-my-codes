pub mod account_store;
pub mod memory;
pub mod message_store;
pub mod model_store;
pub mod surreal;
pub mod token_usage_store;
pub mod wal;
pub mod workspace_store;

use async_trait::async_trait;
use omc_core::error::Result;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn put(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
}
