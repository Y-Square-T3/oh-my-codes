use crate::postgres::setup;
use crate::suites::providers as suite;

#[tokio::test]
async fn postgres_provider_tests() {
    let Some(backend) = setup().await else {
        return;
    };
    suite::test_list_providers_empty(&backend).await;
    suite::test_list_providers_nonexistent_account(&backend).await;
    suite::test_replace_providers(&backend).await;
    suite::test_replace_providers_json_roundtrip(&backend).await;
    suite::test_replace_providers_replaces_existing(&backend).await;
    suite::test_delete_providers(&backend).await;
    suite::test_delete_providers_nonexistent_account(&backend).await;
}
