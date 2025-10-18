use crate::core::base::query_builder::query_models::QueryResult;
use crate::core::errors::errors::ApiError;
use sqlx::{Database, FromRow, Pool, Transaction};

// Structure to manage bound parameters
pub struct QueryExecutor<'a, DB: Database> {
    sql: &'a str,
    query: sqlx::query::Query<'a, DB, <DB as Database>::Arguments<'a>>,
}

impl<'a, DB> QueryExecutor<'a, DB>
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

    // Execute within a transaction
    pub async fn execute_with_transaction<'tx>(
        self,
        tx: &mut Transaction<'tx, DB>,
    ) -> QueryResult<DB::QueryResult>
    where
            for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
    {
        self.query.execute(tx).await.map_err(ApiError::from)
    }

    // Fetch all with typed results
    pub async fn fetch_all<T>(self, pool: &Pool<DB>) -> QueryResult<Vec<T>>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {

        let rows = self.query.fetch_all(pool).await.map_err(ApiError::from)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let item = T::from_row(&row).map_err(ApiError::from)?;
            out.push(item);
        }
        Ok(out)
    }

    pub async fn fetch_all_with_transaction<'tx,T>(
        self,
        tx: &mut Transaction<'tx, DB>,
    ) -> QueryResult<Vec<T>>
    where
            for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
            T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        let rows = self.query.fetch_all(tx).await.map_err(ApiError::from)?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let item = T::from_row(&row).map_err(ApiError::from)?;
            out.push(item);
        }
        Ok(out)
    }


    // Fetch one
    pub async fn fetch_one<T>(self, pool: &Pool<DB>) -> QueryResult<T>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        let row = self.query.fetch_one(pool).await.map_err(ApiError::from)?;
        T::from_row(&row).map_err(ApiError::from)
    }

    pub async fn fetch_one_with_transaction<'tx,T>(
        self,
        tx: &mut Transaction<'tx, DB>,
    ) -> QueryResult<T>
    where
            for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
            T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        let row = self.query.fetch_one(tx).await.map_err(ApiError::from)?;
        T::from_row(&row).map_err(ApiError::from)
    }

    // Fetch optional
    pub async fn fetch_optional<T>(self, pool: &Pool<DB>) -> QueryResult<Option<T>>
    where
        T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {

        let opt_row = self.query.fetch_optional(pool).await.map_err(ApiError::from)?;
        match opt_row {
            Some(row) => {
                let item = T::from_row(&row).map_err(ApiError::from)?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }

    pub async fn fetch_optional_with_transaction<'tx,T>(
        self,
        tx: &mut Transaction<'tx, DB>,
    ) -> QueryResult<Option<T>>
    where
            for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
            T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    {
        let opt_row = self.query.fetch_optional(tx).await.map_err(ApiError::from)?;
        match opt_row {
            Some(row) => {
                let item = T::from_row(&row).map_err(ApiError::from)?;
                Ok(Some(item))
            }
            None => Ok(None),
        }
    }
}
