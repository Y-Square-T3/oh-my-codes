use omc_core::token_usage::TokenUsage;
use omc_storage::StorageBackend;
use omc_storage::backend::sqlite::SqliteBackend;

#[allow(clippy::too_many_arguments)]
fn make_usage(
    id: &str,
    client: &str,
    agent: Option<&str>,
    provider: &str,
    model: &str,
    input: i64,
    output: i64,
    reasoning: i64,
    pushed: bool,
) -> TokenUsage {
    let now = chrono::Utc::now().timestamp_millis();
    TokenUsage {
        id: id.to_string(),
        client: client.to_string(),
        session_id: format!("session-{id}"),
        message_id: format!("msg-{id}"),
        agent: agent.map(|s| s.to_string()),
        provider_id: provider.to_string(),
        model_id: model.to_string(),
        input_tokens: input,
        output_tokens: output,
        reasoning_tokens: reasoning,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
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
    assert!(overview.top_clients.is_empty());
}

#[tokio::test]
async fn test_overview_with_data() {
    let backend = SqliteBackend::new_memory().await.unwrap();

    let usages = vec![
        make_usage(
            "1",
            "vscode",
            Some("agent-a"),
            "openai",
            "gpt-4o",
            1000,
            500,
            100,
            false,
        ),
        make_usage(
            "2",
            "vscode",
            Some("agent-a"),
            "openai",
            "gpt-4o",
            2000,
            800,
            200,
            true,
        ),
        make_usage(
            "3",
            "cursor",
            Some("agent-b"),
            "anthropic",
            "claude-3",
            3000,
            1200,
            300,
            false,
        ),
        make_usage("4", "vim", None, "openai", "o1-mini", 500, 200, 50, true),
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
    assert_eq!(overview.top_models[0].model_id, "gpt-4o");
    assert_eq!(overview.top_models[1].model_id, "claude-3");
    assert_eq!(overview.top_models[2].model_id, "o1-mini");

    assert_eq!(overview.top_agents.len(), 3);
    assert_eq!(overview.top_agents[0].label, "agent-a");
    assert_eq!(overview.top_agents[1].label, "agent-b");
    assert_eq!(overview.top_agents[2].label, "unknown");

    assert_eq!(overview.top_clients.len(), 3);
    assert_eq!(overview.top_clients[0].label, "vscode");
    assert_eq!(overview.top_clients[1].label, "cursor");
    assert_eq!(overview.top_clients[2].label, "vim");
}

#[tokio::test]
async fn test_overview_with_days_filter() {
    let backend = SqliteBackend::new_memory().await.unwrap();

    let now = chrono::Utc::now().timestamp_millis();
    let old = now - (10 * 86_400_000);

    let mut usage = make_usage(
        "1",
        "vscode",
        Some("agent"),
        "openai",
        "gpt-4o",
        1000,
        500,
        100,
        false,
    );
    usage.recorded_at = old;
    backend.upsert_usage(&usage).await.unwrap();

    let usage2 = make_usage(
        "2",
        "cursor",
        Some("agent"),
        "anthropic",
        "claude-3",
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
    assert_eq!(overview.top_models[0].model_id, "claude-3");
}
