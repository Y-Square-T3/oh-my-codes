use omc_core::account::Account;
use omc_core::error::{OmcError, Result};
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};

#[derive(Debug, Clone, FromRow)]
struct AccountRow {
    id: String,
    email: String,
    url: String,
    access_token: String,
    refresh_token: String,
    token_expiry: i64,
    active_workspace_id: Option<String>,
}

fn map_err(e: sqlx::Error) -> OmcError {
    OmcError::Storage(format!("Postgres error: {e}"))
}

fn row_to_account(r: AccountRow) -> Account {
    Account {
        id: r.id,
        email: r.email,
        url: r.url,
        access_token: r.access_token,
        refresh_token: r.refresh_token,
        token_expiry: r.token_expiry,
        active_workspace_id: r.active_workspace_id,
    }
}

pub(crate) async fn get_account(pool: &PgPool, id: &str) -> Result<Option<Account>> {
    let row: Option<AccountRow> = sqlx::query_as(
        "SELECT id, email, url, access_token, refresh_token, token_expiry, active_workspace_id FROM account WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    Ok(row.map(row_to_account))
}

pub(crate) async fn list_accounts(pool: &PgPool) -> Result<Vec<Account>> {
    let rows: Vec<AccountRow> = sqlx::query_as(
        "SELECT id, email, url, access_token, refresh_token, token_expiry, active_workspace_id FROM account",
    )
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    Ok(rows.into_iter().map(row_to_account).collect())
}

pub(crate) async fn upsert_account(pool: &PgPool, account: &Account) -> Result<()> {
    sqlx::query(
        "INSERT INTO account (id, email, url, access_token, refresh_token, token_expiry, active_workspace_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT(id) DO UPDATE SET
            email = EXCLUDED.email,
            url = EXCLUDED.url,
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            token_expiry = EXCLUDED.token_expiry,
            active_workspace_id = EXCLUDED.active_workspace_id",
    )
    .bind(&account.id)
    .bind(&account.email)
    .bind(&account.url)
    .bind(&account.access_token)
    .bind(&account.refresh_token)
    .bind(account.token_expiry)
    .bind(&account.active_workspace_id)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn delete_account(pool: &PgPool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM account WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn get_active_account_id(pool: &PgPool) -> Result<Option<String>> {
    let row: Option<PgRow> = sqlx::query("SELECT account_id FROM active_account WHERE id = 1")
        .fetch_optional(pool)
        .await
        .map_err(map_err)?;
    Ok(row.and_then(|r| r.get::<Option<String>, _>("account_id")))
}

pub(crate) async fn set_active_account(pool: &PgPool, id: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO active_account (id, account_id) VALUES (1, $1)
         ON CONFLICT(id) DO UPDATE SET account_id = EXCLUDED.account_id",
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn clear_active_account(pool: &PgPool) -> Result<()> {
    sqlx::query("DELETE FROM active_account")
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn set_active_workspace(
    pool: &PgPool,
    account_id: &str,
    workspace_id: &str,
) -> Result<()> {
    let result = sqlx::query("UPDATE account SET active_workspace_id = $1 WHERE id = $2")
        .bind(workspace_id)
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    if result.rows_affected() == 0 {
        return Err(OmcError::Storage(format!(
            "Account '{account_id}' not found"
        )));
    }
    Ok(())
}
