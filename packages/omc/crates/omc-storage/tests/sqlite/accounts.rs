use crate::common::setup;
use crate::suites::accounts as suite;

#[tokio::test]
async fn sqlite_get_account_empty() {
    let backend = setup().await;
    suite::test_get_account_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_upsert_and_get_account() {
    let backend = setup().await;
    suite::test_upsert_and_get_account(&backend).await;
}

#[tokio::test]
async fn sqlite_upsert_account_updates_existing() {
    let backend = setup().await;
    suite::test_upsert_account_updates_existing(&backend).await;
}

#[tokio::test]
async fn sqlite_list_accounts_empty() {
    let backend = setup().await;
    suite::test_list_accounts_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_list_accounts_multiple() {
    let backend = setup().await;
    suite::test_list_accounts_multiple(&backend).await;
}

#[tokio::test]
async fn sqlite_delete_account() {
    let backend = setup().await;
    suite::test_delete_account(&backend).await;
}

#[tokio::test]
async fn sqlite_delete_nonexistent_account() {
    let backend = setup().await;
    suite::test_delete_nonexistent_account(&backend).await;
}

#[tokio::test]
async fn sqlite_get_active_account_id_none() {
    let backend = setup().await;
    suite::test_get_active_account_id_none(&backend).await;
}

#[tokio::test]
async fn sqlite_set_and_get_active_account() {
    let backend = setup().await;
    suite::test_set_and_get_active_account(&backend).await;
}

#[tokio::test]
async fn sqlite_set_active_account_changes() {
    let backend = setup().await;
    suite::test_set_active_account_changes(&backend).await;
}

#[tokio::test]
async fn sqlite_clear_active_account() {
    let backend = setup().await;
    suite::test_clear_active_account(&backend).await;
}

#[tokio::test]
async fn sqlite_clear_active_account_when_none() {
    let backend = setup().await;
    suite::test_clear_active_account_when_none(&backend).await;
}

#[tokio::test]
async fn sqlite_set_active_workspace() {
    let backend = setup().await;
    suite::test_set_active_workspace(&backend).await;
}

#[tokio::test]
async fn sqlite_set_active_workspace_nonexistent_account() {
    let backend = setup().await;
    suite::test_set_active_workspace_nonexistent_account(&backend).await;
}

#[tokio::test]
async fn sqlite_delete_account_with_providers() {
    let backend = setup().await;
    suite::test_delete_account_with_providers(&backend).await;
}
