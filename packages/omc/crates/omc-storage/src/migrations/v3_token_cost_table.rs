use super::{DialectSql, Migration};

pub const MIGRATION: Migration = Migration {
    version: 3,
    name: "token_cost_table",
    up_sql: DialectSql {
        sqlite: include_str!("sql/sqlite/v3_token_cost_table.up.sql"),
        postgres: include_str!("sql/postgres/v3_token_cost_table.up.sql"),
    },
    down_sql: Some(DialectSql {
        sqlite: include_str!("sql/sqlite/v3_token_cost_table.down.sql"),
        postgres: include_str!("sql/postgres/v3_token_cost_table.down.sql"),
    }),
};
