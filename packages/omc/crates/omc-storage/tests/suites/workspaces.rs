use omc_storage::StorageBackend;

use crate::common::builders::{make_account, make_workspace};

pub async fn test_list_workspaces_empty<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let workspaces = backend.list_workspaces(&account.id).await.unwrap();
    assert!(workspaces.is_empty());
}

pub async fn test_list_workspaces_nonexistent_account<B: StorageBackend>(backend: &B) {
    let workspaces = backend.list_workspaces("nonexistent").await.unwrap();
    assert!(workspaces.is_empty());
}

pub async fn test_upsert_workspaces<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let workspace1 = make_workspace(&account.id);
    let workspace2 = make_workspace(&account.id);
    backend
        .upsert_workspaces(&[workspace1.clone(), workspace2.clone()])
        .await
        .unwrap();

    let workspaces = backend.list_workspaces(&account.id).await.unwrap();
    assert_eq!(workspaces.len(), 2);

    let ids: Vec<&str> = workspaces.iter().map(|w| w.id.as_str()).collect();
    assert!(ids.contains(&workspace1.id.as_str()));
    assert!(ids.contains(&workspace2.id.as_str()));
}

pub async fn test_upsert_workspaces_updates_existing<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let mut workspace = make_workspace(&account.id);
    backend.upsert_workspaces(&[workspace.clone()]).await.unwrap();

    workspace.name = "updated-name".to_string();
    workspace.is_admin = true;
    backend.upsert_workspaces(&[workspace.clone()]).await.unwrap();

    let workspaces = backend.list_workspaces(&account.id).await.unwrap();
    assert_eq!(workspaces.len(), 1);
    assert_eq!(workspaces[0].name, "updated-name");
    assert!(workspaces[0].is_admin);
}

pub async fn test_clear_workspaces<B: StorageBackend>(backend: &B) {
    let account = make_account();
    backend.upsert_account(&account).await.unwrap();

    let workspace1 = make_workspace(&account.id);
    let workspace2 = make_workspace(&account.id);
    backend
        .upsert_workspaces(&[workspace1, workspace2])
        .await
        .unwrap();

    backend.clear_workspaces(&account.id).await.unwrap();

    let workspaces = backend.list_workspaces(&account.id).await.unwrap();
    assert!(workspaces.is_empty());
}

pub async fn test_clear_workspaces_nonexistent_account<B: StorageBackend>(backend: &B) {
    let result = backend.clear_workspaces("nonexistent").await;
    assert!(result.is_ok());
}
