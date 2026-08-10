use crate::postgres::setup;
use crate::suites::workspaces as suite;

#[tokio::test]
async fn postgres_workspace_tests() {
    let Some(backend) = setup().await else {
        return;
    };
    suite::test_list_workspaces_empty(&backend).await;
    suite::test_list_workspaces_nonexistent_account(&backend).await;
    suite::test_upsert_workspaces(&backend).await;
    suite::test_upsert_workspaces_updates_existing(&backend).await;
    suite::test_clear_workspaces(&backend).await;
    suite::test_clear_workspaces_nonexistent_account(&backend).await;
}
