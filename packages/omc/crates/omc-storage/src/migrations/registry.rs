use super::{Migration, v1_initial_schema};

pub fn sqlite_migrations() -> Vec<Migration> {
    vec![v1_initial_schema::MIGRATION]
}

pub fn postgres_migrations() -> Vec<Migration> {
    vec![v1_initial_schema::MIGRATION]
}
