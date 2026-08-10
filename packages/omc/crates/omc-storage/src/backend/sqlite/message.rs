use omc_core::error::{OmcError, Result};
use omc_core::types::{Channel, Message};
use sqlx::FromRow;
use sqlx::SqlitePool;

#[derive(Debug, Clone, FromRow)]
struct ChannelRow {
    id: String,
    name: String,
    topic: Option<String>,
    created_at: i64,
}

#[derive(Debug, Clone, FromRow)]
struct MessageRow {
    id: String,
    channel_id: String,
    author_id: String,
    content: String,
    timestamp: i64,
    edited_at: Option<i64>,
    reply_to: Option<String>,
}

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("SQLite error: {e}"))
}

pub(crate) async fn create_channel(pool: &SqlitePool, name: &str) -> Result<Channel> {
    let now = chrono::Utc::now().timestamp_millis();
    let id = ulid::Ulid::new().to_string();
    sqlx::query("INSERT INTO channel (id, name, topic, created_at) VALUES (?, ?, NULL, ?)")
        .bind(&id)
        .bind(name)
        .bind(now)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(Channel {
        id,
        name: name.to_string(),
        topic: None,
        created_at: now,
    })
}

pub(crate) async fn list_channels(pool: &SqlitePool) -> Result<Vec<Channel>> {
    let rows: Vec<ChannelRow> =
        sqlx::query_as("SELECT id, name, topic, created_at FROM channel ORDER BY created_at")
            .fetch_all(pool)
            .await
            .map_err(map_err)?;
    Ok(rows
        .into_iter()
        .map(|r| Channel {
            id: r.id,
            name: r.name,
            topic: r.topic,
            created_at: r.created_at,
        })
        .collect())
}

pub(crate) async fn send_message(
    pool: &SqlitePool,
    channel_id: &str,
    author_id: &str,
    content: &str,
) -> Result<Message> {
    let now = chrono::Utc::now().timestamp_millis();
    let id = ulid::Ulid::new().to_string();
    sqlx::query(
        "INSERT INTO message (id, channel_id, author_id, content, timestamp, edited_at, reply_to) VALUES (?, ?, ?, ?, ?, NULL, NULL)",
    )
    .bind(&id)
    .bind(channel_id)
    .bind(author_id)
    .bind(content)
    .bind(now)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(Message {
        id,
        channel_id: channel_id.to_string(),
        author_id: author_id.to_string(),
        content: content.to_string(),
        timestamp: now,
        edited_at: None,
        reply_to: None,
    })
}

pub(crate) async fn get_messages(
    pool: &SqlitePool,
    channel_id: &str,
    limit: usize,
    before: Option<String>,
) -> Result<Vec<Message>> {
    let rows: Vec<MessageRow> = if let Some(before_id) = before {
        sqlx::query_as(
            "SELECT id, channel_id, author_id, content, timestamp, edited_at, reply_to
             FROM message
             WHERE channel_id = ? AND timestamp < (SELECT timestamp FROM message WHERE id = ?)
             ORDER BY timestamp DESC
             LIMIT ?",
        )
        .bind(channel_id)
        .bind(&before_id)
        .bind(limit as i64)
        .fetch_all(pool)
        .await
        .map_err(map_err)?
    } else {
        sqlx::query_as(
            "SELECT id, channel_id, author_id, content, timestamp, edited_at, reply_to
             FROM message
             WHERE channel_id = ?
             ORDER BY timestamp DESC
             LIMIT ?",
        )
        .bind(channel_id)
        .bind(limit as i64)
        .fetch_all(pool)
        .await
        .map_err(map_err)?
    };
    Ok(rows
        .into_iter()
        .map(|r| Message {
            id: r.id,
            channel_id: r.channel_id,
            author_id: r.author_id,
            content: r.content,
            timestamp: r.timestamp,
            edited_at: r.edited_at,
            reply_to: r.reply_to,
        })
        .collect())
}
