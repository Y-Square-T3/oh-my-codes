use omc_core::account::Account;
use omc_core::error::{OmcError, Result};
use sqlx::sqlite::SqliteRow;
use sqlx::{FromRow, Row, SqlitePool};

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
    OmcError::Storage(format!("SQLite error: {e}"))
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

pub(crate) async fn get_account(pool: &SqlitePool, id: &str) -> Result<Option<Account>> {
    let row: Option<AccountRow> = sqlx::query_as(
        "SELECT id, email, url, access_token, refresh_token, token_expiry, active_workspace_id FROM account WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    Ok(row.map(row_to_account))
}

pub(crate) async fn list_accounts(pool: &SqlitePool) -> Result<Vec<Account>> {
    let rows: Vec<AccountRow> = sqlx::query_as(
        "SELECT id, email, url, access_token, refresh_token, token_expiry, active_workspace_id FROM account",
    )
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    Ok(rows.into_iter().map(row_to_account).collect())
}

pub(crate) async fn upsert_account(pool: &SqlitePool, account: &Account) -> Result<()> {
    sqlx::query(
        "INSERT INTO account (id, email, url, access_token, refresh_token, token_expiry, active_workspace_id)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            email = excluded.email,
            url = excluded.url,
            access_token = excluded.access_token,
            refresh_token = excluded.refresh_token,
            token_expiry = excluded.token_expiry,
            active_workspace_id = excluded.active_workspace_id",
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

pub(crate) async fn delete_account(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM account WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn get_active_account_id(pool: &SqlitePool) -> Result<Option<String>> {
    let row: Option<SqliteRow> = sqlx::query("SELECT account_id FROM active_account WHERE id = 1")
        .fetch_optional(pool)
        .await
        .map_err(map_err)?;
    Ok(row.and_then(|r| r.get::<Option<String>, _>("account_id")))
}

pub(crate) async fn set_active_account(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO active_account (id, account_id) VALUES (1, ?)
         ON CONFLICT(id) DO UPDATE SET account_id = excluded.account_id",
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn clear_active_account(pool: &SqlitePool) -> Result<()> {
    sqlx::query("DELETE FROM active_account")
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

pub(crate) async fn set_active_workspace(
    pool: &SqlitePool,
    account_id: &str,
    workspace_id: &str,
) -> Result<()> {
    let result = sqlx::query("UPDATE account SET active_workspace_id = ? WHERE id = ?")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_to_account_maps_all_fields() {
        let row = AccountRow {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            url: "https://api.example.com".to_string(),
            access_token: "access-token".to_string(),
            refresh_token: "refresh-token".to_string(),
            token_expiry: 1234567890,
            active_workspace_id: Some("workspace-id".to_string()),
        };

        let account = row_to_account(row);

        assert_eq!(account.id, "test-id");
        assert_eq!(account.email, "test@example.com");
        assert_eq!(account.url, "https://api.example.com");
        assert_eq!(account.access_token, "access-token");
        assert_eq!(account.refresh_token, "refresh-token");
        assert_eq!(account.token_expiry, 1234567890);
        assert_eq!(account.active_workspace_id, Some("workspace-id".to_string()));
    }

    #[test]
    fn row_to_account_handles_null_workspace() {
        let row = AccountRow {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            url: "https://api.example.com".to_string(),
            access_token: "access-token".to_string(),
            refresh_token: "refresh-token".to_string(),
            token_expiry: 1234567890,
            active_workspace_id: None,
        };

        let account = row_to_account(row);

        assert_eq!(account.active_workspace_id, None);
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
