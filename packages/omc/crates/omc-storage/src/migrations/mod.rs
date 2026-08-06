pub mod registry;
mod v1_initial_schema;

use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, Clone)]
pub struct DialectSql {
    pub sqlite: &'static str,
    pub postgres: &'static str,
}

impl DialectSql {
    pub fn for_sqlite(&self) -> &'static str {
        self.sqlite
    }

    pub fn for_postgres(&self) -> &'static str {
        self.postgres
    }
}

#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up_sql: DialectSql,
    pub down_sql: Option<DialectSql>,
}

impl Migration {
    pub fn checksum(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.up_sql.sqlite.as_bytes());
        hasher.update(b"|||");
        hasher.update(self.up_sql.postgres.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[derive(Debug, Clone)]
pub struct AppliedMigration {
    pub version: i64,
    pub name: String,
    pub applied_at: i64,
    pub checksum: String,
    pub dialect: String,
}

#[derive(Debug)]
pub enum MigrationError {
    Sqlx(sqlx::Error),
    ChecksumMismatch {
        version: i64,
        name: String,
        stored_checksum: String,
        current_checksum: String,
    },
    MissingDownMigration {
        version: i64,
        name: String,
    },
    InvalidTargetVersion {
        current: i64,
        target: i64,
    },
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationError::Sqlx(e) => write!(f, "SQLx error: {e}"),
            MigrationError::ChecksumMismatch {
                version,
                name,
                stored_checksum,
                current_checksum,
            } => {
                write!(
                    f,
                    "Checksum mismatch for migration v{version} ({name}): stored={stored_checksum}, current={current_checksum}"
                )
            }
            MigrationError::MissingDownMigration { version, name } => {
                write!(f, "Down migration missing for v{version} ({name})")
            }
            MigrationError::InvalidTargetVersion { current, target } => {
                write!(f, "Invalid target version {target} (current: {current})")
            }
        }
    }
}

impl std::error::Error for MigrationError {}

impl From<sqlx::Error> for MigrationError {
    fn from(e: sqlx::Error) -> Self {
        MigrationError::Sqlx(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Sqlite,
    Postgres,
}

impl Dialect {
    pub fn as_str(&self) -> &'static str {
        match self {
            Dialect::Sqlite => "sqlite",
            Dialect::Postgres => "postgres",
        }
    }
}

pub struct MigrationRunner;

impl MigrationRunner {
    pub async fn run_sqlite(
        pool: &sqlx::SqlitePool,
        migrations: &[Migration],
    ) -> Result<(), MigrationError> {
        Self::ensure_migrations_table_sqlite(pool).await?;
        let applied = Self::load_applied_sqlite(pool).await?;
        Self::apply_pending_sqlite(pool, migrations, &applied).await
    }

    pub async fn run_postgres(
        pool: &sqlx::PgPool,
        migrations: &[Migration],
    ) -> Result<(), MigrationError> {
        Self::ensure_migrations_table_postgres(pool).await?;
        let applied = Self::load_applied_postgres(pool).await?;
        Self::apply_pending_postgres(pool, migrations, &applied).await
    }

    pub async fn down_sqlite(
        pool: &sqlx::SqlitePool,
        migrations: &[Migration],
        target: i64,
    ) -> Result<(), MigrationError> {
        let applied = Self::load_applied_sqlite(pool).await?;
        Self::rollback_sqlite(pool, migrations, &applied, target).await
    }

    pub async fn down_postgres(
        pool: &sqlx::PgPool,
        migrations: &[Migration],
        target: i64,
    ) -> Result<(), MigrationError> {
        let applied = Self::load_applied_postgres(pool).await?;
        Self::rollback_postgres(pool, migrations, &applied, target).await
    }

    async fn ensure_migrations_table_sqlite(pool: &sqlx::SqlitePool) -> Result<(), MigrationError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version BIGINT PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at BIGINT NOT NULL,
                checksum TEXT NOT NULL,
                dialect TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn ensure_migrations_table_postgres(pool: &sqlx::PgPool) -> Result<(), MigrationError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version BIGINT PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at BIGINT NOT NULL,
                checksum TEXT NOT NULL,
                dialect TEXT NOT NULL
            )",
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn load_applied_sqlite(
        pool: &sqlx::SqlitePool,
    ) -> Result<Vec<AppliedMigration>, MigrationError> {
        let rows: Vec<(i64, String, i64, String, String)> = sqlx::query_as(
            "SELECT version, name, applied_at, checksum, dialect FROM _migrations ORDER BY version ASC",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(version, name, applied_at, checksum, dialect)| AppliedMigration {
                    version,
                    name,
                    applied_at,
                    checksum,
                    dialect,
                },
            )
            .collect())
    }

    async fn load_applied_postgres(
        pool: &sqlx::PgPool,
    ) -> Result<Vec<AppliedMigration>, MigrationError> {
        let rows: Vec<(i64, String, i64, String, String)> = sqlx::query_as(
            "SELECT version, name, applied_at, checksum, dialect FROM _migrations ORDER BY version ASC",
        )
        .fetch_all(pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(version, name, applied_at, checksum, dialect)| AppliedMigration {
                    version,
                    name,
                    applied_at,
                    checksum,
                    dialect,
                },
            )
            .collect())
    }

    async fn apply_pending_sqlite(
        pool: &sqlx::SqlitePool,
        migrations: &[Migration],
        applied: &[AppliedMigration],
    ) -> Result<(), MigrationError> {
        let mut sorted_migrations = migrations.to_vec();
        sorted_migrations.sort_by_key(|m| m.version);

        let applied_versions: std::collections::HashMap<i64, &AppliedMigration> =
            applied.iter().map(|a| (a.version, a)).collect();

        for migration in &sorted_migrations {
            if let Some(applied_mig) = applied_versions.get(&migration.version) {
                if applied_mig.checksum != migration.checksum() {
                    return Err(MigrationError::ChecksumMismatch {
                        version: migration.version,
                        name: migration.name.to_string(),
                        stored_checksum: applied_mig.checksum.clone(),
                        current_checksum: migration.checksum(),
                    });
                }
                continue;
            }

            let sql = migration.up_sql.for_sqlite();
            let now = chrono::Utc::now().timestamp_millis();

            sqlx::query(sql).execute(pool).await?;

            sqlx::query(
                "INSERT INTO _migrations (version, name, applied_at, checksum, dialect) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(migration.version)
            .bind(migration.name)
            .bind(now)
            .bind(migration.checksum())
            .bind(Dialect::Sqlite.as_str())
            .execute(pool)
            .await?;

            tracing::info!(
                "Applied migration v{} ({})",
                migration.version,
                migration.name
            );
        }

        Ok(())
    }

    async fn apply_pending_postgres(
        pool: &sqlx::PgPool,
        migrations: &[Migration],
        applied: &[AppliedMigration],
    ) -> Result<(), MigrationError> {
        let mut sorted_migrations = migrations.to_vec();
        sorted_migrations.sort_by_key(|m| m.version);

        let applied_versions: std::collections::HashMap<i64, &AppliedMigration> =
            applied.iter().map(|a| (a.version, a)).collect();

        for migration in &sorted_migrations {
            if let Some(applied_mig) = applied_versions.get(&migration.version) {
                if applied_mig.checksum != migration.checksum() {
                    return Err(MigrationError::ChecksumMismatch {
                        version: migration.version,
                        name: migration.name.to_string(),
                        stored_checksum: applied_mig.checksum.clone(),
                        current_checksum: migration.checksum(),
                    });
                }
                continue;
            }

            let sql = migration.up_sql.for_postgres();
            let now = chrono::Utc::now().timestamp_millis();

            sqlx::query(sql).execute(pool).await?;

            sqlx::query(
                "INSERT INTO _migrations (version, name, applied_at, checksum, dialect) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(migration.version)
            .bind(migration.name)
            .bind(now)
            .bind(migration.checksum())
            .bind(Dialect::Postgres.as_str())
            .execute(pool)
            .await?;

            tracing::info!(
                "Applied migration v{} ({})",
                migration.version,
                migration.name
            );
        }

        Ok(())
    }

    async fn rollback_sqlite(
        pool: &sqlx::SqlitePool,
        migrations: &[Migration],
        applied: &[AppliedMigration],
        target: i64,
    ) -> Result<(), MigrationError> {
        let current = applied.last().map(|a| a.version).unwrap_or(0);
        if target > current {
            return Err(MigrationError::InvalidTargetVersion { current, target });
        }

        let mut sorted_applied = applied.to_vec();
        sorted_applied.sort_by_key(|a| std::cmp::Reverse(a.version));

        let migration_map: std::collections::HashMap<i64, &Migration> =
            migrations.iter().map(|m| (m.version, m)).collect();

        for applied_mig in sorted_applied {
            if applied_mig.version <= target {
                break;
            }

            let migration = migration_map.get(&applied_mig.version).ok_or_else(|| {
                MigrationError::ChecksumMismatch {
                    version: applied_mig.version,
                    name: applied_mig.name.clone(),
                    stored_checksum: applied_mig.checksum.clone(),
                    current_checksum: String::new(),
                }
            })?;

            let down_sql = migration.down_sql.as_ref().ok_or_else(|| {
                MigrationError::MissingDownMigration {
                    version: migration.version,
                    name: migration.name.to_string(),
                }
            })?;

            let sql = down_sql.for_sqlite();

            sqlx::query(sql).execute(pool).await?;

            sqlx::query("DELETE FROM _migrations WHERE version = ?")
                .bind(migration.version)
                .execute(pool)
                .await?;

            tracing::info!(
                "Rolled back migration v{} ({})",
                migration.version,
                migration.name
            );
        }

        Ok(())
    }

    async fn rollback_postgres(
        pool: &sqlx::PgPool,
        migrations: &[Migration],
        applied: &[AppliedMigration],
        target: i64,
    ) -> Result<(), MigrationError> {
        let current = applied.last().map(|a| a.version).unwrap_or(0);
        if target > current {
            return Err(MigrationError::InvalidTargetVersion { current, target });
        }

        let mut sorted_applied = applied.to_vec();
        sorted_applied.sort_by_key(|a| std::cmp::Reverse(a.version));

        let migration_map: std::collections::HashMap<i64, &Migration> =
            migrations.iter().map(|m| (m.version, m)).collect();

        for applied_mig in sorted_applied {
            if applied_mig.version <= target {
                break;
            }

            let migration = migration_map.get(&applied_mig.version).ok_or_else(|| {
                MigrationError::ChecksumMismatch {
                    version: applied_mig.version,
                    name: applied_mig.name.clone(),
                    stored_checksum: applied_mig.checksum.clone(),
                    current_checksum: String::new(),
                }
            })?;

            let down_sql = migration.down_sql.as_ref().ok_or_else(|| {
                MigrationError::MissingDownMigration {
                    version: migration.version,
                    name: migration.name.to_string(),
                }
            })?;

            let sql = down_sql.for_postgres();

            sqlx::query(sql).execute(pool).await?;

            sqlx::query("DELETE FROM _migrations WHERE version = $1")
                .bind(migration.version)
                .execute(pool)
                .await?;

            tracing::info!(
                "Rolled back migration v{} ({})",
                migration.version,
                migration.name
            );
        }

        Ok(())
    }
}
