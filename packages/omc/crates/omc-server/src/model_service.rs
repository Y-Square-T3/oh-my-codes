use crate::server_client::OmcServerClient;
use omc_core::error::{OmcError, Result};
use omc_core::model::{Model, Provider};
use omc_storage::model_store::ModelStore;
use std::sync::Arc;

pub struct ModelInfo {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub family: Option<String>,
    pub reasoning: Option<bool>,
    pub tool_call: Option<bool>,
    pub attachment: Option<bool>,
    pub temperature: Option<bool>,
    pub open_weights: Option<bool>,
    pub modalities_input: Vec<String>,
    pub modalities_output: Vec<String>,
    pub cost_input: f64,
    pub cost_output: f64,
    pub limit_context: Option<i64>,
    pub limit_output: Option<i64>,
    pub release_date: Option<String>,
}

pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub api: Option<String>,
    pub npm: Option<String>,
    pub model_count: usize,
}

pub struct ListResult {
    pub providers: Vec<ProviderInfo>,
    pub models: Vec<ModelInfo>,
    pub account_email: Option<String>,
    pub account_url: Option<String>,
}

pub struct SyncResult {
    pub providers: usize,
    pub models: usize,
}

pub struct ModelService {
    model_store: Arc<dyn ModelStore>,
    account_service: Arc<crate::account_service::AccountService>,
    server_client: OmcServerClient,
}

impl ModelService {
    pub fn new(
        model_store: Arc<dyn ModelStore>,
        account_service: Arc<crate::account_service::AccountService>,
        server_client: OmcServerClient,
    ) -> Self {
        Self {
            model_store,
            account_service,
            server_client,
        }
    }

    pub async fn list(&self, provider_id: Option<&str>) -> Result<ListResult> {
        let account_opt = self.account_service.active().await?;

        let Some(account) = account_opt else {
            return Ok(ListResult {
                providers: vec![],
                models: vec![],
                account_email: None,
                account_url: None,
            });
        };

        let providers = self.model_store.list_providers(&account.id).await?;

        let filtered_providers = if let Some(pid) = provider_id {
            providers
                .into_iter()
                .filter(|p| p.id == pid)
                .collect::<Vec<_>>()
        } else {
            providers
        };

        let mut provider_infos = Vec::new();
        let mut model_infos = Vec::new();

        for p in &filtered_providers {
            let model_count = p.models.len();
            provider_infos.push(ProviderInfo {
                id: p.id.clone(),
                name: p.name.clone(),
                api: p.api.clone(),
                npm: p.npm.clone(),
                model_count,
            });

            for m in &p.models {
                model_infos.push(model_to_info(m, &p.id));
            }
        }

        Ok(ListResult {
            providers: provider_infos,
            models: model_infos,
            account_email: Some(account.email),
            account_url: Some(account.url),
        })
    }

    pub async fn sync(&self) -> Result<SyncResult> {
        let (account, token) =
            self.account_service
                .active_with_token()
                .await?
                .ok_or_else(|| {
                    OmcError::Auth("No account logged in. Run `account login` first.".into())
                })?;

        let remote_providers = self
            .server_client
            .fetch_models(&account.url, &token)
            .await?;

        let now = chrono::Utc::now().timestamp();
        let mut total_models = 0;

        let providers: Vec<Provider> = remote_providers
            .into_iter()
            .map(|(id, rp)| {
                let models: Vec<Model> = rp
                    .models
                    .into_iter()
                    .map(|(model_id, mut m)| {
                        if m.id.is_empty() {
                            m.id = model_id;
                        }
                        m
                    })
                    .collect();
                total_models += models.len();
                Provider {
                    id: if rp.id.is_empty() { id } else { rp.id },
                    name: rp.name,
                    env: rp.env,
                    api: rp.api,
                    npm: rp.npm,
                    doc: rp.doc,
                    models,
                    account_id: account.id.clone(),
                    last_fetched_at: now,
                }
            })
            .collect();

        let providers_count = providers.len();
        self.model_store
            .replace_providers(&account.id, providers)
            .await?;

        Ok(SyncResult {
            providers: providers_count,
            models: total_models,
        })
    }
}

fn model_to_info(m: &Model, provider_id: &str) -> ModelInfo {
    let (modalities_input, modalities_output) = m
        .modalities
        .as_ref()
        .map(|mod_| (mod_.input.clone(), mod_.output.clone()))
        .unwrap_or_default();

    ModelInfo {
        id: m.id.clone(),
        provider_id: provider_id.to_string(),
        name: m.name.clone(),
        family: m.family.clone(),
        reasoning: m.reasoning,
        tool_call: m.tool_call,
        attachment: m.attachment,
        temperature: m.temperature,
        open_weights: m.open_weights,
        modalities_input,
        modalities_output,
        cost_input: m.cost.as_ref().map(|c| c.input).unwrap_or(0.0),
        cost_output: m.cost.as_ref().map(|c| c.output).unwrap_or(0.0),
        limit_context: Some(m.limit.context),
        limit_output: Some(m.limit.output),
        release_date: m.release_date.clone(),
    }
}
