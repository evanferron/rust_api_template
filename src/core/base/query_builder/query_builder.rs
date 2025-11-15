use crate::core::base::query_builder::query_models::QueryResult;
use crate::core::errors::errors::ApiError;
use sqlx::{Database, FromRow, Pool, Transaction};
use std::marker::PhantomData;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::query::{Query, QueryAs};
use uuid::Uuid;
use crate::core::base::bind_value::BindValue;

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
pub struct QueryBuilder<DB: Database, T: for<'r> FromRow<'r, DB::Row> + Send + Unpin> {
    db_type: DbType,
    sql: String,
    param_count: usize,
    params: Vec<BindValue>,
    _phantom: PhantomData<DB>,
    _phantom_type: PhantomData<T>,
}

impl<DB, T> QueryBuilder<DB, T>
where
    DB: Database,
    T: for<'r> FromRow<'r, DB::Row> + Send + Unpin,
    for<'q> <DB as Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
    for<'q> i32: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> i64: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> f64: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> bool: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> String: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> Uuid: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> Value: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> DateTime<Utc>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    for<'q> Option<String>: sqlx::Encode<'q, DB> + sqlx::Type<DB>, str: sqlx::Type<DB>
{
    pub fn new(db_type: DbType) -> Self {
        Self {
            db_type,
            sql: String::new(),
            param_count: 0,
            params: Vec::new(),
            _phantom: PhantomData,
            _phantom_type: PhantomData,
        }
    }

    // Set the full SQL - pattern mutable
    pub fn set_sql(mut self, sql: impl Into<String>) -> Self {
        self.sql = sql.into();
        self
    }

    // Append SQL
    pub fn append(&mut self, sql: &str) -> &mut Self {
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
    pub async fn execute_simple(&self, pool: &Pool<DB>) -> QueryResult<DB::QueryResult> {
        sqlx::query::<DB>(&self.sql)
            .execute(pool)
            .await
            .map_err(ApiError::from)
    }

    // Helper: fetch_all without parameters
    pub async fn fetch_all_simple(&self, pool: &Pool<DB>) -> QueryResult<Vec<T>>
    {
        sqlx::query_as::<DB, T>(&self.sql)
            .fetch_all(pool)
            .await
            .map_err(ApiError::from)
    }

    // Helper: fetch_one without parameters
    pub async fn fetch_one_simple(&self, pool: &Pool<DB>) -> QueryResult<T>
    {
        sqlx::query_as::<DB, T>(&self.sql)
            .fetch_one(pool)
            .await
            .map_err(ApiError::from)
    }

    // Helper: fetch_optional without parameters
    pub async fn fetch_optional_simple(&self, pool: &Pool<DB>) -> QueryResult<Option<T>>
    {
        sqlx::query_as::<DB, T>(&self.sql)
            .fetch_optional(pool)
            .await
            .map_err(ApiError::from)
    }

    pub fn add_param(mut self, value: BindValue) -> Self
    {
        self.param_count += 1;
        self.params.push(value);
        self
    }

    pub fn add_params(mut self, values: Vec<BindValue>) -> Self
    {
        for value in values {
            self = self.add_param(value);
        }
        self
    }

    fn build_execute(&self) -> Query<'_, DB, DB::Arguments<'_>> {
        let mut q = sqlx::query::<DB>(&self.sql);

        for v in &self.params {
            q = match v {
                BindValue::I32(v) => q.bind(*v),
                BindValue::I64(v) => q.bind(*v),
                BindValue::F64(v) => q.bind(*v),
                BindValue::Bool(v) => q.bind(*v),
                BindValue::String(v) => q.bind(v),
                BindValue::Uuid(v) => q.bind(*v),
                BindValue::Json(v) => q.bind(v.clone()),
                BindValue::DateTime(v) => q.bind(*v),
                BindValue::Null => q.bind(Option::<String>::None),
            }
        }

        q
    }
    fn build_query<'q>(&self) -> QueryAs<'_, DB, T, DB::Arguments<'_>>{
        let mut q = sqlx::query_as::<DB,T>(&self.sql);
        for v in &self.params {
            q = match v {
                BindValue::I32(v) => q.bind(*v),
                BindValue::I64(v) => q.bind(*v),
                BindValue::F64(v) => q.bind(*v),
                BindValue::Bool(v) => q.bind(*v),
                BindValue::String(v) => q.bind(v),
                BindValue::Uuid(v) => q.bind(*v),
                BindValue::Json(v) => q.bind(v.clone()),
                BindValue::DateTime(v) => q.bind(*v),
                BindValue::Null => q.bind(Option::<String>::None),
            }
        }
        q
    }

    // Execute the query
    pub async fn execute(self, pool: &Pool<DB>) -> QueryResult<DB::QueryResult> {
        self.build_execute()
            .execute(pool)
            .await
            .map_err(ApiError::from)
    }

    // Execute within a transaction
    pub async fn execute_with_transaction<'tx>(
        self,
        tx: &mut Transaction<'tx, DB>,
    ) -> QueryResult<DB::QueryResult>
    where
        for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
    {
        self.build_execute()
            .execute(tx)
            .await
            .map_err(ApiError::from)
    }

    // Fetch all with typed results
    pub async fn fetch_all(self, pool: &Pool<DB>) -> QueryResult<Vec<T>>
    {
        self.build_query()
            .fetch_all(pool)
            .await
            .map_err(ApiError::from)
    }

    pub async fn fetch_all_with_transaction<'tx>(
        self,
        tx: &mut Transaction<'tx, DB>,
    ) -> QueryResult<Vec<T>>
    where
        for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
    {
        self.build_query()
            .fetch_all(tx)
            .await
            .map_err(ApiError::from)
    }

    // Fetch one
    pub async fn fetch_one(self, pool: &Pool<DB>) -> QueryResult<T>
    {
        self.build_query()
            .fetch_one(pool)
            .await
            .map_err(ApiError::from)
    }

    pub async fn fetch_one_with_transaction<'tx>(
        self,
        tx: &mut Transaction<'tx, DB>,
    ) -> QueryResult<T>
    where
        for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
    {
        self.build_query()
            .fetch_one(tx)
            .await
            .map_err(ApiError::from)
    }

    // Fetch optional
    pub async fn fetch_optional(self, pool: &Pool<DB>) -> QueryResult<Option<T>>
    {
        self.build_query()
            .fetch_optional(pool)
            .await
            .map_err(ApiError::from)
    }

    pub async fn fetch_optional_with_transaction<'tx>(
        self,
        tx: &mut Transaction<'tx, DB>,
    ) -> QueryResult<Option<T>>
    where
        for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
    {
        self.build_query()
            .fetch_optional(tx)
            .await
            .map_err(ApiError::from)
    }
}
