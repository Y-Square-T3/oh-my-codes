use omc_storage::migrations::{MigrationError, MigrationRunner, registry};
use omc_storage::sqlite::SqliteStorage;

#[tokio::test]
async fn test_sqlite_migrations_run_successfully() {
    let storage = SqliteStorage::new_memory().await.unwrap();
    let pool = storage.pool();

    let row: Option<(i64,)> = sqlx::query_as("SELECT COUNT(*) FROM _migrations WHERE version = 1")
        .fetch_optional(&*pool)
        .await
        .unwrap();

    assert!(row.is_some());
    assert_eq!(row.unwrap().0, 1);
}

#[tokio::test]
async fn test_sqlite_migrations_create_tables() {
    let storage = SqliteStorage::new_memory().await.unwrap();
    let pool = storage.pool();

    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(&*pool)
    .await
    .unwrap();

    let table_names: Vec<String> = tables.into_iter().map(|t| t.0).collect();
    assert!(table_names.contains(&"_migrations".to_string()));
    assert!(table_names.contains(&"channel".to_string()));
    assert!(table_names.contains(&"message".to_string()));
    assert!(table_names.contains(&"account".to_string()));
    assert!(table_names.contains(&"workspace".to_string()));
    assert!(table_names.contains(&"active_account".to_string()));
    assert!(table_names.contains(&"provider".to_string()));
    assert!(table_names.contains(&"token_usage".to_string()));
}

#[tokio::test]
async fn test_sqlite_migrations_idempotent() {
    let storage = SqliteStorage::new_memory().await.unwrap();
    let pool = storage.pool();

    MigrationRunner::run_sqlite(&pool, &registry::sqlite_migrations())
        .await
        .unwrap();

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _migrations")
        .fetch_one(&*pool)
        .await
        .unwrap();

    assert_eq!(row.0, 1);
}

#[tokio::test]
async fn test_sqlite_down_migration() {
    let storage = SqliteStorage::new_memory().await.unwrap();
    let pool = storage.pool();

    MigrationRunner::down_sqlite(&pool, &registry::sqlite_migrations(), 0)
        .await
        .unwrap();

    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM _migrations")
        .fetch_one(&*pool)
        .await
        .unwrap();

    assert_eq!(row.0, 0);

    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_migrations'",
    )
    .fetch_all(&*pool)
    .await
    .unwrap();

    assert!(tables.is_empty());
}

#[tokio::test]
async fn test_sqlite_checksum_mismatch() {
    let storage = SqliteStorage::new_memory().await.unwrap();
    let pool = storage.pool();

    let mut migrations = registry::sqlite_migrations();
    migrations[0].up_sql.sqlite = "SELECT 1";

    let result = MigrationRunner::run_sqlite(&pool, &migrations).await;

    assert!(matches!(
        result,
        Err(MigrationError::ChecksumMismatch { .. })
    ));
}
