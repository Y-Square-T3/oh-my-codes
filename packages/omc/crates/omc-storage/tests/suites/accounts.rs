use omc_storage::StorageBackend;

use crate::common::builders::{make_account, make_provider};

pub async fn test_get_account_empty<B: StorageBackend>(backend: &B) {
    let result = backend.get_account("nonexistent").await.unwrap();
    assert!(result.is_none());
}

pub async fn test_upsert_and_get_account<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let retrieved = backend.get_account(&account.id).await.unwrap().unwrap();
    assert_eq!(retrieved.id, account.id);
    assert_eq!(retrieved.email, account.email);
    assert_eq!(retrieved.url, account.url);
    assert_eq!(retrieved.access_token, account.access_token);
    assert_eq!(retrieved.refresh_token, account.refresh_token);
    assert_eq!(retrieved.token_expiry, account.token_expiry);
    assert_eq!(retrieved.active_workspace_id, account.active_workspace_id);
}

pub async fn test_upsert_account_updates_existing<B: StorageBackend>(backend: &B) {
    let mut account = make_account();
    backend.upsert_account(&account).await.unwrap();

    account.email = "updated@example.com".to_string();
    account.access_token = "new-access-token".to_string();
    backend.upsert_account(&account).await.unwrap();

    let retrieved = backend.get_account(&account.id).await.unwrap().unwrap();
    assert_eq!(retrieved.email, "updated@example.com");
    assert_eq!(retrieved.access_token, "new-access-token");
}

pub async fn test_list_accounts_empty<B: StorageBackend>(backend: &B) {
    let accounts = backend.list_accounts().await.unwrap();
    assert!(accounts.is_empty());
}

pub async fn test_list_accounts_multiple<B: StorageBackend>(backend: &B) {
    let account1 = make_account();
    let account2 = make_account();
    backend.upsert_account(&account1).await.unwrap();
    backend.upsert_account(&account2).await.unwrap();

    let accounts = backend.list_accounts().await.unwrap();
    assert_eq!(accounts.len(), 2);

    let ids: Vec<&str> = accounts.iter().map(|a| a.id.as_str()).collect();
    assert!(ids.contains(&account1.id.as_str()));
    assert!(ids.contains(&account2.id.as_str()));
}

pub async fn test_delete_account<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    backend.delete_account(&account.id).await.unwrap();

    let result = backend.get_account(&account.id).await.unwrap();
    assert!(result.is_none());
}

pub async fn test_delete_nonexistent_account<B: StorageBackend>(backend: &B) {
    let result = backend.delete_account("nonexistent").await;
    assert!(result.is_ok());
}

pub async fn test_get_active_account_id_none<B: StorageBackend>(backend: &B) {
    let result = backend.get_active_account_id().await.unwrap();
    assert!(result.is_none());
}

pub async fn test_set_and_get_active_account<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    backend.set_active_account(&account.id).await.unwrap();

    let active_id = backend.get_active_account_id().await.unwrap().unwrap();
    assert_eq!(active_id, account.id);
}

pub async fn test_set_active_account_changes<B: StorageBackend>(backend: &B) {
    let account1 = make_account();
    let account2 = make_account();
    backend.upsert_account(&account1).await.unwrap();
    backend.upsert_account(&account2).await.unwrap();

    backend.set_active_account(&account1.id).await.unwrap();
    let active_id = backend.get_active_account_id().await.unwrap().unwrap();
    assert_eq!(active_id, account1.id);

    backend.set_active_account(&account2.id).await.unwrap();
    let active_id = backend.get_active_account_id().await.unwrap().unwrap();
    assert_eq!(active_id, account2.id);
}

pub async fn test_clear_active_account<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();
    backend.set_active_account(&account.id).await.unwrap();

    backend.clear_active_account().await.unwrap();

    let result = backend.get_active_account_id().await.unwrap();
    assert!(result.is_none());
}

pub async fn test_clear_active_account_when_none<B: StorageBackend>(backend: &B) {
    let result = backend.clear_active_account().await;
    assert!(result.is_ok());
}

pub async fn test_set_active_workspace<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    backend
        .set_active_workspace(&account.id, "workspace-123")
        .await
        .unwrap();

    let retrieved = backend.get_account(&account.id).await.unwrap().unwrap();
    assert_eq!(
        retrieved.active_workspace_id,
        Some("workspace-123".to_string())
    );
}

pub async fn test_set_active_workspace_nonexistent_account<B: StorageBackend>(backend: &B) {
    let result = backend
        .set_active_workspace("nonexistent", "workspace-123")
        .await;
    assert!(result.is_err());
}

pub async fn test_delete_account_with_providers<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let provider = make_provider(&account.id);
    backend
        .replace_providers(&account.id, vec![provider])
        .await
        .unwrap();

    backend.delete_providers(&account.id).await.unwrap();
    backend.delete_account(&account.id).await.unwrap();

    let result = backend.get_account(&account.id).await.unwrap();
    assert!(result.is_none());

    let providers = backend.list_providers(&account.id).await.unwrap();
    assert!(providers.is_empty());
}
