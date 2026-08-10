use crate::postgres::setup;
use crate::suites::accounts as suite;

#[tokio::test]
async fn postgres_account_tests() {
    let Some(backend) = setup().await else {
        return;
    };
    suite::test_get_account_empty(&backend).await;
    suite::test_upsert_and_get_account(&backend).await;
    suite::test_upsert_account_updates_existing(&backend).await;
    suite::test_list_accounts_empty(&backend).await;
    suite::test_list_accounts_multiple(&backend).await;
    suite::test_delete_account(&backend).await;
    suite::test_delete_nonexistent_account(&backend).await;
    suite::test_get_active_account_id_none(&backend).await;
    suite::test_set_and_get_active_account(&backend).await;
    suite::test_set_active_account_changes(&backend).await;
    suite::test_clear_active_account(&backend).await;
    suite::test_clear_active_account_when_none(&backend).await;
    suite::test_set_active_workspace(&backend).await;
    suite::test_set_active_workspace_nonexistent_account(&backend).await;
}
