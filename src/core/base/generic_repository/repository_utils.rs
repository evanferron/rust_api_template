use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{ Database, Pool, Transaction};
use uuid::Uuid;
use crate::core::base::bind_value::BindValue;
use crate::core::base::generic_repository::entry_trait::Entry;
use crate::core::base::query_builder::query_builder::QueryBuilder;
use crate::core::errors::errors::ApiError;

pub fn bind_entry_to_query<'a, DB, T>(
    mut query_builder: QueryBuilder<DB,T>,
    entry: &T,
) -> QueryBuilder<DB,T>
where
    T: Entry<DB> + Send + Sync + Unpin + 'static + for<'r> sqlx::FromRow<'r, <DB as Database>::Row>,
    DB: Database,
    for<'q> <DB as Database>::Arguments<'q>: sqlx::IntoArguments<'q, DB>,
    for<'c> &'c mut <DB as Database>::Connection: sqlx::Executor<'c, Database = DB>,
    for<'q> i32: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> i64: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> f64: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> bool: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> String: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> i32: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> Uuid: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> Value: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> DateTime<Utc>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    for<'q> Option<String>: sqlx::Encode<'q, DB> + sqlx::Type<DB>, str: sqlx::Type<DB>,
{
    let values = entry.to_bind_values();
    for bind_value in values {
        query_builder = query_builder.add_param(bind_value);
    }
    query_builder
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