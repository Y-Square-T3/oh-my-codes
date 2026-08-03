use async_trait::async_trait;
use omc_core::error::Result;
use omc_core::token_usage::{TokenUsage, TokenUsageOverview, UsageSummary};

#[async_trait]
pub trait TokenUsageStore: Send + Sync {
    async fn upsert(&self, usage: &TokenUsage) -> Result<()>;
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
    async fn summary(&self, days: Option<i64>) -> Result<Vec<UsageSummary>>;
    async fn overview(&self, days: Option<i64>) -> Result<TokenUsageOverview>;
}
