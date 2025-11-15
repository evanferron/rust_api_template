use crate::core::{
    base::query_builder::{query_builder::QueryBuilder},
    errors::errors::ApiError,
};

use super::entry_trait::{BindValue, Entry};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Database, FromRow, Pool, Transaction};
use uuid::Uuid;
use crate::core::base::extension::query_result_extension::QueryResultExt;
use crate::core::base::generic_repository::result_models::{CountResult, ExistResult};

pub trait RepositoryTrait<T, DB>
where
    T: Entry<DB> + Send + Sync + Unpin + 'static + for<'r> sqlx::FromRow<'r, <DB as Database>::Row>,
    DB: Database,
    DB::QueryResult: QueryResultExt,
    for<'a> <DB as Database>::Arguments<'a>: sqlx::IntoArguments<'a, DB>,
    for<'a> &'a mut <DB as Database>::Connection: sqlx::Executor<'a, Database=DB>,
    for<'a> &'a str: sqlx::ColumnIndex<<DB as Database>::Row>,
    for<'q> i32: sqlx::Encode<'q, DB>+ sqlx::Decode<'q, DB> + sqlx::Type<DB>,
    for<'q> i64: sqlx::Encode<'q, DB>+ sqlx::Decode<'q, DB> + sqlx::Type<DB>,
    for<'q> f64: sqlx::Encode<'q, DB>+ sqlx::Decode<'q, DB> + sqlx::Type<DB>,
    for<'q> bool: sqlx::Encode<'q, DB>+ sqlx::Decode<'q, DB> + sqlx::Type<DB>,
    for<'q> String: sqlx::Encode<'q, DB>+ sqlx::Decode<'q, DB> + sqlx::Type<DB>,
    for<'q> Uuid: sqlx::Encode<'q, DB>+ sqlx::Decode<'q, DB> + sqlx::Type<DB>,
    for<'q> Value: sqlx::Encode<'q, DB>+ sqlx::Type<DB>,
    for<'q> DateTime<Utc>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
    for<'q> Option<String>: sqlx::Encode<'q, DB> + sqlx::Type<DB>, str: sqlx::Type<DB>,
    for<'q> sqlx::types::Json<Value>: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
{
    /// Returns a reference to the Postgres connection pool.
    fn get_pool(&self) -> &Pool<DB>;

    /// Creates a new QueryBuilder instance for building queries.
    fn query(&self) -> QueryBuilder<DB, T>;

    fn query_custom<M>(&self)
                       -> QueryBuilder<DB, M>
    where
        M: for<'r> FromRow<'r, DB::Row> + Send + Unpin;

    /// Fetches all records of type T from the database.
    async fn find_all(&self) -> Result<Vec<T>,ApiError> {
        let sql = format!(
            "SELECT {} FROM {}",
            T::columns().join(", "),
            T::table_name()
        );

        self.query().set_sql(&sql).fetch_all(self.get_pool()).await
    }

    /// Finds a record by its primary key (id). Returns an Option<T>.
    async fn find_by_id(&self, id: BindValue) -> Result<T,ApiError> {
        let mut qb = self.query();

        let sql = format!(
            "SELECT {} FROM {} WHERE id = {}",
            T::columns_to_string(),
            T::table_name(),
            qb.placeholder()
        );

        qb.set_sql(&sql).add_param(id)
            .fetch_one(self.get_pool())
            .await
    }

    /// Finds records by a specific column and value.
    async fn find_by_column(&self, column: &str, value: BindValue) -> Result<Vec<T>,ApiError>
    {
        if !T::columns().contains(&column) {
            return Err(ApiError::BadRequest(format!("Invalid column: {}", column)));
        }

        let mut qb = self.query();

        let sql = format!(
            "SELECT {} FROM {} WHERE {} = {}",
            T::columns_to_string(),
            T::table_name(),
            column,
            qb.placeholder()
        );

        qb.set_sql(&sql)
            .add_param(value)
            .fetch_all(self.get_pool())
            .await
    }

    /// Finds records matching a set of criteria (column, value pairs).
    async fn find_by_columns(&self, columns: &[&str], values: Vec<BindValue>) -> Result<Vec<T>,ApiError>
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
        qb.set_sql(&sql).add_params(values).fetch_all(self.get_pool()).await
    }

    /// Counts the total number of records of type T.
    async fn count(&self) -> Result<i64,ApiError> {
        let sql = format!("SELECT COUNT(*) as count FROM {}", T::table_name());
        let res = self
            .query_custom::<CountResult>()
            .set_sql(&sql)
            .fetch_one_simple(self.get_pool())
            .await;
        match res {
            Ok(row) => Ok(row.count),
            Err(e) => Err(e),
        }
    }

    /// Fetches a paginated list of records, ordered by id ascending.
    async fn paginate(&self, page: u32, page_size: u32) -> Result<Vec<T>,ApiError> {
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
    async fn create(&self, mut entry: T) -> Result<T,ApiError> {
        let now = Utc::now();
        entry.set_created_at(now);
        entry.set_updated_at(now);

        let nb_columns = T::insertable_columns().len();
        let mut qb = self.query();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
            T::table_name(),
            T::insertable_columns_to_string(),
            (0..nb_columns)
                .map(|_i| { qb.placeholder() })
                .collect::<Vec<String>>()
                .join(", "),
            T::columns_to_string()
        );

        qb.set_sql(&sql).add_params(entry.to_bind_values()).fetch_one(self.get_pool()).await
    }

    async fn create_many<'tx>(&self, entries: Vec<T>) -> Result<Vec<T>,ApiError>
    where
            for<'c> &'c mut Transaction<'tx, DB>: sqlx::Executor<'c, Database = DB>,
            Self: Sync,
    {
        if entries.is_empty() {
            return Err(ApiError::BadRequest("Empty entries".to_string()));
        }

        let now = Utc::now();
        let nb_columns = T::insertable_columns().len();
        let nb_entries = entries.len();

        // Générer les placeholders pour tous les entries : (?, ?, ?), (?, ?, ?), ...
        let mut placeholder_qb = self.query();
        let values_clause = (0..nb_entries)
            .map(|_| {
                format!(
                    "({})",
                    (0..nb_columns)
                        .map(|_| placeholder_qb.placeholder())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        let sql = format!(
            "INSERT INTO {} ({}) VALUES {} RETURNING {}",
            T::table_name(),
            T::insertable_columns_to_string(),
            values_clause,
            T::columns_to_string()
        );

        let mut tx = self.get_pool().begin().await.map_err(ApiError::from)?;

        let mut qb = self.query().set_sql(&sql);
        for mut entry in entries.into_iter() {
            entry.set_created_at(now);
            entry.set_updated_at(now);
            for value in entry.to_bind_values() {
                qb = qb.add_param(value);
            }
        }

        let created_entries = qb.fetch_all_with_transaction(&mut tx).await?;
        tx.commit().await.map_err(ApiError::from)?;

        Ok(created_entries)
    }

    /// Partially updates a record by its id with the provided updates.
    async fn update(
        &self,
        id: BindValue,
        columns: Vec<&str>,
        values: Vec<BindValue>,
    ) -> Result<T,ApiError> {
        // validation des entrées
        if columns.is_empty() || values.is_empty() || columns.len() != values.len() {
            return Err(ApiError::BadRequest("Empty columns or values".to_string()));
        }
        let insertable_columns = T::insertable_columns();
        for col in &columns {
            if !insertable_columns.contains(col) {
                return Err(ApiError::BadRequest(format!(
                    "Column '{}' is not updatable",
                    col
                )));
            }
        }
        // fin des vérifications


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
        qb.set_sql(&sql).add_params(values).add_param(id).fetch_one(self.get_pool()).await
    }

    /// Deletes a record by its id. Returns true if a record was deleted.
    async fn delete(&self, id: BindValue) -> Result<bool,ApiError> {
        let mut qb = self.query();

        let sql = format!(
            "DELETE FROM {} WHERE id = {}",
            T::table_name(),
            qb.placeholder()
        );
        let rows_affected = qb.set_sql(&sql).add_param(id).execute(self.get_pool()).await?.rows_affected();
        Ok(rows_affected == 1)
    }

    /// Met à jour plusieurs enregistrements en une seule requête
    async fn update_many(
        &self,
        ids: Vec<BindValue>,
        columns: Vec<&str>,
        values: Vec<BindValue>,
    ) -> Result<Vec<T>,ApiError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        // Valider les colonnes
        let insertable_columns = T::insertable_columns();
        for col in &columns {
            if !insertable_columns.contains(col) {
                return Err(ApiError::BadRequest(format!("Column '{}' is not updatable", col)));
            }
        }

        let mut qb = self.query();
        let id_placeholders = ids.iter().map(|_| qb.placeholder()).collect::<Vec<_>>().join(", ");

        let sql = format!(
            "UPDATE {} SET {} WHERE id IN ({}) RETURNING {}",
            T::table_name(),
            columns
                .iter()
                .map(|col| format!("{} = {}", col, qb.placeholder()))
                .collect::<Vec<_>>()
                .join(", "),
            id_placeholders,
            T::columns_to_string()
        );

        qb.set_sql(&sql)
            .add_params(values)
            .add_params(ids)
            .fetch_all(self.get_pool())
            .await
    }

    async fn upsert(
        &self,
        mut entry: T,
        conflict_columns: &[&str],
        update_columns: &[&str],
    ) -> Result<T,ApiError> {
        let now = Utc::now();
        entry.set_created_at(now);
        entry.set_updated_at(now);

        let nb_columns = T::insertable_columns().len();
        let mut qb = self.query();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) ON CONFLICT ({}) DO UPDATE SET {} RETURNING {}",
            T::table_name(),
            T::insertable_columns_to_string(),
            (0..nb_columns).map(|_| qb.placeholder()).collect::<Vec<_>>().join(", "),
            conflict_columns.join(", "),
            update_columns
                .iter()
                .map(|col| format!("{} = EXCLUDED.{}", col, col))
                .collect::<Vec<_>>()
                .join(", "),
            T::columns_to_string()
        );

        qb.set_sql(&sql)
            .add_params(entry.to_bind_values())
            .fetch_one(self.get_pool())
            .await
    }

    /// Deletes multiple records by their ids. Returns the number of records deleted.
    async fn delete_many(&self, ids: Vec<BindValue>) -> Result<u64,ApiError> {
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

        qb = qb.set_sql(&sql);
        for id in ids {
            qb = qb.add_param(id);
        }


        let result = qb.execute(self.get_pool()).await?;
        Ok(result.rows_affected())
    }

    async fn soft_delete(&self, id: BindValue) -> Result<T,ApiError> {
        let now = Utc::now();
        self.update(
            id,
            vec!["deleted_at"],
            vec![BindValue::DateTime(now)],
        ).await
    }

    async fn restore(&self, id: BindValue) -> Result<T,ApiError> {
        self.update(
            id,
            vec!["deleted_at"],
            vec![BindValue::Null],
        ).await
    }

    /// Checks if a record exists by its id.
    async fn exists(&self, id: BindValue) -> Result<bool,ApiError> {
        let mut qb = self.query_custom::<ExistResult>();

        let sql = format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE id = {}) as exists",
            T::table_name(),
            qb.placeholder()
        );

        let row = qb
            .set_sql(&sql)
            .add_param(id)
            .fetch_one(self.get_pool())
            .await?;

        Ok(row.exist)
    }
}
