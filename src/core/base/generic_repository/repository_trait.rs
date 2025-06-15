use std::collections::HashMap;

use crate::core::{
    base::query_builder::{query_builder::QueryBuilderUtil, query_models::OrderDirection},
    errors::errors::ApiError,
};

use super::entry_trait::Entry;
use serde_json::Value;
use sqlx::{Pool, Postgres, QueryBuilder};

pub type RepositoryResult<T> = Result<T, ApiError>;

pub trait RepositoryTrait<T: Entry + Send + Sync + Unpin + 'static> {
    /// Returns a reference to the Postgres connection pool.
    fn get_pool(&self) -> &Pool<Postgres>;

    /// Creates a new QueryBuilderUtil instance for building queries.
    fn query(&self) -> QueryBuilderUtil<T> {
        QueryBuilderUtil::new()
    }

    /// Fetches all records of type T from the database.
    async fn find_all(&self) -> RepositoryResult<Vec<T>> {
        self.query().fetch_all(self.get_pool()).await
    }

    /// Finds a record by its primary key (id). Returns an Option<T>.
    async fn find_by_id(&self, id: T::Id) -> RepositoryResult<Option<T>> {
        self.query()
            .where_eq(
                "id",
                serde_json::to_value(id).map_err(|e| ApiError::Serialization(e))?,
            )?
            .fetch_optional(self.get_pool())
            .await
    }

    /// Finds a record by its primary key (id). Returns an error if not found.
    async fn find_by_id_required(&self, id: T::Id) -> RepositoryResult<T> {
        self.query()
            .where_eq(
                "id",
                serde_json::to_value(id).map_err(|e| ApiError::Serialization(e))?,
            )?
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| match e {
                ApiError::NotFound(_) => ApiError::NotFound(format!("{:?}", id)),
                _ => e,
            })
    }

    /// Finds records by a specific column and value.
    async fn find_by_column<V>(&self, column: &str, value: V) -> RepositoryResult<Vec<T>>
    where
        V: Send + Sync + serde::Serialize,
    {
        let json_value = serde_json::to_value(value).map_err(|e| ApiError::Serialization(e))?;

        self.query()
            .where_eq(column, json_value)?
            .fetch_all(self.get_pool())
            .await
    }

    /// Finds records matching a set of criteria (column, value pairs).
    async fn find_by_criteria(&self, criteria: &[(&str, Value)]) -> RepositoryResult<Vec<T>> {
        if criteria.is_empty() {
            return self.find_all().await;
        }

        let mut query = self.query();

        for (i, (column, value)) in criteria.iter().enumerate() {
            query = query.where_eq(column, value.clone())?;
            if i < criteria.len() - 1 {
                query = query.and();
            }
        }

        query.fetch_all(self.get_pool()).await
    }

    /// Counts the total number of records of type T.
    async fn count(&self) -> RepositoryResult<i64> {
        self.query().count(self.get_pool()).await
    }

    /// Fetches a paginated list of records, ordered by id ascending.
    async fn paginate(&self, page: u32, page_size: u32) -> RepositoryResult<Vec<T>> {
        self.query()
            .order_by("id", OrderDirection::Asc)?
            .paginate(page, page_size)
            .fetch_all(self.get_pool())
            .await
    }

    /// Creates a new record in the database and returns it.
    async fn create(&self, mut entry: T) -> RepositoryResult<T> {
        use chrono::Utc;

        let columns = T::insertable_columns();
        let now = Utc::now();
        entry.set_created_at(now);
        entry.set_updated_at(now);

        let entry_json = serde_json::to_value(&entry).map_err(|e| ApiError::Serialization(e))?;

        let mut insert_data = HashMap::new();
        for col in &columns {
            let value = entry_json.get(*col).cloned().unwrap_or(Value::Null);
            insert_data.insert(col.to_string(), value);
        }

        self.query()
            .values(insert_data)?
            .insert_returning(self.get_pool())
            .await
    }

    /// Creates multiple records in the database and returns them.
    async fn create_many(&self, entries: Vec<T>) -> RepositoryResult<Vec<T>> {
        if entries.is_empty() {
            return Ok(vec![]);
        }

        use chrono::Utc;
        let columns = T::insertable_columns();
        let now = Utc::now();
        let mut results = Vec::new();

        for mut entry in entries {
            entry.set_created_at(now);
            entry.set_updated_at(now);

            let entry_json =
                serde_json::to_value(&entry).map_err(|e| ApiError::Serialization(e))?;

            let mut insert_data = HashMap::new();
            for col in &columns {
                let value = entry_json.get(*col).cloned().unwrap_or(Value::Null);
                insert_data.insert(col.to_string(), value);
            }

            let created_entry = self
                .query()
                .values(insert_data)?
                .insert_returning(self.get_pool())
                .await?;

            results.push(created_entry);
        }

        Ok(results)
    }

    /// Updates a record by its id with the provided entry data.
    async fn update(&self, id: T::Id, mut entry: T) -> RepositoryResult<T> {
        use chrono::Utc;

        let columns = T::columns();
        let now = Utc::now();
        entry.set_updated_at(now);

        let entry_json = serde_json::to_value(&entry).map_err(|e| ApiError::Serialization(e))?;

        let mut update_data = HashMap::new();
        for col in &columns {
            if *col == "id" || *col == "created_at" {
                continue;
            }
            let value = entry_json.get(*col).cloned().unwrap_or(Value::Null);
            update_data.insert(col.to_string(), value);
        }

        let updated_entries = self
            .query()
            .where_eq(
                "id",
                serde_json::to_value(id).map_err(|e| ApiError::Serialization(e))?,
            )?
            .set_multiple(update_data)?
            .update_returning(self.get_pool())
            .await?;

        updated_entries
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::NotFound(format!("No record found with id: {:?}", id)))
    }

    /// Partially updates a record by its id with the provided updates.
    async fn update_partial(
        &self,
        id: T::Id,
        updates: Vec<(String, Value)>,
    ) -> RepositoryResult<T> {
        if updates.is_empty() {
            return self.find_by_id_required(id).await;
        }

        use chrono::Utc;
        let now = Utc::now();

        let mut update_data = HashMap::new();

        for (column, value) in updates.into_iter() {
            if !T::columns().contains(&column.as_str()) {
                return Err(ApiError::InvalidColumn(column.clone()));
            }

            if column == "id" || column == "created_at" {
                continue;
            }

            update_data.insert(column, value);
        }

        update_data.insert(
            "updated_at".to_string(),
            serde_json::to_value(now).map_err(|e| ApiError::Serialization(e))?,
        );

        let updated_entries = self
            .query()
            .where_eq(
                "id",
                serde_json::to_value(id.clone()).map_err(|e| ApiError::Serialization(e))?,
            )?
            .set_multiple(update_data)?
            .update_returning(self.get_pool())
            .await?;

        updated_entries
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::NotFound(format!("No record found with id: {:?}", id)))
    }

    /// Deletes a record by its id. Returns true if a record was deleted.
    async fn delete(&self, id: T::Id) -> RepositoryResult<bool> {
        let rows_affected = self
            .query()
            .where_eq(
                "id",
                serde_json::to_value(id).map_err(|e| ApiError::Serialization(e))?,
            )?
            .delete(self.get_pool())
            .await?;

        Ok(rows_affected > 0)
    }

    /// Deletes multiple records by their ids. Returns the number of records deleted.
    async fn delete_many(&self, ids: &[T::Id]) -> RepositoryResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let id_values: Result<Vec<Value>, _> =
            ids.iter().map(|id| serde_json::to_value(id)).collect();
        let id_values = id_values.map_err(|e| ApiError::Serialization(e))?;

        self.query()
            .where_in("id", id_values)?
            .delete(self.get_pool())
            .await
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
