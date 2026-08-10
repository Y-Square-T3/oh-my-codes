#![allow(dead_code)]

use omc_core::account::{Account, Workspace};
use omc_core::model::{Model, ModelLimit, Provider};
use omc_core::token_usage::{TokenUsage, generate_id};
use omc_core::types::{Channel, Message};
use ulid::Ulid;

pub fn make_account() -> Account {
    let id = Ulid::new().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    Account {
        id: id.clone(),
        email: format!("test-{id}@example.com"),
        url: "https://api.example.com".to_string(),
        access_token: format!("access-{id}"),
        refresh_token: format!("refresh-{id}"),
        token_expiry: now + 3_600_000,
        active_workspace_id: None,
    }
}

pub fn make_workspace(account_id: &str) -> Workspace {
    let id = Ulid::new().to_string();
    Workspace {
        id: id.clone(),
        account_id: account_id.to_string(),
        name: format!("workspace-{}", &id[..8]),
        is_admin: false,
    }
}

pub fn make_channel() -> Channel {
    let id = Ulid::new().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    Channel {
        id: id.clone(),
        name: format!("channel-{}", &id[..8]),
        topic: None,
        created_at: now,
    }
}

pub fn make_message(channel_id: &str, author_id: &str) -> Message {
    let id = Ulid::new().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    Message {
        id: id.clone(),
        channel_id: channel_id.to_string(),
        author_id: author_id.to_string(),
        content: format!("message-{}", &id[..8]),
        timestamp: now,
        edited_at: None,
        reply_to: None,
    }
}

pub fn make_provider(account_id: &str) -> Provider {
    let id = Ulid::new().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    Provider {
        id: id.clone(),
        name: format!("provider-{}", &id[..8]),
        env: vec!["API_KEY".to_string()],
        api: Some("https://api.provider.com".to_string()),
        npm: None,
        doc: Some("https://docs.provider.com".to_string()),
        models: vec![make_model()],
        account_id: account_id.to_string(),
        last_fetched_at: now,
    }
}

pub fn make_model() -> Model {
    let id = Ulid::new().to_string();
    Model {
        id: id.clone(),
        name: format!("model-{}", &id[..8]),
        family: None,
        release_date: None,
        last_updated: None,
        attachment: None,
        reasoning: None,
        temperature: None,
        tool_call: None,
        interleaved: None,
        cost: None,
        limit: ModelLimit {
            context: 128_000,
            input: None,
            output: 4_096,
        },
        modalities: None,
        experimental: None,
        structured_output: None,
        knowledge: None,
        open_weights: None,
        provider: None,
        status: None,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn make_usage(
    agent: &str,
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    reasoning_tokens: i64,
    pushed: bool,
) -> TokenUsage {
    let message_id = format!("msg-{}", Ulid::new());
    let id = generate_id(agent, &message_id);
    let now = chrono::Utc::now().timestamp_millis();
    let total_tokens = input_tokens + output_tokens + reasoning_tokens;
    TokenUsage {
        id,
        workspace_id: None,
        session_id: format!("session-{message_id}"),
        agent: agent.to_string(),
        model: model.to_string(),
        metadata: Some(format!(r#"{{"messageId":"{message_id}"}}"#)),
        input_tokens,
        output_tokens,
        reasoning_tokens,
        cache_read_tokens: 0,
        cache_write_tokens: 0,
        audio_input_tokens: 0,
        video_input_tokens: 0,
        image_input_tokens: 0,
        total_tokens,
        pushed,
        recorded_at: now,
        created_at: now,
    }
}
