use omc_storage::StorageBackend;

pub async fn test_create_channel<B: StorageBackend>(backend: &B) {
    let channel = backend.create_channel("test-channel").await.unwrap();
    assert_eq!(channel.name, "test-channel");
    assert!(channel.topic.is_none());
    assert!(!channel.id.is_empty());
    assert!(channel.created_at > 0);
}

pub async fn test_list_channels_empty<B: StorageBackend>(backend: &B) {
    let channels = backend.list_channels().await.unwrap();
    assert!(channels.is_empty());
}

pub async fn test_list_channels_multiple<B: StorageBackend>(backend: &B) {
    backend.create_channel("channel-1").await.unwrap();
    backend.create_channel("channel-2").await.unwrap();

    let channels = backend.list_channels().await.unwrap();
    assert_eq!(channels.len(), 2);
}

pub async fn test_send_message<B: StorageBackend>(backend: &B) {
    let channel = backend.create_channel("test-channel").await.unwrap();

    let message = backend
        .send_message(&channel.id, "author-1", "Hello, world!")
        .await
        .unwrap();

    assert_eq!(message.channel_id, channel.id);
    assert_eq!(message.author_id, "author-1");
    assert_eq!(message.content, "Hello, world!");
    assert!(message.edited_at.is_none());
    assert!(message.reply_to.is_none());
}

pub async fn test_get_messages_empty<B: StorageBackend>(backend: &B) {
    let channel = backend.create_channel("test-channel").await.unwrap();

    let messages = backend.get_messages(&channel.id, 10, None).await.unwrap();
    assert!(messages.is_empty());
}

pub async fn test_get_messages_multiple<B: StorageBackend>(backend: &B) {
    let channel = backend.create_channel("test-channel").await.unwrap();

    backend
        .send_message(&channel.id, "author-1", "Message 1")
        .await
        .unwrap();
    backend
        .send_message(&channel.id, "author-1", "Message 2")
        .await
        .unwrap();
    backend
        .send_message(&channel.id, "author-1", "Message 3")
        .await
        .unwrap();

    let messages = backend.get_messages(&channel.id, 10, None).await.unwrap();
    assert_eq!(messages.len(), 3);
}

pub async fn test_get_messages_with_limit<B: StorageBackend>(backend: &B) {
    let channel = backend.create_channel("test-channel").await.unwrap();

    backend
        .send_message(&channel.id, "author-1", "Message 1")
        .await
        .unwrap();
    backend
        .send_message(&channel.id, "author-1", "Message 2")
        .await
        .unwrap();
    backend
        .send_message(&channel.id, "author-1", "Message 3")
        .await
        .unwrap();

    let messages = backend.get_messages(&channel.id, 2, None).await.unwrap();
    assert_eq!(messages.len(), 2);
}

pub async fn test_get_messages_with_before_cursor<B: StorageBackend>(backend: &B) {
    let channel = backend.create_channel("test-channel").await.unwrap();

    let msg1 = backend
        .send_message(&channel.id, "author-1", "Message 1")
        .await
        .unwrap();

    // Small delay to ensure distinct millisecond timestamps
    tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;

    let msg2 = backend
        .send_message(&channel.id, "author-1", "Message 2")
        .await
        .unwrap();

    tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;

    let msg3 = backend
        .send_message(&channel.id, "author-1", "Message 3")
        .await
        .unwrap();

    let messages = backend
        .get_messages(&channel.id, 10, Some(msg3.id.clone()))
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);

    let messages = backend
        .get_messages(&channel.id, 10, Some(msg2.id.clone()))
        .await
        .unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, msg1.id);
}
