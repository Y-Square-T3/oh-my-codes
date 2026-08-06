use omc_storage::database_url::DatabaseUrl;

#[test]
fn parse_postgres_http() {
    let url = DatabaseUrl::parse("postgres://localhost/mydb");
    assert!(matches!(url, DatabaseUrl::Postgres(_)));
    assert_eq!(url.as_str(), "postgres://localhost/mydb");
}

#[test]
fn parse_postgresql_http() {
    let url = DatabaseUrl::parse("postgresql://localhost/mydb");
    assert!(matches!(url, DatabaseUrl::Postgres(_)));
    assert_eq!(url.as_str(), "postgresql://localhost/mydb");
}

#[test]
fn parse_sqlite_explicit() {
    let url = DatabaseUrl::parse("sqlite:/path/to/db.sqlite");
    assert!(matches!(url, DatabaseUrl::Sqlite(_)));
    assert_eq!(url.as_str(), "sqlite:/path/to/db.sqlite");
}

#[test]
fn parse_sqlite_implicit() {
    let url = DatabaseUrl::parse("/path/to/db.sqlite");
    assert!(matches!(url, DatabaseUrl::Sqlite(_)));
    assert_eq!(url.as_str(), "sqlite:/path/to/db.sqlite");
}

#[test]
fn parse_sqlite_memory() {
    let url = DatabaseUrl::parse(":memory:");
    assert!(matches!(url, DatabaseUrl::Sqlite(_)));
    assert_eq!(url.as_str(), "sqlite::memory:");
}

#[test]
fn is_sqlite() {
    let url = DatabaseUrl::parse("/path/to/db.sqlite");
    assert!(url.is_sqlite());
    assert!(!url.is_postgres());
}

#[test]
fn is_postgres() {
    let url = DatabaseUrl::parse("postgres://localhost/mydb");
    assert!(url.is_postgres());
    assert!(!url.is_sqlite());
}

#[test]
fn as_str_sqlite() {
    let url = DatabaseUrl::parse("sqlite:/path/to/db.sqlite");
    assert_eq!(url.as_str(), "sqlite:/path/to/db.sqlite");
}

#[test]
fn as_str_postgres() {
    let url = DatabaseUrl::parse("postgres://localhost/mydb");
    assert_eq!(url.as_str(), "postgres://localhost/mydb");
}

#[test]
fn derives_clone() {
    let url = DatabaseUrl::parse("postgres://localhost/mydb");
    let cloned = url.clone();
    assert_eq!(url, cloned);
}

#[test]
fn derives_debug() {
    let url = DatabaseUrl::parse("postgres://localhost/mydb");
    let s = format!("{:?}", url);
    assert!(s.contains("Postgres"));
}

#[test]
fn derives_partial_eq() {
    let u1 = DatabaseUrl::parse("postgres://localhost/mydb");
    let u2 = DatabaseUrl::parse("postgres://localhost/mydb");
    assert_eq!(u1, u2);
}

#[test]
fn parses_postgres_with_query_params() {
    let url = DatabaseUrl::parse("postgres://user:pass@localhost:5432/db?sslmode=require");
    assert!(url.is_postgres());
    assert_eq!(url.as_str(), "postgres://user:pass@localhost:5432/db?sslmode=require");
}
