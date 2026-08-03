use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub id: String,
    pub client: String,
    pub session_id: String,
    pub message_id: String,
    pub agent: Option<String>,
    pub provider_id: String,
    pub model_id: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub pushed: bool,
    pub recorded_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub provider_id: String,
    pub model_id: String,
    pub total_input: i64,
    pub total_output: i64,
    pub total_reasoning: i64,
    pub total_cache_read: i64,
    pub total_cache_write: i64,
    pub request_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsageOverview {
    pub headline: HeadlineStats,
    pub top_models: Vec<UsageSummary>,
    pub top_agents: Vec<UsageGroup>,
    pub top_clients: Vec<UsageGroup>,
    pub trend: Vec<DailyUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadlineStats {
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub unpushed_records: usize,
    pub unpushed_tokens: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageGroup {
    pub label: String,
    pub total_input: i64,
    pub total_output: i64,
    pub total_reasoning: i64,
    pub total_cache_read: i64,
    pub total_cache_write: i64,
    pub request_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyUsage {
    pub date: String,
    pub requests: i64,
    pub total_tokens: i64,
}
