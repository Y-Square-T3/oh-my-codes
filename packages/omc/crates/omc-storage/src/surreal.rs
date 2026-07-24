use crate::message_store::MessageStore;
use async_trait::async_trait;
use omc_core::error::{OmcError, Result};
use omc_core::types::{Channel, Message};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, Mem, RocksDb};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealChannel {
    id: String,
    name: String,
    topic: Option<String>,
    created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SurrealMessage {
    id: String,
    channel_id: String,
    author_id: String,
    content: String,
    timestamp: i64,
    edited_at: Option<i64>,
    reply_to: Option<String>,
}

pub struct SurrealStorage {
    db: Arc<Surreal<Db>>,
}

impl SurrealStorage {
    pub async fn new_memory() -> Result<Self> {
        let db = Surreal::new::<Mem>(())
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create in-memory SurrealDB: {e}")))?;
        db.use_ns("omc")
            .use_db("main")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to set namespace: {e}")))?;
        Self::init_schema(&db).await?;
        Ok(Self { db: Arc::new(db) })
    }

    pub async fn new_rocksdb(path: &Path) -> Result<Self> {
        let db = Surreal::new::<RocksDb>(path)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create RocksDB SurrealDB: {e}")))?;
        db.use_ns("omc")
            .use_db("main")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to set namespace: {e}")))?;
        Self::init_schema(&db).await?;
        Ok(Self { db: Arc::new(db) })
    }

    async fn init_schema(db: &Surreal<Db>) -> Result<()> {
        let _result = db
            .query("DEFINE TABLE IF NOT EXISTS channel SCHEMALESS;")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to define channel table: {e}")))?;
        let _result = db
            .query("DEFINE TABLE IF NOT EXISTS message SCHEMALESS;")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to define message table: {e}")))?;
        Ok(())
    }
}

#[async_trait]
impl MessageStore for SurrealStorage {
    async fn create_channel(&self, name: &str) -> Result<Channel> {
        let now = chrono::Utc::now().timestamp();
        let channel = SurrealChannel {
            id: ulid::Ulid::new().to_string(),
            name: name.to_string(),
            topic: None,
            created_at: now,
        };
        let _: Option<SurrealChannel> = self
            .db
            .create("channel")
            .content(channel.clone())
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create channel: {e}")))?;
        Ok(Channel {
            id: channel.id,
            name: channel.name,
            topic: channel.topic,
            created_at: channel.created_at,
        })
    }

    async fn list_channels(&self) -> Result<Vec<Channel>> {
        let channels: Vec<SurrealChannel> = self
            .db
            .select("channel")
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to list channels: {e}")))?;
        Ok(channels
            .into_iter()
            .map(|c| Channel {
                id: c.id,
                name: c.name,
                topic: c.topic,
                created_at: c.created_at,
            })
            .collect())
    }

    async fn send_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<Message> {
        let now = chrono::Utc::now().timestamp();
        let msg = SurrealMessage {
            id: ulid::Ulid::new().to_string(),
            channel_id: channel_id.to_string(),
            author_id: author_id.to_string(),
            content: content.to_string(),
            timestamp: now,
            edited_at: None,
            reply_to: None,
        };
        let _: Option<SurrealMessage> = self
            .db
            .create("message")
            .content(msg.clone())
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to create message: {e}")))?;
        Ok(Message {
            id: msg.id,
            channel_id: msg.channel_id,
            author_id: msg.author_id,
            content: msg.content,
            timestamp: msg.timestamp,
            edited_at: msg.edited_at,
            reply_to: msg.reply_to,
        })
    }

    async fn get_messages(
        &self,
        channel_id: &str,
        limit: usize,
        before: Option<String>,
    ) -> Result<Vec<Message>> {
        let query = if let Some(before_id) = before {
            format!(
                "SELECT * FROM message WHERE channel_id = '{channel_id}' AND timestamp < (SELECT timestamp FROM message WHERE id = '{before_id}')[0].timestamp ORDER BY timestamp DESC LIMIT {limit};"
            )
        } else {
            format!(
                "SELECT * FROM message WHERE channel_id = '{channel_id}' ORDER BY timestamp DESC LIMIT {limit};"
            )
        };
        let mut result = self
            .db
            .query(&query)
            .await
            .map_err(|e| OmcError::Storage(format!("Failed to query messages: {e}")))?;
        let messages: Vec<SurrealMessage> = result
            .take(0)
            .map_err(|e| OmcError::Storage(format!("Failed to extract messages: {e}")))?;
        Ok(messages
            .into_iter()
            .map(|m| Message {
                id: m.id,
                channel_id: m.channel_id,
                author_id: m.author_id,
                content: m.content,
                timestamp: m.timestamp,
                edited_at: m.edited_at,
                reply_to: m.reply_to,
            })
            .collect())
    }
}
