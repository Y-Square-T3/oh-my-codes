use crate::common::setup;
use crate::suites::messaging as suite;

#[tokio::test]
async fn sqlite_create_channel() {
    let backend = setup().await;
    suite::test_create_channel(&backend).await;
}

#[tokio::test]
async fn sqlite_list_channels_empty() {
    let backend = setup().await;
    suite::test_list_channels_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_list_channels_multiple() {
    let backend = setup().await;
    suite::test_list_channels_multiple(&backend).await;
}

#[tokio::test]
async fn sqlite_send_message() {
    let backend = setup().await;
    suite::test_send_message(&backend).await;
}

#[tokio::test]
async fn sqlite_get_messages_empty() {
    let backend = setup().await;
    suite::test_get_messages_empty(&backend).await;
}

#[tokio::test]
async fn sqlite_get_messages_multiple() {
    let backend = setup().await;
    suite::test_get_messages_multiple(&backend).await;
}

#[tokio::test]
async fn sqlite_get_messages_with_limit() {
    let backend = setup().await;
    suite::test_get_messages_with_limit(&backend).await;
}

#[tokio::test]
async fn sqlite_get_messages_with_before_cursor() {
    let backend = setup().await;
    suite::test_get_messages_with_before_cursor(&backend).await;
}
