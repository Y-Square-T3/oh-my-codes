use crate::common::setup;
use crate::suites::token_cost as suite;

#[tokio::test]
async fn sqlite_upsert_token_cost() {
    let backend = setup().await;
    suite::test_upsert_token_cost(&backend).await;
}

#[tokio::test]
async fn sqlite_get_token_cost_not_found() {
    let backend = setup().await;
    suite::test_get_token_cost_not_found(&backend).await;
}

#[tokio::test]
async fn sqlite_upsert_token_cost_immutable() {
    let backend = setup().await;
    suite::test_upsert_token_cost_immutable(&backend).await;
}

#[tokio::test]
async fn sqlite_upsert_token_cost_requires_usage() {
    let backend = setup().await;
    suite::test_upsert_token_cost_requires_usage(&backend).await;
}
