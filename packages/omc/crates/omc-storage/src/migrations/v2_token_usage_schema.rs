use super::{DialectSql, Migration};

pub const MIGRATION: Migration = Migration {
    version: 2,
    name: "token_usage_schema",
    up_sql: DialectSql {
        sqlite: include_str!("sql/sqlite/v2_token_usage_schema.up.sql"),
        postgres: include_str!("sql/postgres/v2_token_usage_schema.up.sql"),
    },
    down_sql: Some(DialectSql {
        sqlite: include_str!("sql/sqlite/v2_token_usage_schema.down.sql"),
        postgres: include_str!("sql/postgres/v2_token_usage_schema.down.sql"),
    }),
};
