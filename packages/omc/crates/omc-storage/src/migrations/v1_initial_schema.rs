use super::{DialectSql, Migration};

pub const MIGRATION: Migration = Migration {
    version: 1,
    name: "initial_schema",
    up_sql: DialectSql {
        sqlite: include_str!("sql/sqlite/v1_initial_schema.up.sql"),
        postgres: include_str!("sql/postgres/v1_initial_schema.up.sql"),
    },
    down_sql: Some(DialectSql {
        sqlite: include_str!("sql/sqlite/v1_initial_schema.down.sql"),
        postgres: include_str!("sql/postgres/v1_initial_schema.down.sql"),
    }),
};
