use crate::common::setup;
use crate::suites::providers as suite;

#[tokio::test]
async fn sqlite_list_providers_empty() {
    let backend = setup().await;
    suite::test_list_providers_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_list_providers_nonexistent_account() {
    let backend = setup().await;
    suite::test_list_providers_nonexistent_account(&backend).await;
}

#[tokio::test]
async fn sqlite_replace_providers() {
    let backend = setup().await;
    suite::test_replace_providers(&backend).await;
}

#[tokio::test]
async fn sqlite_replace_providers_json_roundtrip() {
    let backend = setup().await;
    suite::test_replace_providers_json_roundtrip(&backend).await;
}

#[tokio::test]
async fn sqlite_replace_providers_replaces_existing() {
    let backend = setup().await;
    suite::test_replace_providers_replaces_existing(&backend).await;
}

#[tokio::test]
async fn sqlite_delete_providers() {
    let backend = setup().await;
    suite::test_delete_providers(&backend).await;
}

#[tokio::test]
async fn sqlite_delete_providers_nonexistent_account() {
    let backend = setup().await;
    suite::test_delete_providers_nonexistent_account(&backend).await;
}
