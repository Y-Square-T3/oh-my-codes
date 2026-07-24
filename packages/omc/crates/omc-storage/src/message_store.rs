use async_trait::async_trait;
use omc_core::error::Result;
use omc_core::types::{Channel, Message};

#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn create_channel(&self, name: &str) -> Result<Channel>;
    async fn list_channels(&self) -> Result<Vec<Channel>>;
    async fn send_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<Message>;
    async fn get_messages(
        &self,
        channel_id: &str,
        limit: usize,
        before: Option<String>,
    ) -> Result<Vec<Message>>;
}
