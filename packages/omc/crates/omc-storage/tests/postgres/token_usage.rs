use crate::postgres::setup;
use crate::suites::token_usage as suite;

#[tokio::test]
async fn postgres_token_usage_tests() {
    let Some(backend) = setup().await else {
        return;
    };
    suite::test_upsert_usage(&backend).await;
    suite::test_upsert_usage_conflict_updates(&backend).await;
    suite::test_find_unpushed_empty(&backend).await;
    suite::test_find_unpushed(&backend).await;
    suite::test_find_unpushed_with_limit(&backend).await;
    suite::test_count_unpushed_empty(&backend).await;
    suite::test_count_unpushed(&backend).await;
    suite::test_mark_pushed(&backend).await;
    suite::test_mark_pushed_empty_ids(&backend).await;
    suite::test_list_recent_empty(&backend).await;
    suite::test_list_recent_with_limit_and_offset(&backend).await;
    suite::test_list_recent_filter_by_pushed(&backend).await;
    suite::test_count_all_empty(&backend).await;
    suite::test_count_all(&backend).await;
    suite::test_cleanup_old_pushed(&backend).await;
    suite::test_usage_summary_empty(&backend).await;
    suite::test_usage_summary(&backend).await;
}
