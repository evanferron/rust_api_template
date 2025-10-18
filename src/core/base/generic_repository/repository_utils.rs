use serde_json::Value;
use sqlx::{Database, Pool, Transaction};
use sqlx::types::JsonValue;
use crate::core::base::generic_repository::entry_trait::{BindValue, Entry};
use crate::core::base::query_builder::parameterized_query::QueryExecutor;
use crate::core::errors::errors::ApiError;

pub fn bind_entry_to_query<'a, DB, T>(
    mut query_builder: QueryExecutor<'a, DB>,
    entry: &T,
) -> QueryExecutor<'a,DB>
where
    T: Entry<DB> + Send + Sync + Unpin + 'static + for<'r> sqlx::FromRow<'r, <DB as Database>::Row>,
    DB: Database,
    for<'q> <DB as Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
    bool: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    i64: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    f64: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    sqlx::types::Json<Value>: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    Option<sqlx::types::Json<JsonValue>>: sqlx::Encode<'a, DB>,
{
    for bind_value in entry.to_bind_values() {
        query_builder = bind_value_to_query(query_builder, &bind_value);
    }
    query_builder
}

pub fn bind_value_to_query<'a, DB>(
    query_builder: QueryExecutor<'a, DB>,
    bind_value: &BindValue,
) -> QueryExecutor<'a,DB>
where
    DB: Database,
    for<'q> <DB as Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
    bool: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    i64: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    f64: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    sqlx::types::Json<Value>: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    Option<sqlx::types::Json<JsonValue>>: sqlx::Encode<'a, DB>,
{
    match bind_value {
        BindValue::Null => query_builder.bind(Option::<sqlx::types::Json<Value>>::None),
        BindValue::Bool(v) => query_builder.bind(*v),
        BindValue::Int(v) => query_builder.bind(*v),
        BindValue::Float(v) => query_builder.bind(*v),
        BindValue::String(v) => query_builder.bind(v.clone()),
        BindValue::Json(v) => query_builder.bind(sqlx::types::Json(v.clone())),
    }
}

pub async fn execute_transaction<F, Fut, R, DB, C>(
    pool: &Pool<DB>,
    context: C,
    f: F
) -> Result<R, ApiError>
where
    F: for<'tx> FnOnce(C, &'tx mut Transaction<'tx, DB>) -> Fut + Send,
    Fut: Future<Output = Result<R, ApiError>> + Send + 'static,
    DB: Database,
    C: Send,
    for<'c> &'c mut Transaction<'c, DB>: sqlx::Executor<'c, Database = DB>,
{
    let mut tx = pool.begin().await.map_err(ApiError::from)?;

    let result = unsafe {
        let fut = f(context, &mut *(&mut tx as *mut _));
        fut.await
    };

    match result {
        Ok(res) => {
            tx.commit().await.map_err(ApiError::from)?;
            Ok(res)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}