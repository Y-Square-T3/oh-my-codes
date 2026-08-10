use omc_core::token_usage::TokenCost;
use omc_storage::StorageBackend;

use crate::common::builders::make_usage;

fn make_cost(usage_id: &str) -> TokenCost {
    TokenCost {
        usage_id: usage_id.to_string(),
        input_cost_micros: 5000,
        output_cost_micros: 7500,
        reasoning_cost_micros: 3000,
        cache_read_cost_micros: 100,
        cache_write_cost_micros: 200,
        audio_input_cost_micros: 0,
        video_input_cost_micros: 0,
        image_input_cost_micros: 0,
        total_cost_micros: 15800,
    }
}

pub async fn test_upsert_token_cost<B: StorageBackend>(backend: &B) {
    let usage = make_usage("vscode/agent", "openai/gpt-4o", 1000, 500, 100, false);
    backend.upsert_usage(&usage).await.unwrap();

    let cost = make_cost(&usage.id);
    backend.upsert_token_cost(&cost).await.unwrap();

    let retrieved = backend.get_token_cost(&usage.id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.usage_id, usage.id);
    assert_eq!(retrieved.input_cost_micros, 5000);
    assert_eq!(retrieved.output_cost_micros, 7500);
    assert_eq!(retrieved.reasoning_cost_micros, 3000);
    assert_eq!(retrieved.cache_read_cost_micros, 100);
    assert_eq!(retrieved.cache_write_cost_micros, 200);
    assert_eq!(retrieved.total_cost_micros, 15800);
}

pub async fn test_get_token_cost_not_found<B: StorageBackend>(backend: &B) {
    let retrieved = backend.get_token_cost("nonexistent-id").await.unwrap();
    assert!(retrieved.is_none());
}

pub async fn test_upsert_token_cost_immutable<B: StorageBackend>(backend: &B) {
    let usage = make_usage("vscode/agent", "openai/gpt-4o", 1000, 500, 100, false);
    backend.upsert_usage(&usage).await.unwrap();

    let cost = make_cost(&usage.id);
    backend.upsert_token_cost(&cost).await.unwrap();

    let mut updated_cost = make_cost(&usage.id);
    updated_cost.input_cost_micros = 99999;
    updated_cost.total_cost_micros = 99999;
    backend.upsert_token_cost(&updated_cost).await.unwrap();

    let retrieved = backend.get_token_cost(&usage.id).await.unwrap().unwrap();
    assert_eq!(retrieved.input_cost_micros, 5000);
    assert_eq!(retrieved.total_cost_micros, 15800);
}

pub async fn test_upsert_token_cost_requires_usage<B: StorageBackend>(backend: &B) {
    let cost = TokenCost {
        usage_id: "nonexistent-usage".to_string(),
        input_cost_micros: 100,
        output_cost_micros: 200,
        reasoning_cost_micros: 0,
        cache_read_cost_micros: 0,
        cache_write_cost_micros: 0,
        audio_input_cost_micros: 0,
        video_input_cost_micros: 0,
        image_input_cost_micros: 0,
        total_cost_micros: 300,
    };
    let result = backend.upsert_token_cost(&cost).await;
    assert!(result.is_err());
}
