use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachment: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interleaved: Option<InterleavedMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<ModelCost>,
    pub limit: ModelLimit,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<ModelModalities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<ExperimentalConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ModelStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub knowledge: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_weights: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ModelProviderRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InterleavedMode {
    True(bool),
    Field { field: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCost {
    pub input: f64,
    pub output: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_over_200k: Option<ContextCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextCost {
    pub input: f64,
    pub output: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLimit {
    pub context: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<i64>,
    pub output: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelModalities {
    pub input: Vec<String>,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    Alpha,
    Beta,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modes: Option<std::collections::HashMap<String, ExperimentalMode>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentalMode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<ModelCost>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ExperimentalProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentalProvider {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<std::collections::HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProviderRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub env: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    pub models: Vec<Model>,
    pub account_id: String,
    pub last_fetched_at: i64,
}
