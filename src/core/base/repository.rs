use sqlx::QueryBuilder;
use crate::core::base::bind_value::BindValue;
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

    pub async fn find_all(&self) -> Result<Vec<T>, sqlx::Error> {
        let mut qb = QueryBuilder::new("SELECT * FROM ");
        qb.push(T::table_name());

        qb.build_query_as::<T>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn find_by_id(&self, id: uuid::Uuid) -> Result<Option<T>, sqlx::Error> {
        let mut qb = QueryBuilder::new("SELECT * FROM ");
        qb.push(T::table_name());
        qb.push(" WHERE id = ");
        qb.push_bind(id);

        qb.build_query_as::<T>()
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_by(&self, criteria: Vec<(&str, BindValue)>) -> Result<Vec<T>, sqlx::Error> {
        let mut qb = QueryBuilder::new("SELECT * FROM ");
        qb.push(T::table_name());

        if !criteria.is_empty() {
            qb.push(" WHERE ");

            let mut separated = qb.separated(" AND ");

            for (col, val) in criteria {
                separated.push_unseparated(format!("{} = ", col));

                match val {
                    BindValue::I32(v) => separated.push_bind_unseparated(v),
                    BindValue::I64(v) => separated.push_bind_unseparated(v),
                    BindValue::F64(v) => separated.push_bind_unseparated(v),
                    BindValue::String(v) => separated.push_bind_unseparated(v),
                    BindValue::Bool(v) => separated.push_bind_unseparated(v),
                    BindValue::Uuid(v) => separated.push_bind_unseparated(v),
                    BindValue::Json(v) => separated.push_bind_unseparated(v),
                    BindValue::DateTime(v) => separated.push_bind_unseparated(v),
                    _ => panic!("Type de BindValue non géré dans find_by")
                };
            }
        }

        qb.build_query_as::<T>()
            .fetch_all(&self.pool)
            .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let mut qb = QueryBuilder::new("DELETE FROM ");
        qb.push(T::table_name());
        qb.push(" WHERE id = ");
        qb.push_bind(id);

        let result = qb.build().execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }
}