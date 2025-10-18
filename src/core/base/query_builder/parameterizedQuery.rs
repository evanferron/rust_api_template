use crate::core::base::query_builder::query_models::QueryResult;
use crate::core::errors::errors::ApiError;
use sqlx::{Database, FromRow, Pool};

// Structure to manage bound parameters
pub struct ParameterizedQuery<'a, DB: Database> {
    sql: &'a str,
    query: sqlx::query::Query<'a, DB, <DB as Database>::Arguments<'a>>,
}

impl<'a, DB> ParameterizedQuery<'a, DB>
where
    DB: Database,
    for<'q> <DB as Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
{
    pub(crate) fn new(sql: &'a str) -> Self {
        Self {
            sql,
            query: sqlx::query(sql),
        }
    }

    // Bind a parameter
    pub fn bind<T>(mut self, value: T) -> Self
    where
        T: 'a + Send + sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    {
        self.query = self.query.bind(value);
        self
    }

    // Execute the query
    pub async fn execute(self, pool: &Pool<DB>) -> QueryResult<DB::QueryResult> {
        self.query.execute(pool).await.map_err(ApiError::from)
    }

    // Fetch all with typed results
    pub async fn fetch_all<T>(self, pool: &Pool<DB>) -> QueryResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(self.sql)
            .fetch_all(pool)
            .await
            .map_err(ApiError::from)
    }

    // Fetch one
    pub async fn fetch_one<T>(self, pool: &Pool<DB>) -> QueryResult<T>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(self.sql)
            .fetch_one(pool)
            .await
            .map_err(ApiError::from)
    }

    // Fetch optional
    pub async fn fetch_optional<T>(self, pool: &Pool<DB>) -> QueryResult<Option<T>>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        sqlx::query_as::<_, T>(self.sql)
            .fetch_optional(pool)
            .await
            .map_err(ApiError::from)
    }
}
