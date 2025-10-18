use crate::core::{
    base::query_builder::{generic_query_builder::QueryBuilder, query_models::OrderDirection},
    errors::errors::ApiError,
};

use super::entry_trait::{BindValue, Entry};
use crate::core::base::generic_repository::repository_utils::{
    bind_entry_to_query, bind_value_to_query, execute_transaction,
};
use chrono::Utc;
use serde_json::Value;
use sqlx::types::JsonValue;
use sqlx::{Database, Pool, Transaction};
use crate::core::base::extension::query_result_extension::QueryResultExt;

pub type RepositoryResult<T> = Result<T, ApiError>;

pub trait RepositoryTrait<T, DB>
where
    T: Entry<DB> + Send + Sync + Unpin + 'static + for<'r> sqlx::FromRow<'r, <DB as Database>::Row>,
    DB: Database,
    DB::QueryResult: QueryResultExt,
    for<'a> <DB as Database>::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    for<'a> &'a mut <DB as Database>::Connection: sqlx::Executor<'a, Database = DB>,
    for<'a> bool: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    for<'a> i64: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    for<'a> f64: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    for<'a> String: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    for<'a> sqlx::types::Json<Value>: sqlx::Encode<'a, DB> + sqlx::Type<DB>,
    for<'a> Option<sqlx::types::Json<JsonValue>>: sqlx::Encode<'a, DB>,
{
    /// Returns a reference to the Postgres connection pool.
    fn get_pool(&self) -> &Pool<DB>;

    /// Creates a new QueryBuilder instance for building queries.
    fn query(&self) -> QueryBuilder<DB, T>;

    /// Fetches all records of type T from the database.
    async fn find_all(&self) -> RepositoryResult<Vec<T>> {
        let sql = format!(
            "SELECT {} FROM {}",
            T::columns().join(", "),
            T::table_name()
        );

        let qb = self.query().set_sql(&sql);

        Ok(qb.fetch_all_simple(self.get_pool()).await?)
    }

    /// Finds a record by its primary key (id). Returns an Option<T>.
    async fn find_by_id(&self, id: T::Id) -> RepositoryResult<T> {
        let mut qb = self.query();

        let sql = format!(
            "SELECT {} FROM {} WHERE id = {}",
            T::columns_to_string(),
            T::table_name(),
            qb.placeholder()
        );

        qb.set_sql(&sql)
            .prepare()
            .bind(id)
            .fetch_one(self.get_pool())
            .await
    }

    /// Finds records by a specific column and value.
    async fn find_by_column<V>(&self, column: &str, value: V) -> RepositoryResult<Vec<T>>
    where
        V: Send + Sync + serde::Serialize + sqlx::Encode<'static, DB> + sqlx::Type<DB>,
    {
        let mut qb = self.query();

        let sql = format!(
            "SELECT {} FROM {} WHERE {} = {}",
            T::columns_to_string(),
            T::table_name(),
            column,
            qb.placeholder()
        );

        qb.set_sql(&sql)
            .prepare()
            .bind(value)
            .fetch_all(self.get_pool())
            .await
    }

    /// Finds records matching a set of criteria (column, value pairs).
    async fn find_by_columns<V>(&self, columns: &[&str], values: &[V]) -> RepositoryResult<Vec<T>>
    where
        V: Send + Sync + serde::Serialize + sqlx::Encode<'static, DB> + sqlx::Type<DB>,
    {
        if columns.is_empty() || values.is_empty() {
            return self.find_all().await;
        } else if columns.len() != values.len() {
            return Err(ApiError::InternalServer(
                "Columns and values length mismatch".to_string(),
            ));
        }
        let mut qb = self.query();
        let mut sql = format!(
            "SELECT {} FROM {} WHERE ",
            T::columns_to_string(),
            T::table_name()
        );
        for (i, column) in columns.iter().enumerate() {
            sql.push_str(&format!("{} = {}", column, qb.placeholder()));
            if i < columns.len() - 1 {
                sql.push_str(" AND ");
            }
        }
        let mut executor = qb.set_sql(&sql).prepare();
        for value in values.iter() {
            executor = executor.bind(value.clone());
        }
        executor.fetch_all(self.get_pool()).await
    }

    /// Counts the total number of records of type T.
    async fn count(&self) -> RepositoryResult<i64> {
        let sql = format!("SELECT COUNT(*) as count FROM {}", T::table_name());
        let res = self
            .query()
            .set_sql(&sql)
            .prepare()
            .fetch_one(self.get_pool())
            .await;
        match res {
            Ok(row) => {
                let count: i64 = row
                    .get("count")
                    .ok_or_else(|| ApiError::InternalServer("Cannot count records".to_string()))?;
                Ok(count)
            }
            Err(e) => Err(e),
        }
    }

    /// Fetches a paginated list of records, ordered by id ascending.
    async fn paginate(&self, page: u32, page_size: u32) -> RepositoryResult<Vec<T>> {
        let offset = (page - 1) * page_size;
        let sql = format!(
            "SELECT {} FROM {} ORDER BY id ASC LIMIT {} OFFSET {}",
            T::columns_to_string(),
            T::table_name(),
            page_size,
            offset
        );

        self.query()
            .set_sql(&sql)
            .fetch_all_simple(self.get_pool())
            .await
    }

    /// Creates a new record in the database and returns it.
    async fn create(&self, mut entry: T) -> RepositoryResult<T> {
        let now = Utc::now();
        entry.set_created_at(now);
        entry.set_updated_at(now);

        let nb_columns = T::insertable_columns().len();
        let mut qb = self.query();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
            T::table_name(),
            T::columns_to_string(),
            (0..nb_columns)
                .map(|i| { qb.placeholder() })
                .collect::<Vec<String>>()
                .join(", "),
            T::columns_to_string()
        );

        let mut executor = qb.set_sql(&sql).prepare();

        executor = bind_entry_to_query(executor, &entry);

        executor.fetch_one(self.get_pool()).await
    }

    async fn create_many<'tx>(&self, entries: Vec<T>) -> RepositoryResult<Vec<T>>
    where
        for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
        for<'c> &'c mut Transaction<'c, DB>: sqlx::Executor<'c, Database = DB>,
        Self: Sync,
    {
        if entries.is_empty() {
            return Ok(vec![]);
        }

        let now = Utc::now();
        let nb_columns = T::insertable_columns().len();

        let mut placeholder_qb = self.query();
        let placeholders = (0..nb_columns)
            .map(|_| placeholder_qb.placeholder())
            .collect::<Vec<String>>()
            .join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
            T::table_name(),
            T::insertable_columns_to_string(),
            placeholders,
            T::columns_to_string()
        );

        execute_transaction(self.get_pool(), self, |repo, tx| async move {
            let mut created_entries = Vec::new();

            for mut entry in entries.into_iter() {
                entry.set_created_at(now);
                entry.set_updated_at(now);

                let mut qb = repo.query();
                let mut executor = qb.set_sql(&sql).prepare();
                executor = bind_entry_to_query(executor, &entry);

                let created_entry = executor.fetch_one_with_transaction(tx).await?;
                created_entries.push(created_entry);
            }

            Ok(created_entries)
        })
        .await
    }

    /// Updates a record by its id with the provided entry data.
    async fn update(&self, id: T::Id, mut entry: T) -> RepositoryResult<T> {
        let now = Utc::now();
        entry.set_updated_at(now);

        let mut qb = self.query();

        let sql = format!(
            "UPDATE {} SET {} WHERE id = {} RETURNING {}",
            T::table_name(),
            T::insertable_columns()
                .iter()
                .map(|col| format!("{} = {}", col, qb.placeholder()))
                .collect::<Vec<String>>()
                .join(", "),
            qb.placeholder(),
            T::columns_to_string()
        );

        let mut executor = qb.set_sql(&sql).prepare();

        executor = bind_entry_to_query(executor, &entry);
        executor = executor.bind(id);

        executor.fetch_one(self.get_pool()).await
    }

    /// Partially updates a record by its id with the provided updates.
    async fn update_partial(
        &self,
        id: T::Id,
        columns: Vec<&str>,
        values: Vec<BindValue>,
    ) -> RepositoryResult<T> {
        if columns.is_empty() || values.is_empty() || columns.len() != values.len() {
            return Err(ApiError::BadRequest("Empty columns or values".to_string()));
        }

        let mut qb = self.query();
        let sql = format!(
            "UPDATE {} SET {} WHERE id = {} RETURNING {}",
            T::table_name(),
            columns
                .iter()
                .map(|col| format!("{} = {}", col, qb.placeholder()))
                .collect::<Vec<String>>()
                .join(", "),
            qb.placeholder(),
            T::columns_to_string()
        );
        let mut executor = qb.set_sql(&sql).prepare();
        for value in values.iter() {
            executor = bind_value_to_query(executor, value);
        }
        executor = executor.bind(id);
        executor.fetch_one(self.get_pool()).await
    }

    /// Deletes a record by its id. Returns true if a record was deleted.
    async fn delete(&self, id: T::Id) -> RepositoryResult<bool> {
        let mut qb = self.query();

        let sql = format!(
            "DELETE FROM {} WHERE id = {}",
            T::table_name(),
            qb.placeholder()
        );
        qb.set_sql(&sql)
            .prepare()
            .bind(id)
            .execute(self.get_pool())
            .await?;
        Ok(true)
    }

    /// Deletes multiple records by their ids. Returns the number of records deleted.
    async fn delete_many(&self, ids: &[T::Id]) -> RepositoryResult<u64> {

        if ids.is_empty() {
            return Ok(0);
        }

        let mut qb = self.query();
        let placeholders = ids
            .iter()
            .map(|_| qb.placeholder())
            .collect::<Vec<String>>()
            .join(", ");

        let sql = format!(
            "DELETE FROM {} WHERE id IN ({})",
            T::table_name(),
            placeholders
        );

        let mut executor = qb.set_sql(&sql).prepare();
        for id in ids.iter() {
            executor = executor.bind(*id);
        }

        let result = executor.execute(self.get_pool()).await?;
        Ok(result.rows_affected())
    }

    /// Checks if a record exists by its id.
    async fn exists(&self, id: T::Id) -> RepositoryResult<bool> {
        let count = self
            .query()
            .where_eq(
                "id",
                serde_json::to_value(id).map_err(|e| ApiError::Serialization(e))?,
            )?
            .limit(1)
            .count(self.get_pool())
            .await?;

        Ok(count > 0)
    }

    /// Fetches records using a custom QueryBuilderUtil instance.
    async fn find_with_query(&self, query: QueryBuilderUtil<T>) -> RepositoryResult<Vec<T>> {
        query.fetch_all(self.get_pool()).await
    }

    /// Counts records using a custom QueryBuilderUtil instance.
    async fn count_with_query(&self, query: QueryBuilderUtil<T>) -> RepositoryResult<i64> {
        query.count(self.get_pool()).await
    }

    /// Fetches an optional record using a custom QueryBuilderUtil instance.
    async fn find_one_with_query(&self, query: QueryBuilderUtil<T>) -> RepositoryResult<Option<T>> {
        query.fetch_optional(self.get_pool()).await
    }

    /// Fetches a required record using a custom QueryBuilderUtil instance.
    async fn find_one_required_with_query(
        &self,
        query: QueryBuilderUtil<T>,
    ) -> RepositoryResult<T> {
        query.fetch_one(self.get_pool()).await
    }

    /// Deletes records using a custom QueryBuilderUtil instance.
    async fn delete_by_query(&self, query: QueryBuilderUtil<T>) -> RepositoryResult<u64> {
        query.delete(self.get_pool()).await
    }

    /// Finds records with advanced options: conditions, ordering, limit, and offset.
    async fn find_advanced(
        &self,
        conditions: &[(&str, Value)],
        order_by: Option<(&str, OrderDirection)>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> RepositoryResult<Vec<T>> {
        let mut query = self.query();

        for (i, (column, value)) in conditions.iter().enumerate() {
            query = query.where_eq(column, value.clone())?;
            if i < conditions.len() - 1 {
                query = query.and();
            }
        }

        if let Some((column, direction)) = order_by {
            query = query.order_by(column, direction)?;
        }

        if let Some(l) = limit {
            query = query.limit(l);
        }

        if let Some(o) = offset {
            query = query.offset(o);
        }

        query.fetch_all(self.get_pool()).await
    }

    /// Searches for records where a column matches a pattern (LIKE/ILIKE).
    async fn search_by_pattern(
        &self,
        column: &str,
        pattern: &str,
        case_sensitive: bool,
        limit: Option<u32>,
    ) -> RepositoryResult<Vec<T>> {
        let search_pattern = format!("%{}%", pattern);
        let mut query = self.query();

        query = if case_sensitive {
            query.where_like(column, search_pattern)?
        } else {
            query.where_ilike(column, search_pattern)?
        };

        if let Some(l) = limit {
            query = query.limit(l);
        }

        query.fetch_all(self.get_pool()).await
    }

    /// Finds records where a column value is within a specified range.
    async fn find_by_range<V>(&self, column: &str, start: V, end: V) -> RepositoryResult<Vec<T>>
    where
        V: serde::Serialize,
    {
        let start_value = serde_json::to_value(start).map_err(|e| ApiError::Serialization(e))?;
        let end_value = serde_json::to_value(end).map_err(|e| ApiError::Serialization(e))?;

        self.query()
            .where_between(column, start_value, end_value)?
            .fetch_all(self.get_pool())
            .await
    }

    /// Finds records where a column value is in a list of values.
    async fn find_by_values<V>(&self, column: &str, values: Vec<V>) -> RepositoryResult<Vec<T>>
    where
        V: serde::Serialize,
    {
        if values.is_empty() {
            return Ok(vec![]);
        }

        let json_values: Result<Vec<Value>, _> = values
            .into_iter()
            .map(|v| serde_json::to_value(v))
            .collect();
        let json_values = json_values.map_err(|e| ApiError::Serialization(e))?;

        self.query()
            .where_in(column, json_values)?
            .fetch_all(self.get_pool())
            .await
    }

    /// Fetches a paginated and optionally sorted list of records.
    async fn paginate_sorted(
        &self,
        page: u32,
        page_size: u32,
        sort_column: Option<&str>,
        sort_direction: Option<OrderDirection>,
    ) -> RepositoryResult<Vec<T>> {
        let mut query = self.query().paginate(page, page_size);

        if let Some(column) = sort_column {
            let direction = sort_direction.unwrap_or(OrderDirection::Asc);
            query = query.order_by(column, direction)?;
        } else {
            query = query.order_by("id", OrderDirection::Asc)?;
        }

        query.fetch_all(self.get_pool()).await
    }
}
