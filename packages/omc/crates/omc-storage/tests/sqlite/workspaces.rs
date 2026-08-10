use crate::common::setup;
use crate::suites::workspaces as suite;

#[tokio::test]
async fn sqlite_list_workspaces_empty() {
    let backend = setup().await;
    suite::test_list_workspaces_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_list_workspaces_nonexistent_account() {
    let backend = setup().await;
    suite::test_list_workspaces_nonexistent_account(&backend).await;
}

#[tokio::test]
async fn sqlite_upsert_workspaces() {
    let backend = setup().await;
    suite::test_upsert_workspaces(&backend).await;
}

#[tokio::test]
async fn sqlite_upsert_workspaces_updates_existing() {
    let backend = setup().await;
    suite::test_upsert_workspaces_updates_existing(&backend).await;
}

#[tokio::test]
async fn sqlite_clear_workspaces() {
    let backend = setup().await;
    suite::test_clear_workspaces(&backend).await;
}

#[tokio::test]
async fn sqlite_clear_workspaces_nonexistent_account() {
    let backend = setup().await;
    suite::test_clear_workspaces_nonexistent_account(&backend).await;
}
