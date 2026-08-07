use omc_core::error::{OmcError, Result};
use omc_core::model::Provider;
use sqlx::FromRow;
use sqlx::SqlitePool;

#[derive(Debug, Clone, FromRow)]
struct ProviderRow {
    provider_id: String,
    name: String,
    env: String,
    api: Option<String>,
    npm: Option<String>,
    doc: Option<String>,
    models: String,
    account_id: String,
    last_fetched_at: i64,
}

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("SQLite error: {e}"))
}

pub(crate) async fn list_providers(pool: &SqlitePool, account_id: &str) -> Result<Vec<Provider>> {
    let rows: Vec<ProviderRow> = sqlx::query_as(
        "SELECT provider_id, name, env, api, npm, doc, models, account_id, last_fetched_at FROM provider WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    rows.into_iter()
        .map(|r| {
            let env: Vec<String> = serde_json::from_str(&r.env)
                .map_err(|e| OmcError::Storage(format!("Failed to deserialize env: {e}")))?;
            let models: Vec<omc_core::model::Model> = serde_json::from_str(&r.models)
                .map_err(|e| OmcError::Storage(format!("Failed to deserialize models: {e}")))?;
            Ok(Provider {
                id: r.provider_id,
                name: r.name,
                env,
                api: r.api,
                npm: r.npm,
                doc: r.doc,
                models,
                account_id: r.account_id,
                last_fetched_at: r.last_fetched_at,
            })
        })
        .collect()
}

pub(crate) async fn delete_providers(pool: &SqlitePool, account_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM provider WHERE account_id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn replace_providers(
    pool: &SqlitePool,
    account_id: &str,
    providers: Vec<Provider>,
) -> Result<()> {
    delete_providers(pool, account_id).await?;
    for p in providers {
        let id = format!("{}:{}", p.id, account_id);
        let env_json = serde_json::to_string(&p.env)
            .map_err(|e| OmcError::Storage(format!("Failed to serialize env: {e}")))?;
        let models_json = serde_json::to_string(&p.models)
            .map_err(|e| OmcError::Storage(format!("Failed to serialize models: {e}")))?;
        sqlx::query(
            "INSERT INTO provider (id, provider_id, name, env, api, npm, doc, models, account_id, last_fetched_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&p.id)
        .bind(&p.name)
        .bind(&env_json)
        .bind(&p.api)
        .bind(&p.npm)
        .bind(&p.doc)
        .bind(&models_json)
        .bind(&p.account_id)
        .bind(p.last_fetched_at)
        .execute(pool)
        .await
        .map_err(map_err)?;
    }
    Ok(())
}
