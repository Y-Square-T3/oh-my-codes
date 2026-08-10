use crate::common::setup;
use crate::suites::token_usage as suite;

#[tokio::test]
async fn sqlite_upsert_usage() {
    let backend = setup().await;
    suite::test_upsert_usage(&backend).await;
}

#[tokio::test]
async fn sqlite_upsert_usage_conflict_updates() {
    let backend = setup().await;
    suite::test_upsert_usage_conflict_updates(&backend).await;
}

#[tokio::test]
async fn sqlite_find_unpushed_empty() {
    let backend = setup().await;
    suite::test_find_unpushed_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_find_unpushed() {
    let backend = setup().await;
    suite::test_find_unpushed(&backend).await;
}

#[tokio::test]
async fn sqlite_find_unpushed_with_limit() {
    let backend = setup().await;
    suite::test_find_unpushed_with_limit(&backend).await;
}

#[tokio::test]
async fn sqlite_count_unpushed_empty() {
    let backend = setup().await;
    suite::test_count_unpushed_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_count_unpushed() {
    let backend = setup().await;
    suite::test_count_unpushed(&backend).await;
}

#[tokio::test]
async fn sqlite_mark_pushed() {
    let backend = setup().await;
    suite::test_mark_pushed(&backend).await;
}

#[tokio::test]
async fn sqlite_mark_pushed_empty_ids() {
    let backend = setup().await;
    suite::test_mark_pushed_empty_ids(&backend).await;
}

#[tokio::test]
async fn sqlite_list_recent_empty() {
    let backend = setup().await;
    suite::test_list_recent_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_list_recent_with_limit_and_offset() {
    let backend = setup().await;
    suite::test_list_recent_with_limit_and_offset(&backend).await;
}

#[tokio::test]
async fn sqlite_list_recent_filter_by_pushed() {
    let backend = setup().await;
    suite::test_list_recent_filter_by_pushed(&backend).await;
}

#[tokio::test]
async fn sqlite_count_all_empty() {
    let backend = setup().await;
    suite::test_count_all_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_count_all() {
    let backend = setup().await;
    suite::test_count_all(&backend).await;
}

#[tokio::test]
async fn sqlite_cleanup_old_pushed() {
    let backend = setup().await;
    suite::test_cleanup_old_pushed(&backend).await;
}

#[tokio::test]
async fn sqlite_usage_summary_empty() {
    let backend = setup().await;
    suite::test_usage_summary_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_usage_summary() {
    let backend = setup().await;
    suite::test_usage_summary(&backend).await;
}
