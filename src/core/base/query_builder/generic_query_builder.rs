use sqlx::{Database, Pool, FromRow};
use std::marker::PhantomData;
use crate::core::base::query_builder::parameterizedQuery::ParameterizedQuery;

// Enum to handle differences between DBs
#[derive(Clone, Copy, Debug)]
pub enum DbType {
    Postgres,
    MySQL,
    SQLite,
}

impl DbType {
    // Handle placeholders ($1, $2 vs ?, ?)
    pub fn placeholder(&self, index: usize) -> String {
        match self {
            Self::Postgres => format!("${}", index),
            Self::MySQL | Self::SQLite => "?".to_string(),
        }
    }
}

// Simple QueryBuilder for raw SQL
pub struct QueryBuilder<DB: Database> {
    db_type: DbType,
    pool: Pool<DB>,
    sql: String,
    param_count: usize,
    _phantom: PhantomData<DB>,
}

impl<DB> QueryBuilder<DB>
where
    DB: Database,
    for<'q> <DB as Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
{
    pub fn new(pool: Pool<DB>, db_type: DbType) -> Self {
        Self {
            db_type,
            pool,
            sql: String::new(),
            param_count: 0,
            _phantom: PhantomData,
        }
    }

    // Set the full SQL
    pub fn sql(mut self, sql: impl Into<String>) -> Self {
        self.sql = sql.into();
        self
    }

    // Append SQL
    pub fn append(mut self, sql: &str) -> Self {
        if !self.sql.is_empty() && !self.sql.ends_with(' ') {
            self.sql.push(' ');
        }
        self.sql.push_str(sql);
        self
    }

    // Return the next placeholder and increment the counter
    pub fn placeholder(&mut self) -> String {
        self.param_count += 1;
        self.db_type.placeholder(self.param_count)
    }

    // Get the generated SQL
    pub fn get_sql(&self) -> &str {
        &self.sql
    }

    // Get the parameter count
    pub fn param_count(&self) -> usize {
        self.param_count
    }

    // Get a reference to the pool
    pub fn pool(&self) -> &Pool<DB> {
        &self.pool
    }

    // Clone the pool
    pub fn pool_clone(&self) -> Pool<DB> {
        self.pool.clone()
    }

    // Decompose the builder into SQL and Pool
    pub fn build(self) -> (String, Pool<DB>) {
        (self.sql, self.pool)
    }

    // Helper: execute a simple query without parameters
    // Returns the raw QueryResult; the caller can call .rows_affected() on it
    pub async fn execute_simple(self) -> Result<DB::QueryResult, sqlx::Error> {
        sqlx::query(&self.sql)
            .execute(&self.pool)
            .await
    }

    // Helper: fetch_all without parameters
    pub async fn fetch_all_simple<T>(self) -> Result<Vec<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        let result = sqlx::query_as::<_, T>(&self.sql)
            .fetch_all(&self.pool)
            .await?;
        Ok(result)
    }

    // Helper: fetch_one without parameters
    pub async fn fetch_one_simple<T>(self) -> Result<T, sqlx::Error>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        let result = sqlx::query_as::<_, T>(&self.sql)
            .fetch_one(&self.pool)
            .await?;
        Ok(result)
    }

    // Helper: fetch_optional without parameters
    pub async fn fetch_optional_simple<T>(self) -> Result<Option<T>, sqlx::Error>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        let result = sqlx::query_as::<_, T>(&self.sql)
            .fetch_optional(&self.pool)
            .await?;
        Ok(result)
    }

    // Create a BoundQuery to bind parameters fluently
    pub fn prepare(&self) -> ParameterizedQuery<'_, DB> {
        ParameterizedQuery::new(&self.sql, &self.pool)
    }
}

// Type aliases pour chaque DB
#[cfg(feature = "postgres")]
pub type PgQueryBuilder = QueryBuilder<sqlx::Postgres>;

#[cfg(feature = "mysql")]
pub type MySqlQueryBuilder = QueryBuilder<sqlx::MySql>;

#[cfg(feature = "sqlite")]
pub type SqliteQueryBuilder = QueryBuilder<sqlx::Sqlite>;

// Helpers to create builders
#[cfg(feature = "postgres")]
pub async fn create_pg_builder(database_url: &str) -> Result<PgQueryBuilder, sqlx::Error> {
    let pool = sqlx::postgres::PgPool::connect(database_url).await?;
    Ok(QueryBuilder::new(pool, DbType::Postgres))
}

#[cfg(feature = "mysql")]
pub async fn create_mysql_builder(database_url: &str) -> Result<MySqlQueryBuilder, sqlx::Error> {
    let pool = sqlx::mysql::MySqlPool::connect(database_url).await?;
    Ok(QueryBuilder::new(pool, DbType::MySQL))
}

#[cfg(feature = "sqlite")]
pub async fn create_sqlite_builder(database_url: &str) -> Result<SqliteQueryBuilder, sqlx::Error> {
    let pool = sqlx::sqlite::SqlitePool::connect(database_url).await?;
    Ok(QueryBuilder::new(pool, DbType::SQLite))
}
