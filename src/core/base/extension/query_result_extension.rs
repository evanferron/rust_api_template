// Dans un fichier utils ou extensions
pub trait QueryResultExt {
    fn rows_affected(&self) -> u64;
}

// Implémente pour Postgres
#[cfg(feature = "postgres")]
impl QueryResultExt for sqlx::postgres::PgQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}

// Implémente pour MySQL
#[cfg(feature = "mysql")]
impl QueryResultExt for sqlx::mysql::MySqlQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}

// Implémente pour SQLite
#[cfg(feature = "sqlite")]
impl QueryResultExt for sqlx::sqlite::SqliteQueryResult {
    fn rows_affected(&self) -> u64 {
        self.rows_affected()
    }
}