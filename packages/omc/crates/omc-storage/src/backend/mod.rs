pub mod sqlite;

#[cfg(feature = "postgres")]
pub mod postgres;

use crate::database_url::DatabaseUrl;
use crate::traits::StorageBackend;
#[cfg(not(feature = "postgres"))]
use omc_core::error::OmcError;
use omc_core::error::Result;
use std::sync::Arc;

pub async fn create_backend(url: &DatabaseUrl) -> Result<Arc<dyn StorageBackend>> {
    match url {
        DatabaseUrl::Sqlite(url_str) => {
            let path = url_str.strip_prefix("sqlite:").unwrap_or(url_str);
            let backend = sqlite::SqliteBackend::new(std::path::Path::new(path)).await?;
            Ok(Arc::new(backend))
        }
        DatabaseUrl::Postgres(url_str) => {
            #[cfg(feature = "postgres")]
            {
                let backend = postgres::PgBackend::new(url_str).await?;
                Ok(Arc::new(backend))
            }
            #[cfg(not(feature = "postgres"))]
            {
                let _ = url_str;
                Err(OmcError::Storage(
                    "PostgreSQL backend is not enabled. Rebuild with --features postgres".into(),
                ))
            }
        }
    }
}
