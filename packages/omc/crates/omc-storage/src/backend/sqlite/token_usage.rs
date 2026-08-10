use omc_core::error::{OmcError, Result};
use omc_core::token_usage::{
    DailyUsage, HeadlineStats, TokenUsage, TokenUsageOverview, UsageGroup, UsageSummary,
};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

#[derive(Debug, Clone, FromRow)]
struct TokenUsageRow {
    id: String,
    client: String,
    session_id: String,
    message_id: String,
    agent: Option<String>,
    provider_id: String,
    model_id: String,
    input_tokens: i64,
    output_tokens: i64,
    reasoning_tokens: i64,
    cache_read_tokens: i64,
    cache_write_tokens: i64,
    pushed: i32,
    recorded_at: i64,
    created_at: i64,
}

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("SQLite error: {e}"))
}

fn row_to_token_usage(r: TokenUsageRow) -> TokenUsage {
    TokenUsage {
        id: r.id,
        client: r.client,
        session_id: r.session_id,
        message_id: r.message_id,
        agent: r.agent,
        provider_id: r.provider_id,
        model_id: r.model_id,
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        reasoning_tokens: r.reasoning_tokens,
        cache_read_tokens: r.cache_read_tokens,
        cache_write_tokens: r.cache_write_tokens,
        pushed: r.pushed != 0,
        recorded_at: r.recorded_at,
        created_at: r.created_at,
    }
}

fn cutoff_clause(cutoff: Option<i64>) -> &'static str {
    if cutoff.is_some() {
        " WHERE recorded_at >= ?"
    } else {
        ""
    }
}

async fn fetch_with_optional_cutoff(
    pool: &SqlitePool,
    sql: &str,
    cutoff: Option<i64>,
) -> std::result::Result<Vec<SqliteRow>, sqlx::Error> {
    if let Some(c) = cutoff {
        sqlx::query(sql).bind(c).fetch_all(pool).await
    } else {
        sqlx::query(sql).fetch_all(pool).await
    }
}

pub(crate) async fn upsert_usage(pool: &SqlitePool, usage: &TokenUsage) -> Result<()> {
    let pushed = if usage.pushed { 1 } else { 0 };
    sqlx::query(
        "INSERT INTO token_usage (id, client, session_id, message_id, agent, provider_id, model_id, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, pushed, recorded_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(message_id) DO UPDATE SET
            client = excluded.client,
            session_id = excluded.session_id,
            agent = excluded.agent,
            provider_id = excluded.provider_id,
            model_id = excluded.model_id,
            input_tokens = excluded.input_tokens,
            output_tokens = excluded.output_tokens,
            reasoning_tokens = excluded.reasoning_tokens,
            cache_read_tokens = excluded.cache_read_tokens,
            cache_write_tokens = excluded.cache_write_tokens,
            pushed = excluded.pushed,
            recorded_at = excluded.recorded_at,
            created_at = excluded.created_at",
    )
    .bind(&usage.id)
    .bind(&usage.client)
    .bind(&usage.session_id)
    .bind(&usage.message_id)
    .bind(&usage.agent)
    .bind(&usage.provider_id)
    .bind(&usage.model_id)
    .bind(usage.input_tokens)
    .bind(usage.output_tokens)
    .bind(usage.reasoning_tokens)
    .bind(usage.cache_read_tokens)
    .bind(usage.cache_write_tokens)
    .bind(pushed)
    .bind(usage.recorded_at)
    .bind(usage.created_at)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn find_unpushed(pool: &SqlitePool, limit: usize) -> Result<Vec<TokenUsage>> {
    let rows: Vec<TokenUsageRow> = sqlx::query_as(
        "SELECT id, client, session_id, message_id, agent, provider_id, model_id, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, pushed, recorded_at, created_at
         FROM token_usage WHERE pushed = 0 ORDER BY recorded_at ASC LIMIT ?",
    )
    .bind(limit as i64)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    Ok(rows.into_iter().map(row_to_token_usage).collect())
}

pub(crate) async fn count_unpushed(pool: &SqlitePool) -> Result<usize> {
    let row: SqliteRow = sqlx::query("SELECT COUNT(*) as count FROM token_usage WHERE pushed = 0")
        .fetch_one(pool)
        .await
        .map_err(map_err)?;
    let count: i64 = row.get("count");
    Ok(count as usize)
}

pub(crate) async fn mark_pushed(pool: &SqlitePool, ids: &[String]) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
    let query = format!(
        "UPDATE token_usage SET pushed = 1 WHERE id IN ({})",
        placeholders.join(",")
    );
    let mut q = sqlx::query(&query);
    for id in ids {
        q = q.bind(id);
    }
    q.execute(pool).await.map_err(map_err)?;
    Ok(())
}

pub(crate) async fn list_recent(
    pool: &SqlitePool,
    limit: usize,
    offset: usize,
    pushed: Option<bool>,
) -> Result<Vec<TokenUsage>> {
    let where_clause = match pushed {
        Some(true) => " WHERE pushed = 1",
        Some(false) => " WHERE pushed = 0",
        None => "",
    };
    let sql = format!(
        "SELECT id, client, session_id, message_id, agent, provider_id, model_id, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, pushed, recorded_at, created_at
         FROM token_usage{where_clause} ORDER BY recorded_at DESC LIMIT ? OFFSET ?",
    );
    let rows: Vec<TokenUsageRow> = sqlx::query_as(&sql)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await
        .map_err(map_err)?;
    Ok(rows.into_iter().map(row_to_token_usage).collect())
}

pub(crate) async fn count_all(pool: &SqlitePool, pushed: Option<bool>) -> Result<usize> {
    let sql = match pushed {
        Some(true) => "SELECT COUNT(*) as count FROM token_usage WHERE pushed = 1",
        Some(false) => "SELECT COUNT(*) as count FROM token_usage WHERE pushed = 0",
        None => "SELECT COUNT(*) as count FROM token_usage",
    };
    let row: SqliteRow = sqlx::query(sql).fetch_one(pool).await.map_err(map_err)?;
    let count: i64 = row.get("count");
    Ok(count as usize)
}

pub(crate) async fn cleanup_old_pushed(pool: &SqlitePool, retention_days: i64) -> Result<usize> {
    let cutoff = chrono::Utc::now().timestamp_millis() - (retention_days * 86_400_000);
    let result = sqlx::query("DELETE FROM token_usage WHERE pushed = 1 AND recorded_at < ?")
        .bind(cutoff)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(result.rows_affected() as usize)
}

pub(crate) async fn usage_summary(
    pool: &SqlitePool,
    days: Option<i64>,
) -> Result<Vec<UsageSummary>> {
    let cutoff = days.map(|d| chrono::Utc::now().timestamp_millis() - (d * 86_400_000));
    let sql = format!(
        "SELECT provider_id, model_id,
            SUM(input_tokens) as total_input,
            SUM(output_tokens) as total_output,
            SUM(reasoning_tokens) as total_reasoning,
            SUM(cache_read_tokens) as total_cache_read,
            SUM(cache_write_tokens) as total_cache_write,
            COUNT(*) as request_count
         FROM token_usage{}
         GROUP BY provider_id, model_id",
        cutoff_clause(cutoff)
    );
    let rows = fetch_with_optional_cutoff(pool, &sql, cutoff)
        .await
        .map_err(map_err)?;
    Ok(rows
        .into_iter()
        .map(|r| UsageSummary {
            provider_id: r.get("provider_id"),
            model_id: r.get("model_id"),
            total_input: r.get("total_input"),
            total_output: r.get("total_output"),
            total_reasoning: r.get("total_reasoning"),
            total_cache_read: r.get("total_cache_read"),
            total_cache_write: r.get("total_cache_write"),
            request_count: r.get("request_count"),
        })
        .collect())
}

pub(crate) async fn usage_overview(
    pool: &SqlitePool,
    days: Option<i64>,
) -> Result<TokenUsageOverview> {
    let cutoff = days.map(|d| chrono::Utc::now().timestamp_millis() - (d * 86_400_000));
    let seven_days_ago = chrono::Utc::now().timestamp_millis() - (7 * 86_400_000);

    let headline = headline_stats(pool, cutoff).await?;
    let top_models = top_models(pool, cutoff).await?;
    let top_agents = top_agents(pool, cutoff).await?;
    let top_clients = top_clients(pool, cutoff).await?;
    let trend = daily_trend(pool, seven_days_ago).await?;

    Ok(TokenUsageOverview {
        headline,
        top_models,
        top_agents,
        top_clients,
        trend,
    })
}

async fn headline_stats(pool: &SqlitePool, cutoff: Option<i64>) -> Result<HeadlineStats> {
    let sql = format!(
        "SELECT
            COUNT(*) as requests,
            SUM(input_tokens) as input_tokens,
            SUM(output_tokens) as output_tokens,
            SUM(reasoning_tokens) as reasoning_tokens,
            SUM(cache_read_tokens) as cache_read_tokens,
            SUM(cache_write_tokens) as cache_write_tokens
         FROM token_usage{}",
        cutoff_clause(cutoff)
    );
    let row: SqliteRow = fetch_with_optional_cutoff(pool, &sql, cutoff)
        .await
        .map_err(map_err)?
        .into_iter()
        .next()
        .ok_or_else(|| OmcError::Storage("No headline stats row".into()))?;

    let unpushed_row: SqliteRow = sqlx::query(
        "SELECT COUNT(*) as count, SUM(input_tokens + output_tokens + reasoning_tokens) as tokens FROM token_usage WHERE pushed = 0",
    )
    .fetch_one(pool)
    .await
    .map_err(map_err)?;

    Ok(HeadlineStats {
        requests: row.get("requests"),
        input_tokens: row.get("input_tokens"),
        output_tokens: row.get("output_tokens"),
        reasoning_tokens: row.get("reasoning_tokens"),
        cache_read_tokens: row.get("cache_read_tokens"),
        cache_write_tokens: row.get("cache_write_tokens"),
        unpushed_records: unpushed_row.get::<i64, _>("count") as usize,
        unpushed_tokens: unpushed_row.get::<Option<i64>, _>("tokens").unwrap_or(0),
    })
}

async fn top_models(pool: &SqlitePool, cutoff: Option<i64>) -> Result<Vec<UsageSummary>> {
    let sql = format!(
        "SELECT provider_id, model_id,
            SUM(input_tokens) as total_input,
            SUM(output_tokens) as total_output,
            SUM(reasoning_tokens) as total_reasoning,
            SUM(cache_read_tokens) as total_cache_read,
            SUM(cache_write_tokens) as total_cache_write,
            COUNT(*) as request_count
         FROM token_usage{}
         GROUP BY provider_id, model_id
         ORDER BY (SUM(input_tokens) + SUM(output_tokens) + SUM(reasoning_tokens)) DESC
         LIMIT 3",
        cutoff_clause(cutoff)
    );
    let rows = fetch_with_optional_cutoff(pool, &sql, cutoff)
        .await
        .map_err(map_err)?;
    Ok(rows
        .into_iter()
        .map(|r| UsageSummary {
            provider_id: r.get("provider_id"),
            model_id: r.get("model_id"),
            total_input: r.get("total_input"),
            total_output: r.get("total_output"),
            total_reasoning: r.get("total_reasoning"),
            total_cache_read: r.get("total_cache_read"),
            total_cache_write: r.get("total_cache_write"),
            request_count: r.get("request_count"),
        })
        .collect())
}

async fn top_agents(pool: &SqlitePool, cutoff: Option<i64>) -> Result<Vec<UsageGroup>> {
    let sql = format!(
        "SELECT COALESCE(agent, 'unknown') as label,
            SUM(input_tokens) as total_input,
            SUM(output_tokens) as total_output,
            SUM(reasoning_tokens) as total_reasoning,
            SUM(cache_read_tokens) as total_cache_read,
            SUM(cache_write_tokens) as total_cache_write,
            COUNT(*) as request_count
         FROM token_usage{}
         GROUP BY label
         ORDER BY (SUM(input_tokens) + SUM(output_tokens) + SUM(reasoning_tokens)) DESC
         LIMIT 3",
        cutoff_clause(cutoff)
    );
    let rows = fetch_with_optional_cutoff(pool, &sql, cutoff)
        .await
        .map_err(map_err)?;
    Ok(rows
        .into_iter()
        .map(|r| UsageGroup {
            label: r.get("label"),
            total_input: r.get("total_input"),
            total_output: r.get("total_output"),
            total_reasoning: r.get("total_reasoning"),
            total_cache_read: r.get("total_cache_read"),
            total_cache_write: r.get("total_cache_write"),
            request_count: r.get("request_count"),
        })
        .collect())
}

async fn top_clients(pool: &SqlitePool, cutoff: Option<i64>) -> Result<Vec<UsageGroup>> {
    let sql = format!(
        "SELECT client as label,
            SUM(input_tokens) as total_input,
            SUM(output_tokens) as total_output,
            SUM(reasoning_tokens) as total_reasoning,
            SUM(cache_read_tokens) as total_cache_read,
            SUM(cache_write_tokens) as total_cache_write,
            COUNT(*) as request_count
         FROM token_usage{}
         GROUP BY client
         ORDER BY (SUM(input_tokens) + SUM(output_tokens) + SUM(reasoning_tokens)) DESC
         LIMIT 3",
        cutoff_clause(cutoff)
    );
    let rows = fetch_with_optional_cutoff(pool, &sql, cutoff)
        .await
        .map_err(map_err)?;
    Ok(rows
        .into_iter()
        .map(|r| UsageGroup {
            label: r.get("label"),
            total_input: r.get("total_input"),
            total_output: r.get("total_output"),
            total_reasoning: r.get("total_reasoning"),
            total_cache_read: r.get("total_cache_read"),
            total_cache_write: r.get("total_cache_write"),
            request_count: r.get("request_count"),
        })
        .collect())
}

async fn daily_trend(pool: &SqlitePool, cutoff: i64) -> Result<Vec<DailyUsage>> {
    let rows: Vec<SqliteRow> = sqlx::query(
        "SELECT
            DATE(recorded_at / 1000, 'unixepoch') as date,
            COUNT(*) as requests,
            SUM(input_tokens + output_tokens + reasoning_tokens) as total_tokens
         FROM token_usage
         WHERE recorded_at >= ?
         GROUP BY date
         ORDER BY date",
    )
    .bind(cutoff)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;

    Ok(rows
        .into_iter()
        .map(|r| DailyUsage {
            date: r.get("date"),
            requests: r.get("requests"),
            total_tokens: r.get("total_tokens"),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_to_token_usage_maps_all_fields() {
        let row = TokenUsageRow {
            id: "test-id".to_string(),
            client: "vscode".to_string(),
            session_id: "session-123".to_string(),
            message_id: "msg-456".to_string(),
            agent: Some("agent-a".to_string()),
            provider_id: "openai".to_string(),
            model_id: "gpt-4o".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            reasoning_tokens: 100,
            cache_read_tokens: 50,
            cache_write_tokens: 25,
            pushed: 1,
            recorded_at: 1234567890,
            created_at: 1234567800,
        };

        let usage = row_to_token_usage(row);

        assert_eq!(usage.id, "test-id");
        assert_eq!(usage.client, "vscode");
        assert_eq!(usage.session_id, "session-123");
        assert_eq!(usage.message_id, "msg-456");
        assert_eq!(usage.agent, Some("agent-a".to_string()));
        assert_eq!(usage.provider_id, "openai");
        assert_eq!(usage.model_id, "gpt-4o");
        assert_eq!(usage.input_tokens, 1000);
        assert_eq!(usage.output_tokens, 500);
        assert_eq!(usage.reasoning_tokens, 100);
        assert_eq!(usage.cache_read_tokens, 50);
        assert_eq!(usage.cache_write_tokens, 25);
        assert!(usage.pushed);
        assert_eq!(usage.recorded_at, 1234567890);
        assert_eq!(usage.created_at, 1234567800);
    }

    #[test]
    fn row_to_token_usage_handles_null_agent() {
        let row = TokenUsageRow {
            id: "test-id".to_string(),
            client: "vscode".to_string(),
            session_id: "session-123".to_string(),
            message_id: "msg-456".to_string(),
            agent: None,
            provider_id: "openai".to_string(),
            model_id: "gpt-4o".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            reasoning_tokens: 100,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            pushed: 0,
            recorded_at: 1234567890,
            created_at: 1234567800,
        };

        let usage = row_to_token_usage(row);

        assert_eq!(usage.agent, None);
        assert!(!usage.pushed);
    }

    #[test]
    fn row_to_token_usage_converts_pushed_integer_to_bool() {
        let row_pushed = TokenUsageRow {
            id: "test-id".to_string(),
            client: "vscode".to_string(),
            session_id: "session-123".to_string(),
            message_id: "msg-456".to_string(),
            agent: None,
            provider_id: "openai".to_string(),
            model_id: "gpt-4o".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            reasoning_tokens: 100,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            pushed: 1,
            recorded_at: 1234567890,
            created_at: 1234567800,
        };

        let row_not_pushed = TokenUsageRow {
            pushed: 0,
            ..row_pushed.clone()
        };

        assert!(row_to_token_usage(row_pushed).pushed);
        assert!(!row_to_token_usage(row_not_pushed).pushed);
    }

    #[test]
    fn cutoff_clause_with_cutoff() {
        let clause = cutoff_clause(Some(1234567890));
        assert_eq!(clause, " WHERE recorded_at >= ?");
    }

    #[test]
    fn cutoff_clause_without_cutoff() {
        let clause = cutoff_clause(None);
        assert_eq!(clause, "");
    }

    #[test]
    fn map_err_wraps_sqlx_error() {
        let sqlx_err = sqlx::Error::PoolTimedOut;
        let omc_err = map_err(sqlx_err);

        match omc_err {
            OmcError::Storage(msg) => {
                assert!(msg.contains("SQLite error"));
            }
            _ => panic!("Expected OmcError::Storage"),
        }
    }
}
