use crate::account_service::AccountService;
use crate::server_client::{OmcServerClient, TokenUsagePayload};
use omc_core::error::{OmcError, Result};
use omc_core::token_usage::{TokenUsage, UsageSummary};
use omc_storage::token_usage_store::TokenUsageStore;
use std::sync::Arc;
use tokio::sync::Notify;

const DEFAULT_BATCH_SIZE: usize = 20;
const DEFAULT_RETENTION_DAYS: i64 = 30;
const MAX_LOOP_ITERATIONS: usize = 100;

pub struct StatusResult {
    pub unpushed_count: usize,
    pub has_active_account: bool,
}

pub struct PushResult {
    pub pushed_count: usize,
    pub failed_count: usize,
    pub total_batches: usize,
}

pub struct TokenUsageService {
    store: Arc<dyn TokenUsageStore>,
    account_service: Arc<AccountService>,
    server_client: OmcServerClient,
}

impl TokenUsageService {
    pub fn new(
        store: Arc<dyn TokenUsageStore>,
        account_service: Arc<AccountService>,
        server_client: OmcServerClient,
    ) -> Self {
        Self {
            store,
            account_service,
            server_client,
        }
    }

    pub async fn record(&self, usage: &TokenUsage) -> Result<()> {
        self.store.upsert(usage).await
    }

    pub async fn status(&self) -> Result<StatusResult> {
        let unpushed_count = self.store.count_unpushed().await?;
        let has_active_account = self.account_service.active().await?.is_some();
        Ok(StatusResult {
            unpushed_count,
            has_active_account,
        })
    }

    pub async fn push_batch(&self, batch_size: Option<usize>) -> Result<PushResult> {
        let batch_size = batch_size.unwrap_or(DEFAULT_BATCH_SIZE);

        let (account, token) =
            self.account_service
                .active_with_token()
                .await?
                .ok_or_else(|| {
                    OmcError::Auth("No active account. Run `omc account login` first.".into())
                })?;

        let workspace_id = account.active_workspace_id.as_deref();

        let mut total_pushed = 0usize;
        let mut total_failed = 0usize;
        let mut total_batches = 0usize;
        let mut iterations = 0usize;

        loop {
            iterations += 1;
            if iterations > MAX_LOOP_ITERATIONS {
                tracing::error!("push_batch exceeded max iterations ({MAX_LOOP_ITERATIONS})");
                break;
            }

            let unpushed = self.store.find_unpushed(batch_size).await?;
            if unpushed.is_empty() {
                break;
            }

            let payload: Vec<TokenUsagePayload> = unpushed
                .iter()
                .map(|u| TokenUsagePayload {
                    model: format!("{}/{}", u.provider_id, u.model_id),
                    prompt_tokens: u.input_tokens,
                    completion_tokens: u.output_tokens,
                    reasoning_tokens: u.reasoning_tokens,
                    cache_read_tokens: u.cache_read_tokens,
                    cache_write_tokens: u.cache_write_tokens,
                    request_id: u.message_id.clone(),
                    session_id: u.session_id.clone(),
                    agent: u.agent.clone(),
                    created_at: chrono::DateTime::from_timestamp_millis(u.recorded_at)
                        .map(|dt| dt.to_rfc3339())
                        .unwrap_or_default(),
                })
                .collect();

            total_batches += 1;

            match self
                .server_client
                .push_token_usages(&account.url, &token, workspace_id, &payload)
                .await
            {
                Ok(()) => {
                    let ids: Vec<String> = unpushed.iter().map(|u| u.id.clone()).collect();
                    self.store.mark_pushed(&ids).await?;
                    total_pushed += unpushed.len();
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("duplicate") || err_str.contains("already exists") {
                        tracing::warn!(
                            "Batch contains duplicates on remote, marking as pushed: {e}"
                        );
                        let ids: Vec<String> = unpushed.iter().map(|u| u.id.clone()).collect();
                        self.store.mark_pushed(&ids).await?;
                        total_pushed += unpushed.len();
                    } else {
                        tracing::error!("Failed to push batch: {e}");
                        total_failed += unpushed.len();
                        break;
                    }
                }
            }
        }

        let _ = self.store.cleanup_old_pushed(DEFAULT_RETENTION_DAYS).await;

        Ok(PushResult {
            pushed_count: total_pushed,
            failed_count: total_failed,
            total_batches,
        })
    }

    pub async fn list_recent(&self, limit: usize, offset: usize) -> Result<Vec<TokenUsage>> {
        self.store.list_recent(limit, offset).await
    }

    pub async fn count_all(&self) -> Result<usize> {
        self.store.count_all().await
    }

    pub async fn summary(&self, days: Option<i64>) -> Result<Vec<UsageSummary>> {
        self.store.summary(days).await
    }

    pub fn start_auto_push(self: Arc<Self>, interval_secs: u64, batch_size: usize) -> Arc<Notify> {
        let stop = Arc::new(Notify::new());
        let stop_clone = stop.clone();
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match self.store.count_unpushed().await {
                            Ok(0) => {}
                            Ok(count) => {
                                tracing::debug!("Auto-push: {} unpushed records found", count);
                                match self.push_batch(Some(batch_size)).await {
                                    Ok(result) => {
                                        if result.pushed_count > 0 {
                                            tracing::info!(
                                                "Auto-push: pushed {} records in {} batches",
                                                result.pushed_count,
                                                result.total_batches
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Auto-push failed: {e}");
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Auto-push count check failed: {e}");
                            }
                        }
                    }
                    _ = stop_clone.notified() => {
                        tracing::info!("Auto-push stopped");
                        break;
                    }
                }
            }
        });
        stop
    }
}
