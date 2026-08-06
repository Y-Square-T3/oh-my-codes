#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseUrl {
    Sqlite(String),
    Postgres(String),
}

impl DatabaseUrl {
    pub fn parse(url: &str) -> Self {
        if url.starts_with("postgres://") || url.starts_with("postgresql://") {
            DatabaseUrl::Postgres(url.to_string())
        } else {
            let sqlite_url = if url.starts_with("sqlite:") {
                url.to_string()
            } else {
                format!("sqlite:{}", url)
            };
            DatabaseUrl::Sqlite(sqlite_url)
        }
    }

    pub fn is_sqlite(&self) -> bool {
        matches!(self, DatabaseUrl::Sqlite(_))
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self, DatabaseUrl::Postgres(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            DatabaseUrl::Sqlite(s) => s,
            DatabaseUrl::Postgres(s) => s,
        }
    }
}
