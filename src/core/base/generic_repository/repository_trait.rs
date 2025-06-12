use crate::core::errors::errors::ApiError;

use super::entry_trait::Entry;
use serde_json::Value;
use sqlx::{Pool, Postgres, QueryBuilder};

pub type RepositoryResult<T> = Result<T, ApiError>;

pub trait RepositoryTrait<T: Entry + Send + Sync + Unpin + 'static> {
    fn get_pool(&self) -> &Pool<Postgres>;

    async fn find_all(&self) -> RepositoryResult<Vec<T>> {
        let query = format!(
            "SELECT {} FROM {}",
            T::columns().join(", "),
            T::table_name()
        );

        let items = sqlx::query_as::<_, T>(&query)
            .fetch_all(self.get_pool())
            .await
            .map_err(ApiError::Database)?;

        Ok(items)
    }

    async fn find_by_id(&self, id: T::Id) -> RepositoryResult<Option<T>> {
        let mut query_builder = QueryBuilder::new("SELECT ");
        query_builder.push(T::columns().join(", "));
        query_builder.push(" FROM ");
        query_builder.push(T::table_name());
        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);

        let item = query_builder
            .build_query_as::<T>()
            .fetch_optional(self.get_pool())
            .await?;

        Ok(item)
    }

    async fn find_by_id_required(&self, id: T::Id) -> RepositoryResult<T> {
        self.find_by_id(id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("{:?}", id)))
    }

    async fn find_by_column<V>(&self, column: &str, value: V) -> RepositoryResult<Vec<T>>
    where
        V: Send + Sync + for<'q> sqlx::Encode<'q, Postgres> + sqlx::Type<Postgres>,
    {
        // Validation basique du nom de colonne
        if !T::columns().contains(&column) {
            return Err(ApiError::InvalidColumn(column.to_string()));
        }

        let mut query_builder = QueryBuilder::new("SELECT ");
        query_builder.push(T::columns().join(", "));
        query_builder.push(" FROM ");
        query_builder.push(T::table_name());
        query_builder.push(" WHERE ");
        query_builder.push(column);
        query_builder.push(" = ");
        query_builder.push_bind(value);

        let items = query_builder
            .build_query_as::<T>()
            .fetch_all(self.get_pool())
            .await
            .map_err(ApiError::Database)?;

        Ok(items)
    }

    async fn find_by_criteria(&self, criteria: &[(&str, Value)]) -> RepositoryResult<Vec<T>> {
        if criteria.is_empty() {
            return self.find_all().await;
        }

        let mut query_builder = QueryBuilder::new("SELECT ");
        query_builder.push(T::columns().join(", "));
        query_builder.push(" FROM ");
        query_builder.push(T::table_name());
        query_builder.push(" WHERE ");

        for (i, (column, value)) in criteria.iter().enumerate() {
            if !T::columns().contains(column) {
                return Err(ApiError::InvalidColumn(column.to_string()));
            }

            if i > 0 {
                query_builder.push(" AND ");
            }

            query_builder.push(*column);
            query_builder.push(" = ");
            query_builder.push_bind(value.clone());
        }

        let items = query_builder
            .build_query_as::<T>()
            .fetch_all(self.get_pool())
            .await
            .map_err(ApiError::Database)?;

        Ok(items)
    }

    async fn count(&self) -> RepositoryResult<i64> {
        let query = format!("SELECT COUNT(*) FROM {}", T::table_name());

        let count: (i64,) = sqlx::query_as(&query).fetch_one(self.get_pool()).await?;

        Ok(count.0)
    }

    async fn paginate(&self, page: u32, page_size: u32) -> RepositoryResult<Vec<T>> {
        let offset = (page.saturating_sub(1)) * page_size;

        let query = format!(
            "SELECT {} FROM {} ORDER BY id LIMIT {} OFFSET {}",
            T::columns().join(", "),
            T::table_name(),
            page_size,
            offset
        );

        let items = sqlx::query_as::<_, T>(&query)
            .fetch_all(self.get_pool())
            .await?;

        Ok(items)
    }

    async fn create(&self, mut entry: T) -> RepositoryResult<T> {
        use chrono::Utc;

        let columns = T::columns();
        let now = Utc::now();
        entry.set_created_at(now);
        entry.set_updated_at(now);

        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        query_builder.push(T::table_name());
        query_builder.push(" (");
        query_builder.push(columns.join(", "));
        query_builder.push(") VALUES (");

        let entry_json = serde_json::to_value(&entry)?;

        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                query_builder.push(", ");
            }
            let value = entry_json.get(*col).cloned().unwrap_or(Value::Null);
            query_builder.push_bind(value);
        }

        query_builder.push(") RETURNING *");

        let item = query_builder
            .build_query_as::<T>()
            .fetch_one(self.get_pool())
            .await?;

        Ok(item)
    }

    async fn create_many(&self, entries: Vec<T>) -> RepositoryResult<Vec<T>> {
        if entries.is_empty() {
            return Ok(vec![]);
        }

        use chrono::Utc;
        let columns = T::columns();
        let now = Utc::now();

        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        query_builder.push(T::table_name());
        query_builder.push(" (");
        query_builder.push(columns.join(", "));
        query_builder.push(") VALUES ");

        for (entry_idx, mut entry) in entries.into_iter().enumerate() {
            entry.set_created_at(now);
            entry.set_updated_at(now);

            if entry_idx > 0 {
                query_builder.push(", ");
            }

            query_builder.push("(");
            let entry_json = serde_json::to_value(&entry)?;

            for (i, col) in columns.iter().enumerate() {
                if i > 0 {
                    query_builder.push(", ");
                }
                let value = entry_json.get(*col).cloned().unwrap_or(Value::Null);
                query_builder.push_bind(value);
            }
            query_builder.push(")");
        }

        query_builder.push(" RETURNING *");

        let items = query_builder
            .build_query_as::<T>()
            .fetch_all(self.get_pool())
            .await?;

        Ok(items)
    }

    async fn update(&self, id: T::Id, mut entry: T) -> RepositoryResult<T> {
        use chrono::Utc;

        let columns = T::columns();
        let now = Utc::now();
        entry.set_updated_at(now);

        let mut query_builder = QueryBuilder::new("UPDATE ");
        query_builder.push(T::table_name());
        query_builder.push(" SET ");

        let entry_json = serde_json::to_value(&entry)?;
        let mut first = true;

        for col in &columns {
            if *col == "id" || *col == "created_at" {
                continue;
            }
            if !first {
                query_builder.push(", ");
            }
            first = false;

            query_builder.push(*col);
            query_builder.push(" = ");

            let value = entry_json.get(*col).cloned().unwrap_or(Value::Null);
            query_builder.push_bind(value);
        }

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let item = query_builder
            .build_query_as::<T>()
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => ApiError::NotFound(format!("{:?}", id)),
                _ => ApiError::Database(e),
            })?;

        Ok(item)
    }

    async fn update_partial(&self, id: T::Id, updates: &[(&str, Value)]) -> RepositoryResult<T> {
        if updates.is_empty() {
            return self.find_by_id_required(id).await;
        }

        use chrono::Utc;
        let now = Utc::now();

        let mut query_builder = QueryBuilder::new("UPDATE ");
        query_builder.push(T::table_name());
        query_builder.push(" SET ");

        for (i, (column, value)) in updates.iter().enumerate() {
            if !T::columns().contains(column) {
                return Err(ApiError::InvalidColumn(column.to_string()));
            }

            if *column == "id" || *column == "created_at" {
                continue;
            }

            if i > 0 {
                query_builder.push(", ");
            }

            query_builder.push(*column);
            query_builder.push(" = ");
            query_builder.push_bind(value.clone());
        }

        // Toujours mettre à jour updated_at
        query_builder.push(", updated_at = ");
        query_builder.push_bind(now);

        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);
        query_builder.push(" RETURNING *");

        let item = query_builder
            .build_query_as::<T>()
            .fetch_one(self.get_pool())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => ApiError::NotFound(format!("{:?}", id)),
                _ => ApiError::Database(e),
            })?;

        Ok(item)
    }

    async fn delete(&self, id: T::Id) -> RepositoryResult<bool> {
        let mut query_builder = QueryBuilder::new("DELETE FROM ");
        query_builder.push(T::table_name());
        query_builder.push(" WHERE id = ");
        query_builder.push_bind(id);

        let result = query_builder.build().execute(self.get_pool()).await?;
        Ok(result.rows_affected() > 0)
    }

    async fn delete_many(&self, ids: &[T::Id]) -> RepositoryResult<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let mut query_builder = QueryBuilder::new("DELETE FROM ");
        query_builder.push(T::table_name());
        query_builder.push(" WHERE id = ANY(ARRAY[");
        for (i, id) in ids.iter().enumerate() {
            if i > 0 {
                query_builder.push(", ");
            }
            query_builder.push_bind(id.clone());
        }
        query_builder.push("])");

        let result = query_builder.build().execute(self.get_pool()).await?;
        Ok(result.rows_affected())
    }

    async fn exists(&self, id: T::Id) -> RepositoryResult<bool> {
        let mut query_builder = QueryBuilder::new("SELECT 1 FROM ");
        query_builder
            .push(T::table_name())
            .push(" WHERE id = ")
            .push_bind(id)
            .push(" LIMIT 1");

        let exists = query_builder.build().execute(self.get_pool()).await?;

        Ok(exists.rows_affected() > 0)
    }
}
