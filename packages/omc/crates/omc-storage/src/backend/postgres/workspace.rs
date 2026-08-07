use omc_core::account::Workspace;
use omc_core::error::{OmcError, Result};
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, FromRow)]
struct WorkspaceRow {
    id: String,
    account_id: String,
    name: String,
    is_admin: bool,
}

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("Postgres error: {e}"))
}

pub(crate) async fn list_workspaces(pool: &PgPool, account_id: &str) -> Result<Vec<Workspace>> {
    let rows: Vec<WorkspaceRow> = sqlx::query_as(
        "SELECT id, account_id, name, is_admin FROM workspace WHERE account_id = $1",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    Ok(rows
        .into_iter()
        .map(|r| Workspace {
            id: r.id,
            account_id: r.account_id,
            name: r.name,
            is_admin: r.is_admin,
        })
        .collect())
}

pub(crate) async fn upsert_workspaces(pool: &PgPool, workspaces: &[Workspace]) -> Result<()> {
    for w in workspaces {
        sqlx::query(
            "INSERT INTO workspace (id, account_id, name, is_admin)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT(id) DO UPDATE SET
                account_id = EXCLUDED.account_id,
                name = EXCLUDED.name,
                is_admin = EXCLUDED.is_admin",
        )
        .bind(&w.id)
        .bind(&w.account_id)
        .bind(&w.name)
        .bind(w.is_admin)
        .execute(pool)
        .await
        .map_err(map_err)?;
    }
    Ok(())
}

pub(crate) async fn clear_workspaces(pool: &PgPool, account_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM workspace WHERE account_id = $1")
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}
