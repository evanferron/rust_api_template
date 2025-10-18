use crate::core::base::generic_repository::entry_trait::Entry;
use crate::core::base::query_builder::parameterized_query::QueryExecutor;
use crate::core::base::query_builder::query_models::QueryResult;
use crate::core::errors::errors::ApiError;
use sqlx::{Database, FromRow, Pool, Transaction};
use std::marker::PhantomData;

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
pub struct QueryBuilder<DB: Database, T: Entry<DB> + Send + Sync + Unpin + 'static> {
    db_type: DbType,
    sql: String,
    param_count: usize,
    _phantom: PhantomData<DB>,
    _phantom_type: PhantomData<T>,
}

impl<DB, T> QueryBuilder<DB, T>
where
    DB: Database,
    T: Entry<DB> + Send + Sync + Unpin + 'static,
    for<'q> <DB as Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
{
    pub fn new(db_type: DbType) -> Self {
        Self {
            db_type,
            sql: String::new(),
            param_count: 0,
            _phantom: PhantomData,
            _phantom_type: PhantomData,
        }
    }

    // Set the full SQL
    pub fn set_sql(mut self, sql: impl Into<String>) -> Self {
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

    // Helper: execute a simple query without parameters
    // Returns the raw QueryResult; the caller can call .rows_affected() on it
    pub async fn execute_simple(self, pool: &Pool<DB>) -> QueryResult<DB::QueryResult> {
        sqlx::query(&self.sql)
            .execute(pool)
            .await
            .map_err(ApiError::from)
    }

    // Helper: fetch_all without parameters
    pub async fn fetch_all_simple(self, pool: &Pool<DB>) -> QueryResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(&self.sql)
            .fetch_all(pool)
            .await
            .map_err(ApiError::from)
    }

    // Helper: fetch_one without parameters
    pub async fn fetch_one_simple(self, pool: &Pool<DB>) -> QueryResult<T>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(&self.sql)
            .fetch_one(pool)
            .await
            .map_err(ApiError::from)
    }

    // Helper: fetch_optional without parameters
    pub async fn fetch_optional_simple(self, pool: &Pool<DB>) -> QueryResult<Option<T>>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(&self.sql)
            .fetch_optional(pool)
            .await
            .map_err(ApiError::from)
    }

    // Create a BoundQuery to bind parameters fluently
    pub fn prepare(&self) -> QueryExecutor<'_, DB> {
        QueryExecutor::new(&self.sql)
    }
}
