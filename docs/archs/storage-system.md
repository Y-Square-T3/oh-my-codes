# Storage System Architecture

## Overview

The storage system (`omc-storage`) provides a unified abstraction layer for persistent data storage in oh-my-codes. It supports multiple database backends (SQLite and PostgreSQL) with a Flyway/Liquibase-style migration framework for schema versioning and data migrations.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Daemon (omcd)                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    Database URL Configuration                     │  │
│  │  ┌────────────────────────────────────────────────────────────┐  │  │
│  │  │  database_url: "sqlite:/path/to/omc.db"                    │  │  │
│  │  │  database_url: "postgres://user:pass@host/db"              │  │  │
│  │  └────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
│                                    │                                    │
│                                    ▼                                    │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                  Storage Layer (omc-storage)                      │  │
│  │                                                                    │  │
│  │  ┌────────────────────────────────────────────────────────────┐  │  │
│  │  │                   Migration System                          │  │  │
│  │  │  ┌──────────────┐  ┌──────────────┐  ┌────────────────┐  │  │  │
│  │  │  │  Migration   │  │   Migration  │  │    Registry    │  │  │  │
│  │  │  │   Runner     │  │   Tracker    │  │                │  │  │  │
│  │  │  └──────────────┘  └──────────────┘  └────────────────┘  │  │  │
│  │  │  ┌──────────────────────────────────────────────────────┐│  │  │
│  │  │  │  SQL Migrations (embedded)                           ││  │  │
│  │  │  │  ┌──────────────┐  ┌──────────────┐                 ││  │  │
│  │  │  │  │   SQLite     │  │  PostgreSQL  │                 ││  │  │
│  │  │  │  │  migrations  │  │  migrations  │                 ││  │  │
│  │  │  │  └──────────────┘  └──────────────┘                 ││  │  │
│  │  │  └──────────────────────────────────────────────────────┘│  │  │
│  │  └────────────────────────────────────────────────────────────┘  │  │
│  │                                    │                              │  │
│  │                                    ▼                              │  │
│  │  ┌────────────────────────────────────────────────────────────┐  │  │
│  │  │                    Store Traits                             │  │  │
│  │  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐  │  │  │
│  │  │  │AccountStore │ │MessageStore │ │   ModelStore        │  │  │  │
│  │  │  └─────────────┘ └─────────────┘ └─────────────────────┘  │  │  │
│  │  │  ┌─────────────┐ ┌─────────────────────────────────────┐  │  │  │
│  │  │  │WorkspaceStore│ │      TokenUsageStore                │  │  │  │
│  │  │  └─────────────┘ └─────────────────────────────────────┘  │  │  │
│  │  └────────────────────────────────────────────────────────────┘  │  │
│  │                                    │                              │  │
│  │                                    ▼                              │  │
│  │  ┌────────────────────────────────────────────────────────────┐  │  │
│  │  │                  Backend Implementations                    │  │  │
│  │  │  ┌───────────────────────────────────────────────────────┐ │  │  │
│  │  │  │                    SQLite                              │ │  │  │
│  │  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────────┐  │ │  │  │
│  │  │  │  │ Account │ │ Message │ │  Model  │ │TokenUsage │  │ │  │  │
│  │  │  │  │  Store  │ │  Store  │ │  Store  │ │   Store   │  │ │  │  │
│  │  │  │  └─────────┘ └─────────┘ └─────────┘ └───────────┘  │ │  │  │
│  │  │  └───────────────────────────────────────────────────────┘ │  │  │
│  │  │  ┌───────────────────────────────────────────────────────┐ │  │  │
│  │  │  │                  PostgreSQL (future)                   │ │  │  │
│  │  │  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────────┐  │ │  │  │
│  │  │  │  │ Account │ │ Message │ │  Model  │ │TokenUsage │  │ │  │  │
│  │  │  │  │  Store  │ │  Store  │ │  Store  │ │   Store   │  │ │  │  │
│  │  │  │  └─────────┘ └─────────┘ └─────────┘ └───────────┘  │ │  │  │
│  │  │  └───────────────────────────────────────────────────────┘ │  │  │
│  │  └────────────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

## Migration System

The migration system provides Flyway/Liquibase-style schema versioning for both SQLite and PostgreSQL databases.

### Key Features

- **Versioned migrations**: Sequential integer versions with descriptive names
- **Up and down migrations**: Support for both forward migrations and rollbacks
- **DDL and data migrations**: SQL-based migrations for schema changes and data transformations
- **Checksum validation**: SHA-256 checksums detect unauthorized migration changes
- **Dialect-specific SQL**: Separate SQL files for SQLite and PostgreSQL
- **Embedded migrations**: SQL files compiled into the binary via `include_str!`

### Migration Structure

```
src/migrations/
├── mod.rs                    # MigrationRunner, types, error handling
├── registry.rs               # Migration registry for each dialect
├── v1_initial_schema.rs      # Rust module for v1 migration
└── sql/
    ├── sqlite/
    │   ├── v1_initial_schema.up.sql
    │   └── v1_initial_schema.down.sql
    └── postgres/
        ├── v1_initial_schema.up.sql
        └── v1_initial_schema.down.sql
```

### Migration Types

```rust
pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub up_sql: DialectSql,
    pub down_sql: Option<DialectSql>,
}

pub struct DialectSql {
    pub sqlite: &'static str,
    pub postgres: &'static str,
}
```

### Migration Runner

The `MigrationRunner` provides methods for each database backend:

```rust
impl MigrationRunner {
    pub async fn run_sqlite(pool: &SqlitePool, migrations: &[Migration]) -> Result<(), MigrationError>;
    pub async fn run_postgres(pool: &PgPool, migrations: &[Migration]) -> Result<(), MigrationError>;
    pub async fn down_sqlite(pool: &SqlitePool, migrations: &[Migration], target: i64) -> Result<(), MigrationError>;
    pub async fn down_postgres(pool: &PgPool, migrations: &[Migration], target: i64) -> Result<(), MigrationError>;
}
```

### Migration Execution Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    Migration Execution                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  1. Create _migrations table (if not exists)                     │
│     - version (BIGINT PRIMARY KEY)                               │
│     - name (TEXT NOT NULL)                                       │
│     - applied_at (BIGINT NOT NULL)                               │
│     - checksum (TEXT NOT NULL)                                   │
│     - dialect (TEXT NOT NULL)                                    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  2. Load applied migrations from _migrations                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  3. For each pending migration (sorted by version):              │
│     a. Validate checksum (fail hard if mismatch)                 │
│     b. Execute up_sql for the target dialect                     │
│     c. Insert record into _migrations                            │
│     d. Log success                                               │
└─────────────────────────────────────────────────────────────────┘
```

### Checksum Validation

Each migration computes a SHA-256 checksum of its SQL content:

```rust
impl Migration {
    pub fn checksum(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.up_sql.sqlite.as_bytes());
        hasher.update(b"|||");
        hasher.update(self.up_sql.postgres.as_bytes());
        hex::encode(hasher.finalize())
    }
}
```

If a previously-applied migration's SQL changes, the system fails with a `ChecksumMismatch` error, preventing accidental schema corruption.

## Store Traits

The storage layer defines async traits for each domain entity:

### AccountStore

```rust
#[async_trait]
pub trait AccountStore: Send + Sync {
    async fn get_account(&self, id: &str) -> Result<Option<Account>>;
    async fn list_accounts(&self) -> Result<Vec<Account>>;
    async fn upsert_account(&self, account: &Account) -> Result<()>;
    async fn delete_account(&self, id: &str) -> Result<()>;
    async fn get_active_account_id(&self) -> Result<Option<String>>;
    async fn set_active_account(&self, id: &str) -> Result<()>;
    async fn clear_active_account(&self) -> Result<()>;
    async fn set_active_workspace(&self, account_id: &str, workspace_id: &str) -> Result<()>;
}
```

### WorkspaceStore

```rust
#[async_trait]
pub trait WorkspaceStore: Send + Sync {
    async fn list_workspaces(&self, account_id: &str) -> Result<Vec<Workspace>>;
    async fn upsert_workspaces(&self, workspaces: &[Workspace]) -> Result<()>;
    async fn clear_workspaces(&self, account_id: &str) -> Result<()>;
}
```

### MessageStore

```rust
#[async_trait]
pub trait MessageStore: Send + Sync {
    async fn create_channel(&self, name: &str) -> Result<Channel>;
    async fn list_channels(&self) -> Result<Vec<Channel>>;
    async fn send_message(&self, channel_id: &str, author_id: &str, content: &str) -> Result<Message>;
    async fn get_messages(&self, channel_id: &str, limit: usize, before: Option<String>) -> Result<Vec<Message>>;
}
```

### ModelStore

```rust
#[async_trait]
pub trait ModelStore: Send + Sync {
    async fn list_providers(&self, account_id: &str) -> Result<Vec<Provider>>;
    async fn replace_providers(&self, account_id: &str, providers: Vec<Provider>) -> Result<()>;
    async fn delete_providers(&self, account_id: &str) -> Result<()>;
}
```

### TokenUsageStore

```rust
#[async_trait]
pub trait TokenUsageStore: Send + Sync {
    async fn upsert(&self, usage: &TokenUsage) -> Result<()>;
    async fn find_unpushed(&self, limit: usize) -> Result<Vec<TokenUsage>>;
    async fn count_unpushed(&self) -> Result<usize>;
    async fn mark_pushed(&self, ids: &[String]) -> Result<()>;
    async fn list_recent(&self, limit: usize, offset: usize, pushed: Option<bool>) -> Result<Vec<TokenUsage>>;
    async fn count_all(&self, pushed: Option<bool>) -> Result<usize>;
    async fn cleanup_old_pushed(&self, retention_days: i64) -> Result<usize>;
    async fn summary(&self, days: Option<i64>) -> Result<Vec<UsageSummary>>;
    async fn overview(&self, days: Option<i64>) -> Result<TokenUsageOverview>;
}
```

## Database Support

### SQLite (Current)

- **Driver**: `sqlx` with `sqlite` feature
- **Connection**: File-based or in-memory
- **Default path**: `~/.local/share/omc/data/omc.db`
- **Features**:
  - Embedded database (no server required)
  - WAL mode for concurrent reads
  - Full-text search support
  - JSON functions

### PostgreSQL (Future)

- **Driver**: `sqlx` with `postgres` feature
- **Connection**: TCP connection string
- **Features**:
  - Multi-user support
  - Advanced indexing
  - Full ACID compliance
  - Replication and clustering

## Configuration

### Database URL

The database backend is configured via `database_url` in the daemon configuration:

```json
{
  "daemon": {
    "database_url": "sqlite:/path/to/omc.db"
  }
}
```

Or via CLI:

```bash
omcd --database-url "postgres://user:pass@localhost/omc"
```

### URL Parsing

```rust
pub enum DatabaseUrl {
    Sqlite(String),
    Postgres(String),
}

impl DatabaseUrl {
    pub fn parse(url: &str) -> Self {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            DatabaseUrl::Postgres(url.to_string())
        } else {
            DatabaseUrl::Sqlite(url.to_string())
        }
    }
}
```

## Schema Design

### Current Schema (v1)

```sql
-- Migration tracking
CREATE TABLE _migrations (
    version BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at BIGINT NOT NULL,
    checksum TEXT NOT NULL,
    dialect TEXT NOT NULL
);

-- Chat channels
CREATE TABLE channel (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    topic TEXT,
    created_at BIGINT NOT NULL
);

-- Chat messages
CREATE TABLE message (
    id TEXT PRIMARY KEY,
    channel_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp BIGINT NOT NULL,
    edited_at BIGINT,
    reply_to TEXT
);
CREATE INDEX idx_message_channel_ts ON message(channel_id, timestamp);

-- User accounts
CREATE TABLE account (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    url TEXT NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    token_expiry BIGINT NOT NULL,
    active_workspace_id TEXT
);

-- Workspaces
CREATE TABLE workspace (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL,
    name TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL,  -- PostgreSQL
    -- is_admin INTEGER NOT NULL,  -- SQLite
    FOREIGN KEY (account_id) REFERENCES account(id)
);
CREATE INDEX idx_workspace_account ON workspace(account_id);

-- Active account (singleton)
CREATE TABLE active_account (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    account_id TEXT,
    FOREIGN KEY (account_id) REFERENCES account(id)
);

-- AI model providers
CREATE TABLE provider (
    id TEXT PRIMARY KEY,
    provider_id TEXT NOT NULL,
    name TEXT NOT NULL,
    env TEXT NOT NULL,
    api TEXT,
    npm TEXT,
    doc TEXT,
    models TEXT NOT NULL,
    account_id TEXT NOT NULL,
    last_fetched_at BIGINT NOT NULL,
    FOREIGN KEY (account_id) REFERENCES account(id)
);
CREATE INDEX idx_provider_account ON provider(account_id);

-- Token usage tracking
CREATE TABLE token_usage (
    id TEXT PRIMARY KEY,
    client TEXT NOT NULL,
    session_id TEXT NOT NULL,
    message_id TEXT NOT NULL UNIQUE,
    agent TEXT,
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    input_tokens BIGINT NOT NULL,
    output_tokens BIGINT NOT NULL,
    reasoning_tokens BIGINT NOT NULL,
    cache_read_tokens BIGINT NOT NULL,
    cache_write_tokens BIGINT NOT NULL,
    pushed BOOLEAN NOT NULL DEFAULT FALSE,  -- PostgreSQL
    -- pushed INTEGER NOT NULL DEFAULT 0,   -- SQLite
    recorded_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL
);
CREATE INDEX idx_token_usage_pushed ON token_usage(pushed, recorded_at);
```

### Dialect Differences

| Feature | SQLite | PostgreSQL |
|---------|--------|------------|
| Boolean | `INTEGER` (0/1) | `BOOLEAN` |
| Timestamps | `BIGINT` (Unix ms) | `BIGINT` (Unix ms) |
| Placeholders | `?` | `$1`, `$2`, ... |
| Upsert | `ON CONFLICT ... DO UPDATE` | `ON CONFLICT ... DO UPDATE` |
| JSON | `json()` functions | `jsonb` type |

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │AccountService│  │ModelService │  │  TokenUsageService      │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Async trait calls
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Storage Layer                            │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  Store Traits (AccountStore, WorkspaceStore, etc.)          ││
│  └─────────────────────────────────────────────────────────────┘│
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  Backend Implementations (SqliteStorage, PostgresStorage)   ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ sqlx queries
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Database Engine                          │
│  ┌──────────────────┐              ┌──────────────────────────┐ │
│  │     SQLite       │              │      PostgreSQL          │ │
│  │  (embedded)      │              │    (remote/local)        │ │
│  └──────────────────┘              └──────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Adding a New Migration

1. **Create SQL files**:

```bash
# For SQLite
src/migrations/sql/sqlite/v2_add_feature.up.sql
src/migrations/sql/sqlite/v2_add_feature.down.sql

# For PostgreSQL
src/migrations/sql/postgres/v2_add_feature.up.sql
src/migrations/sql/postgres/v2_add_feature.down.sql
```

2. **Create Rust module**:

```rust
// src/migrations/v2_add_feature.rs
use super::{DialectSql, Migration};

pub const MIGRATION: Migration = Migration {
    version: 2,
    name: "add_feature",
    up_sql: DialectSql {
        sqlite: include_str!("sql/sqlite/v2_add_feature.up.sql"),
        postgres: include_str!("sql/postgres/v2_add_feature.up.sql"),
    },
    down_sql: Some(DialectSql {
        sqlite: include_str!("sql/sqlite/v2_add_feature.down.sql"),
        postgres: include_str!("sql/postgres/v2_add_feature.down.sql"),
    }),
};
```

3. **Register in registry**:

```rust
// src/migrations/registry.rs
pub fn sqlite_migrations() -> Vec<Migration> {
    vec![
        v1_initial_schema::MIGRATION,
        v2_add_feature::MIGRATION,
    ]
}
```

## Testing

### SQLite Tests

SQLite tests run in-memory and are always executed:

```rust
#[tokio::test]
async fn test_sqlite_migrations_run_successfully() {
    let storage = SqliteStorage::new_memory().await.unwrap();
    let pool = storage.pool();
    
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM _migrations WHERE version = 1",
    )
    .fetch_optional(&*pool)
    .await
    .unwrap();
    
    assert!(row.is_some());
    assert_eq!(row.unwrap().0, 1);
}
```

### PostgreSQL Tests

PostgreSQL tests are gated by the `OMC_TEST_DATABASE_URL` environment variable:

```rust
#[tokio::test]
async fn test_postgres_migrations() {
    let url = match std::env::var("OMC_TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping PostgreSQL test: OMC_TEST_DATABASE_URL not set");
            return;
        }
    };
    
    let pool = PgPool::connect(&url).await.unwrap();
    MigrationRunner::run_postgres(&pool, &registry::postgres_migrations())
        .await
        .unwrap();
    
    // Verify schema...
}
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `sqlx` | 0.8 | Database driver (SQLite + PostgreSQL) |
| `sha2` | 0.10 | Checksum computation |
| `hex` | 0.4 | Hex encoding for checksums |
| `chrono` | 0.4 | Timestamp handling |
| `async-trait` | 0.1 | Async trait support |

## Future Enhancements

1. **PostgreSQL Store Implementations**: Complete `PostgresStorage` and all `Postgres*Store` types
2. **Connection Pooling**: Tune pool settings for production workloads
3. **Read Replicas**: Support read-only replicas for PostgreSQL
4. **Schema Introspection**: Tools to inspect and compare schemas
5. **Data Export/Import**: CLI commands for backup and restore
6. **Encryption at Rest**: Optional encryption for sensitive data
7. **Migration Hooks**: Pre/post migration callbacks for data transformations
8. **Dry Run Mode**: Preview migrations before applying
9. **Migration History**: Track rollback history and audit trail

## Security Considerations

1. **Migration Integrity**: Checksum validation prevents unauthorized schema changes
2. **SQL Injection**: All queries use parameterized statements via `sqlx`
3. **Connection Security**: PostgreSQL connections should use TLS in production
4. **Credential Storage**: Access tokens stored in plaintext (future: encryption)
5. **Backup Strategy**: Regular backups recommended for production deployments
