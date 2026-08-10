pub mod accounts;
pub mod messaging;
pub mod providers;
pub mod token_usage;
pub mod workspaces;

use omc_storage::backend::postgres::PgBackend;

pub async fn setup() -> Option<PgBackend> {
    let url = match std::env::var("OMC_TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping PostgreSQL test: OMC_TEST_DATABASE_URL not set");
            return None;
        }
    };

    match PgBackend::new(&url).await {
        Ok(backend) => Some(backend),
        Err(e) => {
            eprintln!("Skipping PostgreSQL test: failed to connect: {e}");
            None
        }
    }
}
