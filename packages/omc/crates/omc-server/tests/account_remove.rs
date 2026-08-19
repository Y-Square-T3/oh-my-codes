use omc_core::account::{Account, Workspace};
use omc_core::model::{Model, ModelLimit, Provider};
use omc_server::AccountService;
use omc_server::server_client::OmcServerClient;
use omc_storage::StorageBackend;
use omc_storage::backend::sqlite::SqliteBackend;
use std::sync::Arc;
use ulid::Ulid;

fn make_account() -> Account {
    let id = Ulid::new().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    Account {
        id: id.clone(),
        email: format!("test-{id}@example.com"),
        url: "https://api.example.com".to_string(),
        access_token: format!("access-{id}"),
        refresh_token: format!("refresh-{id}"),
        token_expiry: now + 3_600_000,
        active_workspace_id: None,
    }
}

fn make_workspace(account_id: &str) -> Workspace {
    let id = Ulid::new().to_string();
    Workspace {
        id: id.clone(),
        account_id: account_id.to_string(),
        name: format!("workspace-{}", &id[..8]),
        is_admin: false,
    }
}

fn make_provider(account_id: &str) -> Provider {
    let id = Ulid::new().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    Provider {
        id: id.clone(),
        name: format!("provider-{}", &id[..8]),
        env: vec!["API_KEY".to_string()],
        api: Some("https://api.provider.com".to_string()),
        npm: None,
        doc: Some("https://docs.provider.com".to_string()),
        models: vec![Model {
            id: Ulid::new().to_string(),
            name: "test-model".to_string(),
            family: None,
            release_date: None,
            last_updated: None,
            attachment: None,
            reasoning: None,
            temperature: None,
            tool_call: None,
            interleaved: None,
            cost: None,
            limit: ModelLimit {
                context: 128_000,
                input: None,
                output: 4_096,
            },
            modalities: None,
            experimental: None,
            structured_output: None,
            knowledge: None,
            open_weights: None,
            provider: None,
            status: None,
        }],
        account_id: account_id.to_string(),
        last_fetched_at: now,
    }
}

#[tokio::test]
async fn test_account_remove_with_all_relations() {
    let backend = SqliteBackend::new_memory().await.unwrap();
    let backend = Arc::new(backend);
    let client = OmcServerClient::new();
    let service = AccountService::new(backend.clone(), client);

    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let workspace = make_workspace(&account.id);
    backend.upsert_workspaces(&[workspace]).await.unwrap();

    let provider = make_provider(&account.id);
    backend
        .replace_providers(&account.id, vec![provider])
        .await
        .unwrap();

    backend.set_active_account(&account.id).await.unwrap();

    let result = service.remove(&account.id).await;
    assert!(result.is_ok(), "remove() failed: {:?}", result.err());

    let deleted = backend.get_account(&account.id).await.unwrap();
    assert!(deleted.is_none(), "account still exists after remove");

    let workspaces = backend.list_workspaces(&account.id).await.unwrap();
    assert!(workspaces.is_empty(), "workspaces not cleaned up");

    let providers = backend.list_providers(&account.id).await.unwrap();
    assert!(providers.is_empty(), "providers not cleaned up");

    let active = backend.get_active_account_id().await.unwrap();
    assert!(active.is_none(), "active_account not cleared");
}
