use omc_storage::StorageBackend;

use crate::common::builders::{make_account, make_provider};

pub async fn test_list_providers_empty<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let providers = backend.list_providers(&account.id).await.unwrap();
    assert!(providers.is_empty());
}

pub async fn test_list_providers_nonexistent_account<B: StorageBackend>(backend: &B) {
    let providers = backend.list_providers("nonexistent").await.unwrap();
    assert!(providers.is_empty());
}

pub async fn test_replace_providers<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let provider1 = make_provider(&account.id);
    let provider2 = make_provider(&account.id);
    backend
        .replace_providers(&account.id, vec![provider1.clone(), provider2.clone()])
        .await
        .unwrap();

    let providers = backend.list_providers(&account.id).await.unwrap();
    assert_eq!(providers.len(), 2);

    let ids: Vec<&str> = providers.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&provider1.id.as_str()));
    assert!(ids.contains(&provider2.id.as_str()));
}

pub async fn test_replace_providers_json_roundtrip<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let mut provider = make_provider(&account.id);
    provider.env = vec!["API_KEY".to_string(), "SECRET".to_string()];
    provider.models = vec![crate::common::builders::make_model()];

    backend
        .replace_providers(&account.id, vec![provider.clone()])
        .await
        .unwrap();

    let providers = backend.list_providers(&account.id).await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].env, provider.env);
    assert_eq!(providers[0].models.len(), 1);
    assert_eq!(providers[0].models[0].id, provider.models[0].id);
}

pub async fn test_replace_providers_replaces_existing<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let provider1 = make_provider(&account.id);
    backend
        .replace_providers(&account.id, vec![provider1])
        .await
        .unwrap();

    let provider2 = make_provider(&account.id);
    backend
        .replace_providers(&account.id, vec![provider2.clone()])
        .await
        .unwrap();

    let providers = backend.list_providers(&account.id).await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].id, provider2.id);
}

pub async fn test_delete_providers<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let provider = make_provider(&account.id);
    backend
        .replace_providers(&account.id, vec![provider])
        .await
        .unwrap();

    backend.delete_providers(&account.id).await.unwrap();

    let providers = backend.list_providers(&account.id).await.unwrap();
    assert!(providers.is_empty());
}

pub async fn test_delete_providers_nonexistent_account<B: StorageBackend>(backend: &B) {
    let result = backend.delete_providers("nonexistent").await;
    assert!(result.is_ok());
}
