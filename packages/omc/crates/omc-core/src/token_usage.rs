use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub id: String,
    pub workspace_id: Option<String>,
    pub session_id: String,
    pub agent: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub audio_input_tokens: i64,
    pub video_input_tokens: i64,
    pub image_input_tokens: i64,
    pub total_tokens: i64,
    pub pushed: bool,
    pub recorded_at: i64,
    pub created_at: i64,
}

pub fn generate_id(agent: &str, message_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(agent.as_bytes());
    hasher.update(b":");
    hasher.update(message_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..16])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSummary {
    pub model: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_id_is_deterministic() {
        let id1 = generate_id("claude/sonnet", "msg-123");
        let id2 = generate_id("claude/sonnet", "msg-123");
        assert_eq!(id1, id2);
    }

    #[test]
    fn generate_id_differs_for_different_inputs() {
        let id1 = generate_id("claude/sonnet", "msg-123");
        let id2 = generate_id("claude/sonnet", "msg-456");
        let id3 = generate_id("vscode/copilot", "msg-123");
        assert_ne!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn generate_id_is_32_hex_chars() {
        let id = generate_id("agent", "message");
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
