use super::{Migration, v1_initial_schema, v2_token_usage_schema};

pub fn sqlite_migrations() -> Vec<Migration> {
    vec![
        v1_initial_schema::MIGRATION,
        v2_token_usage_schema::MIGRATION,
    ]
}

pub fn postgres_migrations() -> Vec<Migration> {
    vec![
        v1_initial_schema::MIGRATION,
        v2_token_usage_schema::MIGRATION,
    ]
}
