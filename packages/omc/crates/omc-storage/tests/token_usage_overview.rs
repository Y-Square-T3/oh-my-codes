use omc_core::token_usage::TokenUsage;
use omc_storage::StorageBackend;
use omc_storage::backend::sqlite::SqliteBackend;

#[allow(clippy::too_many_arguments)]
fn make_usage(
    id: &str,
    agent: &str,
    model: &str,
    input: i64,
    output: i64,
    reasoning: i64,
    pushed: bool,
) -> TokenUsage {
    let now = chrono::Utc::now().timestamp_millis();
    let message_id = format!("msg-{id}");
    let total_tokens = input + output + reasoning;
    TokenUsage {
        id: id.to_string(),
        workspace_id: None,
        session_id: format!("session-{id}"),
        agent: agent.to_string(),
        model: model.to_string(),
        metadata: Some(format!(r#"{{"messageId":"{message_id}"}}"#)),
        input_tokens: input,
        output_tokens: output,
        reasoning_tokens: reasoning,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        audio_input_tokens: 0,
        video_input_tokens: 0,
        image_input_tokens: 0,
        total_tokens,
        pushed,
        recorded_at: now,
        created_at: now,
    }
}

#[tokio::test]
async fn test_overview_empty() {
    let backend = SqliteBackend::new_memory().await.unwrap();
    let overview = backend.usage_overview(None).await.unwrap();

    assert_eq!(overview.headline.requests, 0);
    assert_eq!(overview.headline.input_tokens, 0);
    assert!(overview.top_models.is_empty());
    assert!(overview.top_agents.is_empty());
}

#[tokio::test]
async fn test_overview_with_data() {
    let backend = SqliteBackend::new_memory().await.unwrap();

    let usages = vec![
        make_usage(
            "1",
            "vscode/agent-a",
            "openai/gpt-4o",
            1000,
            500,
            100,
            false,
        ),
        make_usage("2", "vscode/agent-a", "openai/gpt-4o", 2000, 800, 200, true),
        make_usage(
            "3",
            "cursor/agent-b",
            "anthropic/claude-3",
            3000,
            1200,
            300,
            false,
        ),
        make_usage("4", "vim/unknown", "openai/o1-mini", 500, 200, 50, true),
    ];

    for u in &usages {
        backend.upsert_usage(u).await.unwrap();
    }

    let overview = backend.usage_overview(None).await.unwrap();

    assert_eq!(overview.headline.requests, 4);
    assert_eq!(overview.headline.input_tokens, 6500);
    assert_eq!(overview.headline.output_tokens, 2700);
    assert_eq!(overview.headline.reasoning_tokens, 650);
    assert_eq!(overview.headline.unpushed_records, 2);
    assert!(overview.headline.unpushed_tokens > 0);

    assert_eq!(overview.top_models.len(), 3);
    assert_eq!(overview.top_models[0].model, "openai/gpt-4o");
    assert_eq!(overview.top_models[1].model, "anthropic/claude-3");
    assert_eq!(overview.top_models[2].model, "openai/o1-mini");

    assert_eq!(overview.top_agents.len(), 3);
    assert_eq!(overview.top_agents[0].label, "vscode/agent-a");
    assert_eq!(overview.top_agents[1].label, "cursor/agent-b");
    assert_eq!(overview.top_agents[2].label, "vim/unknown");
}

#[tokio::test]
async fn test_overview_with_days_filter() {
    let backend = SqliteBackend::new_memory().await.unwrap();

    let now = chrono::Utc::now().timestamp_millis();
    let old = now - (10 * 86_400_000);

    let mut usage = make_usage("1", "vscode/agent", "openai/gpt-4o", 1000, 500, 100, false);
    usage.recorded_at = old;
    backend.upsert_usage(&usage).await.unwrap();

    let usage2 = make_usage(
        "2",
        "cursor/agent",
        "anthropic/claude-3",
        2000,
        800,
        200,
        false,
    );
    backend.upsert_usage(&usage2).await.unwrap();

    let overview = backend.usage_overview(Some(7)).await.unwrap();

    assert_eq!(overview.headline.requests, 1);
    assert_eq!(overview.headline.input_tokens, 2000);
    assert_eq!(overview.top_models.len(), 1);
    assert_eq!(overview.top_models[0].model, "anthropic/claude-3");
}
