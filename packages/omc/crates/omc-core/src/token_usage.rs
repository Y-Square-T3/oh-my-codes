use crate::model::ModelCost;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TokenCost {
    pub usage_id: String,
    pub input_cost_micros: i64,
    pub output_cost_micros: i64,
    pub reasoning_cost_micros: i64,
    pub cache_read_cost_micros: i64,
    pub cache_write_cost_micros: i64,
    pub audio_input_cost_micros: i64,
    pub video_input_cost_micros: i64,
    pub image_input_cost_micros: i64,
    pub total_cost_micros: i64,
}

pub fn calculate_cost(usage: &TokenUsage, model_cost: &ModelCost) -> TokenCost {
    let input = (usage.input_tokens as f64 * model_cost.input * 1_000_000.0).round() as i64;
    let output = (usage.output_tokens as f64 * model_cost.output * 1_000_000.0).round() as i64;
    let reasoning = model_cost
        .reasoning
        .map(|rate| (usage.reasoning_tokens as f64 * rate * 1_000_000.0).round() as i64)
        .unwrap_or(0);
    let cache_read = model_cost
        .cache_read
        .map(|rate| (usage.cache_read_tokens as f64 * rate * 1_000_000.0).round() as i64)
        .unwrap_or(0);
    let cache_write = model_cost
        .cache_write
        .map(|rate| (usage.cache_write_tokens as f64 * rate * 1_000_000.0).round() as i64)
        .unwrap_or(0);
    let total = input + output + reasoning + cache_read + cache_write;

    TokenCost {
        usage_id: usage.id.clone(),
        input_cost_micros: input,
        output_cost_micros: output,
        reasoning_cost_micros: reasoning,
        cache_read_cost_micros: cache_read,
        cache_write_cost_micros: cache_write,
        audio_input_cost_micros: 0,
        video_input_cost_micros: 0,
        image_input_cost_micros: 0,
        total_cost_micros: total,
    }
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
    use crate::model::ModelCost;

    fn make_test_usage(
        input: i64,
        output: i64,
        reasoning: i64,
        cache_read: i64,
        cache_write: i64,
    ) -> TokenUsage {
        TokenUsage {
            id: "test-usage-id".to_string(),
            workspace_id: None,
            session_id: "session-1".to_string(),
            agent: "agent".to_string(),
            model: "model".to_string(),
            metadata: None,
            input_tokens: input,
            output_tokens: output,
            reasoning_tokens: reasoning,
            cache_read_tokens: cache_read,
            cache_write_tokens: cache_write,
            audio_input_tokens: 0,
            video_input_tokens: 0,
            image_input_tokens: 0,
            total_tokens: input + output + reasoning + cache_read + cache_write,
            pushed: false,
            recorded_at: 0,
            created_at: 0,
        }
    }

    #[test]
    fn calculate_cost_basic() {
        let usage = make_test_usage(1000, 500, 0, 0, 0);
        let model_cost = ModelCost {
            input: 0.000005,
            output: 0.000015,
            reasoning: None,
            cache_read: None,
            cache_write: None,
            context_over_200k: None,
        };
        let cost = calculate_cost(&usage, &model_cost);
        assert_eq!(cost.usage_id, "test-usage-id");
        assert_eq!(cost.input_cost_micros, 5000);
        assert_eq!(cost.output_cost_micros, 7500);
        assert_eq!(cost.reasoning_cost_micros, 0);
        assert_eq!(cost.cache_read_cost_micros, 0);
        assert_eq!(cost.cache_write_cost_micros, 0);
        assert_eq!(cost.total_cost_micros, 12500);
    }

    #[test]
    fn calculate_cost_with_all_rates() {
        let usage = make_test_usage(1000, 500, 200, 300, 100);
        let model_cost = ModelCost {
            input: 0.00001,
            output: 0.00002,
            reasoning: Some(0.00003),
            cache_read: Some(0.000001),
            cache_write: Some(0.000005),
            context_over_200k: None,
        };
        let cost = calculate_cost(&usage, &model_cost);
        assert_eq!(cost.input_cost_micros, 10000);
        assert_eq!(cost.output_cost_micros, 10000);
        assert_eq!(cost.reasoning_cost_micros, 6000);
        assert_eq!(cost.cache_read_cost_micros, 300);
        assert_eq!(cost.cache_write_cost_micros, 500);
        assert_eq!(cost.total_cost_micros, 26800);
    }

    #[test]
    fn calculate_cost_zero_tokens() {
        let usage = make_test_usage(0, 0, 0, 0, 0);
        let model_cost = ModelCost {
            input: 0.00001,
            output: 0.00002,
            reasoning: None,
            cache_read: None,
            cache_write: None,
            context_over_200k: None,
        };
        let cost = calculate_cost(&usage, &model_cost);
        assert_eq!(cost.total_cost_micros, 0);
    }

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
