pub mod builders;

use omc_storage::backend::sqlite::SqliteBackend;

pub async fn setup() -> SqliteBackend {
    SqliteBackend::new_memory().await.unwrap()
}
