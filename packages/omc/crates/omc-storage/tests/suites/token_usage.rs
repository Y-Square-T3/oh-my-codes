use omc_storage::StorageBackend;

use crate::common::builders::make_usage;

pub async fn test_upsert_usage<B: StorageBackend>(backend: &B) {
    let usage = make_usage("vscode", Some("agent"), "openai", "gpt-4o", 1000, 500, 100, false);
    backend.upsert_usage(&usage).await.unwrap();

    let recent = backend.list_recent(10, 0, None).await.unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].id, usage.id);
    assert_eq!(recent[0].client, "vscode");
    assert_eq!(recent[0].provider_id, "openai");
    assert_eq!(recent[0].model_id, "gpt-4o");
    assert_eq!(recent[0].input_tokens, 1000);
    assert_eq!(recent[0].output_tokens, 500);
    assert_eq!(recent[0].reasoning_tokens, 100);
}

pub async fn test_upsert_usage_conflict_updates<B: StorageBackend>(backend: &B) {
    let mut usage = make_usage("vscode", Some("agent"), "openai", "gpt-4o", 1000, 500, 100, false);
    backend.upsert_usage(&usage).await.unwrap();

    usage.input_tokens = 2000;
    usage.output_tokens = 1000;
    backend.upsert_usage(&usage).await.unwrap();

    let recent = backend.list_recent(10, 0, None).await.unwrap();
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].input_tokens, 2000);
    assert_eq!(recent[0].output_tokens, 1000);
}

pub async fn test_find_unpushed_empty<B: StorageBackend>(backend: &B) {
    let unpushed = backend.find_unpushed(10).await.unwrap();
    assert!(unpushed.is_empty());
}

pub async fn test_find_unpushed<B: StorageBackend>(backend: &B) {
    let usage1 = make_usage("vscode", None, "openai", "gpt-4o", 100, 50, 10, false);
    let usage2 = make_usage("cursor", None, "anthropic", "claude", 200, 100, 20, true);
    let usage3 = make_usage("vim", None, "openai", "gpt-4", 300, 150, 30, false);

    backend.upsert_usage(&usage1).await.unwrap();
    backend.upsert_usage(&usage2).await.unwrap();
    backend.upsert_usage(&usage3).await.unwrap();

    let unpushed = backend.find_unpushed(10).await.unwrap();
    assert_eq!(unpushed.len(), 2);
}

pub async fn test_find_unpushed_with_limit<B: StorageBackend>(backend: &B) {
    for _ in 0..5 {
        let usage = make_usage("client", None, "provider", "model", 100, 50, 10, false);
        backend.upsert_usage(&usage).await.unwrap();
    }

    let unpushed = backend.find_unpushed(3).await.unwrap();
    assert_eq!(unpushed.len(), 3);
}

pub async fn test_count_unpushed_empty<B: StorageBackend>(backend: &B) {
    let count = backend.count_unpushed().await.unwrap();
    assert_eq!(count, 0);
}

pub async fn test_count_unpushed<B: StorageBackend>(backend: &B) {
    let usage1 = make_usage("vscode", None, "openai", "gpt-4o", 100, 50, 10, false);
    let usage2 = make_usage("cursor", None, "anthropic", "claude", 200, 100, 20, true);
    let usage3 = make_usage("vim", None, "openai", "gpt-4", 300, 150, 30, false);

    backend.upsert_usage(&usage1).await.unwrap();
    backend.upsert_usage(&usage2).await.unwrap();
    backend.upsert_usage(&usage3).await.unwrap();

    let count = backend.count_unpushed().await.unwrap();
    assert_eq!(count, 2);
}

pub async fn test_mark_pushed<B: StorageBackend>(backend: &B) {
    let usage1 = make_usage("vscode", None, "openai", "gpt-4o", 100, 50, 10, false);
    let usage2 = make_usage("cursor", None, "anthropic", "claude", 200, 100, 20, false);

    backend.upsert_usage(&usage1).await.unwrap();
    backend.upsert_usage(&usage2).await.unwrap();

    backend.mark_pushed(std::slice::from_ref(&usage1.id)).await.unwrap();

    let unpushed = backend.find_unpushed(10).await.unwrap();
    assert_eq!(unpushed.len(), 1);
    assert_eq!(unpushed[0].id, usage2.id);
}

pub async fn test_mark_pushed_empty_ids<B: StorageBackend>(backend: &B) {
    let result = backend.mark_pushed(&[]).await;
    assert!(result.is_ok());
}

pub async fn test_list_recent_empty<B: StorageBackend>(backend: &B) {
    let recent = backend.list_recent(10, 0, None).await.unwrap();
    assert!(recent.is_empty());
}

pub async fn test_list_recent_with_limit_and_offset<B: StorageBackend>(backend: &B) {
    for _ in 0..5 {
        let usage = make_usage("client", None, "provider", "model", 100, 50, 10, false);
        backend.upsert_usage(&usage).await.unwrap();
    }

    let recent = backend.list_recent(2, 0, None).await.unwrap();
    assert_eq!(recent.len(), 2);

    let recent = backend.list_recent(2, 2, None).await.unwrap();
    assert_eq!(recent.len(), 2);

    let recent = backend.list_recent(2, 4, None).await.unwrap();
    assert_eq!(recent.len(), 1);
}

pub async fn test_list_recent_filter_by_pushed<B: StorageBackend>(backend: &B) {
    let usage1 = make_usage("vscode", None, "openai", "gpt-4o", 100, 50, 10, false);
    let usage2 = make_usage("cursor", None, "anthropic", "claude", 200, 100, 20, true);
    let usage3 = make_usage("vim", None, "openai", "gpt-4", 300, 150, 30, false);

    backend.upsert_usage(&usage1).await.unwrap();
    backend.upsert_usage(&usage2).await.unwrap();
    backend.upsert_usage(&usage3).await.unwrap();

    let unpushed = backend.list_recent(10, 0, Some(false)).await.unwrap();
    assert_eq!(unpushed.len(), 2);

    let pushed = backend.list_recent(10, 0, Some(true)).await.unwrap();
    assert_eq!(pushed.len(), 1);
}

pub async fn test_count_all_empty<B: StorageBackend>(backend: &B) {
    let count = backend.count_all(None).await.unwrap();
    assert_eq!(count, 0);
}

pub async fn test_count_all<B: StorageBackend>(backend: &B) {
    let usage1 = make_usage("vscode", None, "openai", "gpt-4o", 100, 50, 10, false);
    let usage2 = make_usage("cursor", None, "anthropic", "claude", 200, 100, 20, true);

    backend.upsert_usage(&usage1).await.unwrap();
    backend.upsert_usage(&usage2).await.unwrap();

    let count = backend.count_all(None).await.unwrap();
    assert_eq!(count, 2);

    let unpushed_count = backend.count_all(Some(false)).await.unwrap();
    assert_eq!(unpushed_count, 1);

    let pushed_count = backend.count_all(Some(true)).await.unwrap();
    assert_eq!(pushed_count, 1);
}

pub async fn test_cleanup_old_pushed<B: StorageBackend>(backend: &B) {
    let now = chrono::Utc::now().timestamp_millis();
    let old = now - (10 * 86_400_000);

    let mut old_usage = make_usage("vscode", None, "openai", "gpt-4o", 100, 50, 10, true);
    old_usage.recorded_at = old;
    backend.upsert_usage(&old_usage).await.unwrap();

    let recent_usage = make_usage("cursor", None, "anthropic", "claude", 200, 100, 20, true);
    backend.upsert_usage(&recent_usage).await.unwrap();

    let unpushed_usage = make_usage("vim", None, "openai", "gpt-4", 300, 150, 30, false);
    backend.upsert_usage(&unpushed_usage).await.unwrap();

    let deleted = backend.cleanup_old_pushed(7).await.unwrap();
    assert_eq!(deleted, 1);

    let all = backend.list_recent(10, 0, None).await.unwrap();
    assert_eq!(all.len(), 2);
}

pub async fn test_usage_summary_empty<B: StorageBackend>(backend: &B) {
    let summary = backend.usage_summary(None).await.unwrap();
    assert!(summary.is_empty());
}

pub async fn test_usage_summary<B: StorageBackend>(backend: &B) {
    let usage1 = make_usage("vscode", Some("agent"), "openai", "gpt-4o", 1000, 500, 100, false);
    let usage2 = make_usage("cursor", Some("agent"), "openai", "gpt-4o", 2000, 1000, 200, false);
    let usage3 = make_usage("vim", None, "anthropic", "claude", 3000, 1500, 300, false);

    backend.upsert_usage(&usage1).await.unwrap();
    backend.upsert_usage(&usage2).await.unwrap();
    backend.upsert_usage(&usage3).await.unwrap();

    let summary = backend.usage_summary(None).await.unwrap();
    assert_eq!(summary.len(), 2);

    let openai_summary = summary.iter().find(|s| s.provider_id == "openai").unwrap();
    assert_eq!(openai_summary.total_input, 3000);
    assert_eq!(openai_summary.total_output, 1500);
    assert_eq!(openai_summary.request_count, 2);
}
