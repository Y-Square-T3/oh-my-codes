use omc_core::error::{OmcError, Result};
use omc_core::token_usage::{
    DailyUsage, HeadlineStats, TokenUsage, TokenUsageOverview, UsageGroup, UsageSummary,
};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};

#[derive(Debug, Clone, FromRow)]
struct TokenUsageRow {
    id: String,
    workspace_id: Option<String>,
    session_id: String,
    agent: String,
    model: String,
    metadata: Option<String>,
    input_tokens: i64,
    output_tokens: i64,
    reasoning_tokens: i64,
    cache_read_tokens: i64,
    cache_write_tokens: i64,
    audio_input_tokens: i64,
    video_input_tokens: i64,
    image_input_tokens: i64,
    total_tokens: i64,
    pushed: bool,
    recorded_at: i64,
    created_at: i64,
}

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("Postgres error: {e}"))
}

fn row_to_token_usage(r: TokenUsageRow) -> TokenUsage {
    TokenUsage {
        id: r.id,
        workspace_id: r.workspace_id,
        session_id: r.session_id,
        agent: r.agent,
        model: r.model,
        metadata: r.metadata,
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        reasoning_tokens: r.reasoning_tokens,
        cache_read_tokens: r.cache_read_tokens,
        cache_write_tokens: r.cache_write_tokens,
        audio_input_tokens: r.audio_input_tokens,
        video_input_tokens: r.video_input_tokens,
        image_input_tokens: r.image_input_tokens,
        total_tokens: r.total_tokens,
        pushed: r.pushed,
        recorded_at: r.recorded_at,
        created_at: r.created_at,
    }
}

fn cutoff_clause(cutoff: Option<i64>) -> &'static str {
    if cutoff.is_some() {
        " WHERE recorded_at >= $1"
    } else {
        ""
    }
}

async fn fetch_with_optional_cutoff(
    pool: &PgPool,
    sql: &str,
    cutoff: Option<i64>,
) -> std::result::Result<Vec<PgRow>, sqlx::Error> {
    if let Some(c) = cutoff {
        sqlx::query(sql).bind(c).fetch_all(pool).await
    } else {
        sqlx::query(sql).fetch_all(pool).await
    }
}

const ALL_COLUMNS: &str = "id, workspace_id, session_id, agent, model, metadata, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, audio_input_tokens, video_input_tokens, image_input_tokens, total_tokens, pushed, recorded_at, created_at";

pub(crate) async fn upsert_usage(pool: &PgPool, usage: &TokenUsage) -> Result<()> {
    sqlx::query(
        "INSERT INTO token_usage (id, workspace_id, session_id, agent, model, metadata, input_tokens, output_tokens, reasoning_tokens, cache_read_tokens, cache_write_tokens, audio_input_tokens, video_input_tokens, image_input_tokens, total_tokens, pushed, recorded_at, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
         ON CONFLICT(id) DO UPDATE SET
            workspace_id = EXCLUDED.workspace_id,
            session_id = EXCLUDED.session_id,
            agent = EXCLUDED.agent,
            model = EXCLUDED.model,
            metadata = EXCLUDED.metadata,
            input_tokens = EXCLUDED.input_tokens,
            output_tokens = EXCLUDED.output_tokens,
            reasoning_tokens = EXCLUDED.reasoning_tokens,
            cache_read_tokens = EXCLUDED.cache_read_tokens,
            cache_write_tokens = EXCLUDED.cache_write_tokens,
            audio_input_tokens = EXCLUDED.audio_input_tokens,
            video_input_tokens = EXCLUDED.video_input_tokens,
            image_input_tokens = EXCLUDED.image_input_tokens,
            total_tokens = EXCLUDED.total_tokens,
            pushed = EXCLUDED.pushed,
            recorded_at = EXCLUDED.recorded_at,
            created_at = EXCLUDED.created_at",
    )
    .bind(&usage.id)
    .bind(&usage.workspace_id)
    .bind(&usage.session_id)
    .bind(&usage.agent)
    .bind(&usage.model)
    .bind(&usage.metadata)
    .bind(usage.input_tokens)
    .bind(usage.output_tokens)
    .bind(usage.reasoning_tokens)
    .bind(usage.cache_read_tokens)
    .bind(usage.cache_write_tokens)
    .bind(usage.audio_input_tokens)
    .bind(usage.video_input_tokens)
    .bind(usage.image_input_tokens)
    .bind(usage.total_tokens)
    .bind(usage.pushed)
    .bind(usage.recorded_at)
    .bind(usage.created_at)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn find_unpushed(pool: &PgPool, limit: usize) -> Result<Vec<TokenUsage>> {
    let sql = format!(
        "SELECT {ALL_COLUMNS} FROM token_usage WHERE pushed = false ORDER BY recorded_at ASC LIMIT $1"
    );
    let rows: Vec<TokenUsageRow> = sqlx::query_as(&sql)
        .bind(limit as i64)
        .fetch_all(pool)
        .await
        .map_err(map_err)?;
    Ok(rows.into_iter().map(row_to_token_usage).collect())
}

pub(crate) async fn count_unpushed(pool: &PgPool) -> Result<usize> {
    let row: PgRow = sqlx::query("SELECT COUNT(*) as count FROM token_usage WHERE pushed = false")
        .fetch_one(pool)
        .await
        .map_err(map_err)?;
    let count: i64 = row.get("count");
    Ok(count as usize)
}

pub(crate) async fn mark_pushed(pool: &PgPool, ids: &[String]) -> Result<()> {
    if ids.is_empty() {
        return Ok(());
    }
    let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${}", i)).collect();
    let query = format!(
        "UPDATE token_usage SET pushed = true WHERE id IN ({})",
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
    pool: &PgPool,
    limit: usize,
    offset: usize,
    pushed: Option<bool>,
) -> Result<Vec<TokenUsage>> {
    let rows: Vec<TokenUsageRow> = if let Some(pushed_val) = pushed {
        let where_clause = if pushed_val {
            " WHERE pushed = true"
        } else {
            " WHERE pushed = false"
        };
        let sql = format!(
            "SELECT {ALL_COLUMNS} FROM token_usage{where_clause} ORDER BY recorded_at DESC LIMIT $1 OFFSET $2",
        );
        sqlx::query_as(&sql)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(map_err)?
    } else {
        let sql = format!(
            "SELECT {ALL_COLUMNS} FROM token_usage ORDER BY recorded_at DESC LIMIT $1 OFFSET $2"
        );
        sqlx::query_as(&sql)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(pool)
            .await
            .map_err(map_err)?
    };
    Ok(rows.into_iter().map(row_to_token_usage).collect())
}

pub(crate) async fn count_all(pool: &PgPool, pushed: Option<bool>) -> Result<usize> {
    let sql = match pushed {
        Some(true) => "SELECT COUNT(*) as count FROM token_usage WHERE pushed = true",
        Some(false) => "SELECT COUNT(*) as count FROM token_usage WHERE pushed = false",
        None => "SELECT COUNT(*) as count FROM token_usage",
    };
    let row: PgRow = sqlx::query(sql).fetch_one(pool).await.map_err(map_err)?;
    let count: i64 = row.get("count");
    Ok(count as usize)
}

pub(crate) async fn cleanup_old_pushed(pool: &PgPool, retention_days: i64) -> Result<usize> {
    let cutoff = chrono::Utc::now().timestamp_millis() - (retention_days * 86_400_000);
    let result = sqlx::query("DELETE FROM token_usage WHERE pushed = true AND recorded_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(result.rows_affected() as usize)
}

pub(crate) async fn usage_summary(pool: &PgPool, days: Option<i64>) -> Result<Vec<UsageSummary>> {
    let cutoff = days.map(|d| chrono::Utc::now().timestamp_millis() - (d * 86_400_000));
    let sql = format!(
        "SELECT model,
            SUM(input_tokens) as total_input,
            SUM(output_tokens) as total_output,
            SUM(reasoning_tokens) as total_reasoning,
            SUM(cache_read_tokens) as total_cache_read,
            SUM(cache_write_tokens) as total_cache_write,
            COUNT(*) as request_count
          FROM token_usage{}
          GROUP BY model",
        cutoff_clause(cutoff)
    );
    let rows = fetch_with_optional_cutoff(pool, &sql, cutoff)
        .await
        .map_err(map_err)?;
    Ok(rows
        .into_iter()
        .map(|r| UsageSummary {
            model: r.get("model"),
            total_input: r.get("total_input"),
            total_output: r.get("total_output"),
            total_reasoning: r.get("total_reasoning"),
            total_cache_read: r.get("total_cache_read"),
            total_cache_write: r.get("total_cache_write"),
            request_count: r.get("request_count"),
        })
        .collect())
}

pub(crate) async fn usage_overview(pool: &PgPool, days: Option<i64>) -> Result<TokenUsageOverview> {
    let cutoff = days.map(|d| chrono::Utc::now().timestamp_millis() - (d * 86_400_000));
    let seven_days_ago = chrono::Utc::now().timestamp_millis() - (7 * 86_400_000);

    let headline = headline_stats(pool, cutoff).await?;
    let top_models = top_models(pool, cutoff).await?;
    let top_agents = top_agents(pool, cutoff).await?;
    let trend = daily_trend(pool, seven_days_ago).await?;

    Ok(TokenUsageOverview {
        headline,
        top_models,
        top_agents,
        trend,
    })
}

async fn headline_stats(pool: &PgPool, cutoff: Option<i64>) -> Result<HeadlineStats> {
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
    let row: PgRow = fetch_with_optional_cutoff(pool, &sql, cutoff)
        .await
        .map_err(map_err)?
        .into_iter()
        .next()
        .ok_or_else(|| OmcError::Storage("No headline stats row".into()))?;

    let unpushed_row: PgRow = sqlx::query(
        "SELECT COUNT(*) as count, SUM(input_tokens + output_tokens + reasoning_tokens) as tokens FROM token_usage WHERE pushed = false",
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

async fn top_models(pool: &PgPool, cutoff: Option<i64>) -> Result<Vec<UsageSummary>> {
    let sql = format!(
        "SELECT model,
            SUM(input_tokens) as total_input,
            SUM(output_tokens) as total_output,
            SUM(reasoning_tokens) as total_reasoning,
            SUM(cache_read_tokens) as total_cache_read,
            SUM(cache_write_tokens) as total_cache_write,
            COUNT(*) as request_count
          FROM token_usage{}
          GROUP BY model
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
            model: r.get("model"),
            total_input: r.get("total_input"),
            total_output: r.get("total_output"),
            total_reasoning: r.get("total_reasoning"),
            total_cache_read: r.get("total_cache_read"),
            total_cache_write: r.get("total_cache_write"),
            request_count: r.get("request_count"),
        })
        .collect())
}

async fn top_agents(pool: &PgPool, cutoff: Option<i64>) -> Result<Vec<UsageGroup>> {
    let sql = format!(
        "SELECT agent as label,
            SUM(input_tokens) as total_input,
            SUM(output_tokens) as total_output,
            SUM(reasoning_tokens) as total_reasoning,
            SUM(cache_read_tokens) as total_cache_read,
            SUM(cache_write_tokens) as total_cache_write,
            COUNT(*) as request_count
          FROM token_usage{}
          GROUP BY agent
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

async fn daily_trend(pool: &PgPool, cutoff: i64) -> Result<Vec<DailyUsage>> {
    let rows: Vec<PgRow> = sqlx::query(
        "SELECT
            TO_CHAR(TO_TIMESTAMP(recorded_at / 1000.0), 'YYYY-MM-DD') as date,
            COUNT(*) as requests,
            SUM(total_tokens) as total_tokens
          FROM token_usage
          WHERE recorded_at >= $1
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
