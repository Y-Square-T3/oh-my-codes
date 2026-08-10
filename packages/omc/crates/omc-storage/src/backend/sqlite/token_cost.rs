use omc_core::error::{OmcError, Result};
use omc_core::token_usage::TokenCost;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, FromRow)]
struct TokenCostRow {
    usage_id: String,
    input_cost_micros: i64,
    output_cost_micros: i64,
    reasoning_cost_micros: i64,
    cache_read_cost_micros: i64,
    cache_write_cost_micros: i64,
    audio_input_cost_micros: i64,
    video_input_cost_micros: i64,
    image_input_cost_micros: i64,
    total_cost_micros: i64,
}

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("SQLite error: {e}"))
}

fn row_to_token_cost(r: TokenCostRow) -> TokenCost {
    TokenCost {
        usage_id: r.usage_id,
        input_cost_micros: r.input_cost_micros,
        output_cost_micros: r.output_cost_micros,
        reasoning_cost_micros: r.reasoning_cost_micros,
        cache_read_cost_micros: r.cache_read_cost_micros,
        cache_write_cost_micros: r.cache_write_cost_micros,
        audio_input_cost_micros: r.audio_input_cost_micros,
        video_input_cost_micros: r.video_input_cost_micros,
        image_input_cost_micros: r.image_input_cost_micros,
        total_cost_micros: r.total_cost_micros,
    }
}

pub(crate) async fn upsert_token_cost(pool: &SqlitePool, cost: &TokenCost) -> Result<()> {
    sqlx::query(
        "INSERT INTO token_cost (usage_id, input_cost_micros, output_cost_micros, reasoning_cost_micros, cache_read_cost_micros, cache_write_cost_micros, audio_input_cost_micros, video_input_cost_micros, image_input_cost_micros, total_cost_micros)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(usage_id) DO NOTHING",
    )
    .bind(&cost.usage_id)
    .bind(cost.input_cost_micros)
    .bind(cost.output_cost_micros)
    .bind(cost.reasoning_cost_micros)
    .bind(cost.cache_read_cost_micros)
    .bind(cost.cache_write_cost_micros)
    .bind(cost.audio_input_cost_micros)
    .bind(cost.video_input_cost_micros)
    .bind(cost.image_input_cost_micros)
    .bind(cost.total_cost_micros)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn get_token_cost(pool: &SqlitePool, usage_id: &str) -> Result<Option<TokenCost>> {
    let row: Option<TokenCostRow> = sqlx::query_as(
        "SELECT usage_id, input_cost_micros, output_cost_micros, reasoning_cost_micros, cache_read_cost_micros, cache_write_cost_micros, audio_input_cost_micros, video_input_cost_micros, image_input_cost_micros, total_cost_micros FROM token_cost WHERE usage_id = ?",
    )
    .bind(usage_id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    Ok(row.map(row_to_token_cost))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_to_token_cost_maps_all_fields() {
        let row = TokenCostRow {
            usage_id: "usage-123".to_string(),
            input_cost_micros: 5000,
            output_cost_micros: 7500,
            reasoning_cost_micros: 3000,
            cache_read_cost_micros: 100,
            cache_write_cost_micros: 200,
            audio_input_cost_micros: 0,
            video_input_cost_micros: 0,
            image_input_cost_micros: 0,
            total_cost_micros: 15800,
        };

        let cost = row_to_token_cost(row);

        assert_eq!(cost.usage_id, "usage-123");
        assert_eq!(cost.input_cost_micros, 5000);
        assert_eq!(cost.output_cost_micros, 7500);
        assert_eq!(cost.reasoning_cost_micros, 3000);
        assert_eq!(cost.cache_read_cost_micros, 100);
        assert_eq!(cost.cache_write_cost_micros, 200);
        assert_eq!(cost.audio_input_cost_micros, 0);
        assert_eq!(cost.video_input_cost_micros, 0);
        assert_eq!(cost.image_input_cost_micros, 0);
        assert_eq!(cost.total_cost_micros, 15800);
    }
}
