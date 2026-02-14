use crate::core::base::bind_value::BindValue;
use crate::core::errors::errors::ApiError;
use sqlx::QueryBuilder;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool};

pub trait DbEntity: for<'r> FromRow<'r, PgRow> + Send + Unpin + 'static {
    fn table_name() -> &'static str;
}

pub struct Repository<T: DbEntity> {
    pub pool: PgPool,
    _marker: std::marker::PhantomData<T>,
}

impl<T: DbEntity> Repository<T> {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: DbEntity> Repository<T> {
    pub async fn find_all(&self) -> Result<Vec<T>, ApiError> {
        let mut qb = QueryBuilder::new("SELECT * FROM ");
        qb.push(T::table_name());

        qb.build_query_as::<T>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::from(e))
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<T>, ApiError> {
        let mut qb = QueryBuilder::new("SELECT * FROM ");
        qb.push(T::table_name());
        qb.push(" WHERE id = ");
        qb.push_bind(id);

        qb.build_query_as::<T>()
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::from(e))
    }

    pub async fn find_by(&self, criteria: Vec<(&str, BindValue)>) -> Result<Vec<T>, ApiError> {
        let mut qb = QueryBuilder::new("SELECT * FROM ");
        qb.push(T::table_name());

        if !criteria.is_empty() {
            qb.push(" WHERE ");

            let mut separated = qb.separated(" AND ");

            for (col, val) in criteria {
                separated.push_unseparated(format!("{} = ", col));
                separated = BindValue::push_bind_unseparated(val, separated);
            }
        }

        qb.build_query_as::<T>()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::from(e))
    }

    pub async fn update(
        &self,
        id: uuid::Uuid,
        fields: Vec<(&str, BindValue)>,
    ) -> Result<T, ApiError> {
        if fields.is_empty() {
            return Err(ApiError::BadRequest(
                "Aucun champ à mettre à jour".to_string(),
            ));
        }

        let mut qb = QueryBuilder::new("UPDATE ");
        qb.push(T::table_name());
        qb.push(" SET ");

        let mut separated = qb.separated(", ");

        for (col, val) in fields {
            // On écrit "colonne = "
            separated.push_unseparated(format!("{} = ", col));

            // On bind la valeur selon son type
            separated = BindValue::push_bind_unseparated(val, separated);
        }

        qb.push(" WHERE id = ");
        qb.push_bind(id);

        qb.push(" RETURNING *");

        qb.build_query_as::<T>()
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::from(e))
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, ApiError> {
        let mut qb = QueryBuilder::new("DELETE FROM ");
        qb.push(T::table_name());
        qb.push(" WHERE id = ");
        qb.push_bind(id);

        let result = qb
            .build()
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::from(e))?;
        Ok(result.rows_affected() > 0)
    }
}
