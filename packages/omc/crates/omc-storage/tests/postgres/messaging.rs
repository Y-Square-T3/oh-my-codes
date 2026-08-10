use crate::postgres::setup;
use crate::suites::messaging as suite;

#[tokio::test]
async fn postgres_messaging_tests() {
    let Some(backend) = setup().await else {
        return;
    };
    suite::test_create_channel(&backend).await;
    suite::test_list_channels_empty(&backend).await;
    suite::test_list_channels_multiple(&backend).await;
    suite::test_send_message(&backend).await;
    suite::test_get_messages_empty(&backend).await;
    suite::test_get_messages_multiple(&backend).await;
    suite::test_get_messages_with_limit(&backend).await;
    suite::test_get_messages_with_before_cursor(&backend).await;
}
