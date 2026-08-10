use super::{Migration, v1_initial_schema, v2_token_usage_schema, v3_token_cost_table};

pub fn sqlite_migrations() -> Vec<Migration> {
    vec![
        v1_initial_schema::MIGRATION,
        v2_token_usage_schema::MIGRATION,
        v3_token_cost_table::MIGRATION,
    ]
}

pub fn postgres_migrations() -> Vec<Migration> {
    vec![
        v1_initial_schema::MIGRATION,
        v2_token_usage_schema::MIGRATION,
        v3_token_cost_table::MIGRATION,
    ]
}
