use omc_core::account::Workspace;
use omc_core::error::{OmcError, Result};
use sqlx::FromRow;
use sqlx::SqlitePool;

#[derive(Debug, Clone, FromRow)]
struct WorkspaceRow {
    id: String,
    account_id: String,
    name: String,
    is_admin: i32,
}

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("SQLite error: {e}"))
}

pub(crate) async fn list_workspaces(pool: &SqlitePool, account_id: &str) -> Result<Vec<Workspace>> {
    let rows: Vec<WorkspaceRow> =
        sqlx::query_as("SELECT id, account_id, name, is_admin FROM workspace WHERE account_id = ?")
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
            is_admin: r.is_admin != 0,
        })
        .collect())
}

pub(crate) async fn upsert_workspaces(pool: &SqlitePool, workspaces: &[Workspace]) -> Result<()> {
    for w in workspaces {
        let is_admin = if w.is_admin { 1 } else { 0 };
        sqlx::query(
            "INSERT INTO workspace (id, account_id, name, is_admin)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
                account_id = excluded.account_id,
                name = excluded.name,
                is_admin = excluded.is_admin",
        )
        .bind(&w.id)
        .bind(&w.account_id)
        .bind(&w.name)
        .bind(is_admin)
        .execute(pool)
        .await
        .map_err(map_err)?;
    }
    Ok(())
}

pub(crate) async fn clear_workspaces(pool: &SqlitePool, account_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM workspace WHERE account_id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}
