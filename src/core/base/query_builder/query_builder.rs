use serde_json::Value;
use sqlx::{Pool, Postgres, QueryBuilder};
use std::collections::HashMap;

use crate::core::{
    base::{
        generic_repository::entry_trait::Entry,
        query_builder::query_models::{
            ComparisonOperator, JoinClause, JoinType, LogicalOperator, OrderBy, OrderDirection,
            QueryResult, WhereClause, WhereCondition, WhereGroup,
        },
    },
    errors::errors::ApiError,
};

#[derive(Debug)]
pub struct QueryBuilderUtil<T: Entry> {
    pub(crate) where_clauses: Vec<(WhereClause, Option<LogicalOperator>)>,
    pub(crate) order_by: Vec<OrderBy>,
    pub(crate) joins: Vec<JoinClause>,
    pub(crate) limit: Option<u32>,
    pub(crate) offset: Option<u32>,
    pub(crate) group_by: Vec<String>,
    pub(crate) having: Vec<WhereCondition>,
    pub(crate) distinct: bool,
    pub(crate) select_columns: Option<Vec<String>>,
    pub(crate) update_data: HashMap<String, Value>,
    pub(crate) insert_data: HashMap<String, Value>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Entry + Send + Sync + Unpin + 'static> QueryBuilderUtil<T> {
    pub fn new() -> Self {
        Self {
            where_clauses: Vec::new(),
            order_by: Vec::new(),
            joins: Vec::new(),
            limit: None,
            offset: None,
            group_by: Vec::new(),
            having: Vec::new(),
            distinct: false,
            select_columns: None,
            update_data: HashMap::new(),
            insert_data: HashMap::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    // Méthodes pour construire les conditions WHERE
    pub fn where_eq<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::Equal,
            value: Some(value.into()),
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_ne<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::NotEqual,
            value: Some(value.into()),
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_gt<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Some(value.into()),
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_gte<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::GreaterThanOrEqual,
            value: Some(value.into()),
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_lt<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::LessThan,
            value: Some(value.into()),
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_lte<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::LessThanOrEqual,
            value: Some(value.into()),
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_like<V: Into<Value>>(
        mut self,
        column: &str,
        pattern: V,
    ) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::Like,
            value: Some(pattern.into()),
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_ilike<V: Into<Value>>(
        mut self,
        column: &str,
        pattern: V,
    ) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::ILike,
            value: Some(pattern.into()),
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_in<V: Into<Value>>(
        mut self,
        column: &str,
        values: Vec<V>,
    ) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let values: Vec<Value> = values.into_iter().map(|v| v.into()).collect();
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::In,
            value: None,
            values: Some(values),
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_not_in<V: Into<Value>>(
        mut self,
        column: &str,
        values: Vec<V>,
    ) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let values: Vec<Value> = values.into_iter().map(|v| v.into()).collect();
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::NotIn,
            value: None,
            values: Some(values),
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_null(mut self, column: &str) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::IsNull,
            value: None,
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_not_null(mut self, column: &str) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::IsNotNull,
            value: None,
            values: None,
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_between<V: Into<Value>>(
        mut self,
        column: &str,
        start: V,
        end: V,
    ) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::Between,
            value: None,
            values: Some(vec![start.into(), end.into()]),
        };
        self.where_clauses
            .push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    // Opérateurs logiques
    pub fn and(mut self) -> Self {
        if let Some(last) = self.where_clauses.last_mut() {
            last.1 = Some(LogicalOperator::And);
        }
        self
    }

    pub fn or(mut self) -> Self {
        if let Some(last) = self.where_clauses.last_mut() {
            last.1 = Some(LogicalOperator::Or);
        }
        self
    }

    // ========== GROUPES DE CONDITIONS ==========

    /// Crée un groupe de conditions avec AND
    /// Exemple: where_group_and(|group| group.where_eq("status", "active").or().where_eq("priority", "high"))
    /// Résultat: WHERE (status = 'active' OR priority = 'high') AND ...
    pub fn where_group_and<F>(mut self, builder_fn: F) -> Result<Self, ApiError>
    where
        F: FnOnce(GroupBuilder<T>) -> Result<GroupBuilder<T>, ApiError>,
    {
        let group_builder = GroupBuilder::new();
        let group_builder = builder_fn(group_builder)?;

        if !group_builder.clauses.is_empty() {
            let group = WhereGroup {
                clauses: group_builder.clauses,
                operator: LogicalOperator::And,
            };
            self.where_clauses
                .push((WhereClause::Group(Box::new(group)), None));
        }

        Ok(self)
    }

    /// Crée un groupe de conditions avec OR
    /// Exemple: where_group_or(|group| group.where_eq("status", "draft").and().where_eq("author_id", user_id))
    /// Résultat: WHERE (status = 'draft' AND author_id = 123) OR ...
    pub fn where_group_or<F>(mut self, builder_fn: F) -> Result<Self, ApiError>
    where
        F: FnOnce(GroupBuilder<T>) -> Result<GroupBuilder<T>, ApiError>,
    {
        let group_builder = GroupBuilder::new();
        let group_builder = builder_fn(group_builder)?;

        if !group_builder.clauses.is_empty() {
            let group = WhereGroup {
                clauses: group_builder.clauses,
                operator: LogicalOperator::Or,
            };
            self.where_clauses
                .push((WhereClause::Group(Box::new(group)), None));
        }

        Ok(self)
    }

    // Méthodes pour ORDER BY
    pub fn order_by(mut self, column: &str, direction: OrderDirection) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        self.order_by.push(OrderBy {
            column: column.to_string(),
            direction,
        });
        Ok(self)
    }

    pub fn order_by_asc(self, column: &str) -> Result<Self, ApiError> {
        self.order_by(column, OrderDirection::Asc)
    }

    pub fn order_by_desc(self, column: &str) -> Result<Self, ApiError> {
        self.order_by(column, OrderDirection::Desc)
    }

    // Méthodes pour LIMIT et OFFSET
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn paginate(mut self, page: u32, page_size: u32) -> Self {
        let offset = (page.saturating_sub(1)) * page_size;
        self.limit = Some(page_size);
        self.offset = Some(offset);
        self
    }

    // Méthodes pour JOIN
    pub fn inner_join(mut self, table: &str, on_condition: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Inner,
            table: table.to_string(),
            on_condition: on_condition.to_string(),
        });
        self
    }

    pub fn left_join(mut self, table: &str, on_condition: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Left,
            table: table.to_string(),
            on_condition: on_condition.to_string(),
        });
        self
    }

    pub fn right_join(mut self, table: &str, on_condition: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Right,
            table: table.to_string(),
            on_condition: on_condition.to_string(),
        });
        self
    }

    pub fn full_outer_join(mut self, table: &str, on_condition: &str) -> Self {
        self.joins.push(JoinClause {
            join_type: JoinType::Full,
            table: table.to_string(),
            on_condition: on_condition.to_string(),
        });
        self
    }

    // Méthodes pour GROUP BY et HAVING
    pub fn group_by(mut self, column: &str) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        self.group_by.push(column.to_string());
        Ok(self)
    }

    // Méthodes pour DISTINCT et SELECT
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    pub fn select(mut self, columns: Vec<&str>) -> Result<Self, ApiError> {
        for column in &columns {
            self.validate_column(column)?;
        }
        self.select_columns = Some(columns.iter().map(|&s| s.to_string()).collect());
        Ok(self)
    }

    // Méthodes pour UPDATE
    pub fn set<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        self.update_data.insert(column.to_string(), value.into());
        Ok(self)
    }

    pub fn set_multiple(mut self, data: HashMap<String, Value>) -> Result<Self, ApiError> {
        for column in data.keys() {
            self.validate_column(column)?;
        }
        self.update_data.extend(data);
        Ok(self)
    }

    // Méthodes pour INSERT
    pub fn value<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        self.insert_data.insert(column.to_string(), value.into());
        Ok(self)
    }

    pub fn values(mut self, data: HashMap<String, Value>) -> Result<Self, ApiError> {
        for column in data.keys() {
            self.validate_column(column)?;
        }
        self.insert_data.extend(data);
        Ok(self)
    }

    // Validation des colonnes
    fn validate_column(&self, column: &str) -> Result<(), ApiError> {
        if !T::columns().contains(&column) {
            return Err(ApiError::InvalidColumn(column.to_string()));
        }
        Ok(())
    }

    // Construction de la requête SELECT
    pub fn build_select_query(&self) -> QueryBuilder<'_, Postgres> {
        let mut query_builder = QueryBuilder::new("SELECT ");

        if self.distinct {
            query_builder.push("DISTINCT ");
        }

        // Colonnes à sélectionner
        let columns: Vec<String> = match &self.select_columns {
            Some(cols) => cols.clone(),
            None => T::columns().iter().map(|s| s.to_string()).collect(),
        };
        query_builder.push(columns.join(", "));

        query_builder.push(" FROM ");
        query_builder.push(T::table_name());

        // Ajouter les JOINs
        for join in &self.joins {
            query_builder.push(" ");
            query_builder.push(join.join_type.to_sql());
            query_builder.push(" ");
            query_builder.push(&join.table);
            query_builder.push(" ON ");
            query_builder.push(&join.on_condition);
        }

        // Ajouter les conditions WHERE
        if !self.where_clauses.is_empty() {
            query_builder.push(" WHERE ");
            self.build_where_conditions(&mut query_builder);
        }

        // Ajouter GROUP BY
        if !self.group_by.is_empty() {
            query_builder.push(" GROUP BY ");
            query_builder.push(self.group_by.join(", "));
        }

        // Ajouter ORDER BY
        if !self.order_by.is_empty() {
            query_builder.push(" ORDER BY ");
            for (i, order) in self.order_by.iter().enumerate() {
                if i > 0 {
                    query_builder.push(", ");
                }
                query_builder.push(&order.column);
                query_builder.push(" ");
                query_builder.push(order.direction.to_sql());
            }
        }

        // Ajouter LIMIT
        if let Some(limit) = self.limit {
            query_builder.push(" LIMIT ");
            query_builder.push(limit.to_string());
        }

        // Ajouter OFFSET
        if let Some(offset) = self.offset {
            query_builder.push(" OFFSET ");
            query_builder.push(offset.to_string());
        }
        query_builder
    }

    // Construction de la requête UPDATE
    pub fn build_update_query(&self) -> Result<QueryBuilder<'_, Postgres>, ApiError> {
        if self.update_data.is_empty() {
            return Err(ApiError::InvalidQuery(
                "No data provided for update".to_string(),
            ));
        }

        let mut query_builder = QueryBuilder::new("UPDATE ");
        query_builder.push(T::table_name());
        query_builder.push(" SET ");

        let mut first = true;
        for (column, value) in &self.update_data {
            if !first {
                query_builder.push(", ");
            }
            query_builder.push(column);
            query_builder.push(" = ");
            self.bind_value(&mut query_builder, value.clone());
            first = false;
        }

        // Ajouter les conditions WHERE
        if !self.where_clauses.is_empty() {
            query_builder.push(" WHERE ");
            self.build_where_conditions(&mut query_builder);
        }

        Ok(query_builder)
    }

    // Construction de la requête INSERT
    pub fn build_insert_query(&self) -> Result<QueryBuilder<'_, Postgres>, ApiError> {
        if self.insert_data.is_empty() {
            return Err(ApiError::InvalidQuery(
                "No data provided for insert".to_string(),
            ));
        }

        let mut query_builder = QueryBuilder::new("INSERT INTO ");
        query_builder.push(T::table_name());
        query_builder.push(" (");

        let columns: Vec<&String> = self.insert_data.keys().collect();
        query_builder.push(
            columns
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        );
        query_builder.push(") VALUES (");

        for (i, (_, value)) in self.insert_data.iter().enumerate() {
            if i > 0 {
                query_builder.push(", ");
            }
            self.bind_value(&mut query_builder, value.clone());
        }
        query_builder.push(")");

        Ok(query_builder)
    }

    // Construction de la requête DELETE
    pub fn build_delete_query(&self) -> QueryBuilder<'_, Postgres> {
        let mut query_builder = QueryBuilder::new("DELETE FROM ");
        query_builder.push(T::table_name());

        // Ajouter les conditions WHERE
        if !self.where_clauses.is_empty() {
            query_builder.push(" WHERE ");
            self.build_where_conditions(&mut query_builder);
        }

        query_builder
    }

    pub fn build_where_conditions(&self, query_builder: &mut QueryBuilder<'_, Postgres>) {
        self.build_where_clauses(&self.where_clauses, query_builder);
    }

    fn build_where_clauses(
        &self,
        clauses: &[(WhereClause, Option<LogicalOperator>)],
        query_builder: &mut QueryBuilder<'_, Postgres>,
    ) {
        for (i, (clause, logical_op)) in clauses.iter().enumerate() {
            // Ajouter l'opérateur logique si ce n'est pas la première condition
            if i > 0 {
                query_builder.push(" ");
                if let Some(op) = logical_op {
                    query_builder.push(op.to_sql());
                } else {
                    query_builder.push("AND"); // Par défaut
                }
                query_builder.push(" ");
            }

            match clause {
                WhereClause::Condition(condition) => {
                    self.build_single_condition(condition, query_builder);
                }
                WhereClause::Group(group) => {
                    query_builder.push("(");
                    self.build_where_clauses(&group.clauses, query_builder);
                    query_builder.push(")");
                }
            }
        }
    }

    fn build_single_condition(
        &self,
        condition: &WhereCondition,
        query_builder: &mut QueryBuilder<'_, Postgres>,
    ) {
        query_builder.push(&condition.column);
        query_builder.push(" ");
        query_builder.push(condition.operator.to_sql());

        match &condition.operator {
            ComparisonOperator::IsNull | ComparisonOperator::IsNotNull => {
                // Pas de valeur pour ces opérateurs
            }
            ComparisonOperator::In | ComparisonOperator::NotIn => {
                if let Some(values) = &condition.values {
                    query_builder.push(" (");
                    for (j, value) in values.iter().enumerate() {
                        if j > 0 {
                            query_builder.push(", ");
                        }
                        self.bind_value(query_builder, value.clone());
                    }
                    query_builder.push(")");
                }
            }
            ComparisonOperator::Between => {
                if let Some(values) = &condition.values {
                    if values.len() == 2 {
                        query_builder.push(" ");
                        self.bind_value(query_builder, values[0].clone());
                        query_builder.push(" AND ");
                        self.bind_value(query_builder, values[1].clone());
                    }
                }
            }
            _ => {
                if let Some(value) = &condition.value {
                    query_builder.push(" ");
                    self.bind_value(query_builder, value.clone());
                }
            }
        }
    }

    // Méthodes d'exécution pour SELECT
    pub async fn fetch_all(&self, pool: &Pool<Postgres>) -> QueryResult<Vec<T>> {
        let items = self
            .build_select_query()
            .build_query_as::<T>()
            .fetch_all(pool)
            .await
            .map_err(ApiError::Database)?;

        Ok(items)
    }

    pub async fn fetch_one(&self, pool: &Pool<Postgres>) -> QueryResult<T> {
        let item = self
            .build_select_query()
            .build_query_as::<T>()
            .fetch_one(pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => ApiError::NotFound("No record found".to_string()),
                _ => ApiError::Database(e),
            })?;

        Ok(item)
    }

    pub async fn fetch_optional(&self, pool: &Pool<Postgres>) -> QueryResult<Option<T>> {
        let item = self
            .build_select_query()
            .build_query_as::<T>()
            .fetch_optional(pool)
            .await
            .map_err(ApiError::Database)?;

        Ok(item)
    }

    pub async fn count(&self, pool: &Pool<Postgres>) -> QueryResult<i64> {
        let mut query_builder = QueryBuilder::new("SELECT COUNT(*) FROM ");
        query_builder.push(T::table_name());

        // Ajouter les JOINs
        for join in &self.joins {
            query_builder.push(" ");
            query_builder.push(join.join_type.to_sql());
            query_builder.push(" ");
            query_builder.push(&join.table);
            query_builder.push(" ON ");
            query_builder.push(&join.on_condition);
        }

        // Ajouter les conditions WHERE
        if !self.where_clauses.is_empty() {
            query_builder.push(" WHERE ");
            self.build_where_conditions(&mut query_builder);
        }

        let count: (i64,) = query_builder
            .build_query_as()
            .fetch_one(pool)
            .await
            .map_err(ApiError::Database)?;

        Ok(count.0)
    }

    // Méthodes d'exécution pour UPDATE
    pub async fn update(&self, pool: &Pool<Postgres>) -> QueryResult<u64> {
        let mut query = self.build_update_query()?;
        let result = query
            .build()
            .execute(pool)
            .await
            .map_err(ApiError::Database)?;

        Ok(result.rows_affected())
    }

    pub async fn update_returning(&self, pool: &Pool<Postgres>) -> QueryResult<Vec<T>> {
        let mut query = self.build_update_query()?;
        query.push(" RETURNING *");

        let items = query
            .build_query_as::<T>()
            .fetch_all(pool)
            .await
            .map_err(ApiError::Database)?;

        Ok(items)
    }

    // Méthodes d'exécution pour INSERT
    pub async fn insert(&self, pool: &Pool<Postgres>) -> QueryResult<u64> {
        let mut query = self.build_insert_query()?;
        let result = query
            .build()
            .execute(pool)
            .await
            .map_err(ApiError::Database)?;

        Ok(result.rows_affected())
    }

    pub async fn insert_returning(&self, pool: &Pool<Postgres>) -> QueryResult<T> {
        let mut query = self.build_insert_query()?;
        query.push(" RETURNING *");

        let item = query
            .build_query_as::<T>()
            .fetch_one(pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => ApiError::NotFound("No record inserted".to_string()),
                _ => ApiError::Database(e),
            })?;

        Ok(item)
    }

    // Méthodes d'exécution pour DELETE
    pub async fn delete(&self, pool: &Pool<Postgres>) -> QueryResult<u64> {
        let mut query = self.build_delete_query();
        let result = query
            .build()
            .execute(pool)
            .await
            .map_err(ApiError::Database)?;

        Ok(result.rows_affected())
    }

    pub async fn delete_returning(&self, pool: &Pool<Postgres>) -> QueryResult<Vec<T>> {
        let mut query = self.build_delete_query();
        query.push(" RETURNING *");

        let items = query
            .build_query_as::<T>()
            .fetch_all(pool)
            .await
            .map_err(ApiError::Database)?;

        Ok(items)
    }

    /// # Method that must be used to bind values to the query
    /// It handles the conversion of `Value::String` to `String` for proper binding
    fn bind_value(&self, query_builder: &mut QueryBuilder<'_, Postgres>, value: Value) {
        match value {
            // Handle UUIDs represented as strings
            Value::String(ref s) => {
                // Try to parse as UUID, otherwise bind as string
                if let Ok(uuid) = uuid::Uuid::parse_str(s) {
                    query_builder.push_bind(uuid);
                } else {
                    query_builder.push_bind(s.clone());
                }
            }
            _ => {
                query_builder.push_bind(value);
            }
        };
    }
}

// ========== BUILDER POUR GROUPES ==========

/// Builder spécialisé pour construire des groupes de conditions
pub struct GroupBuilder<T: Entry> {
    pub(crate) clauses: Vec<(WhereClause, Option<LogicalOperator>)>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Entry> GroupBuilder<T> {
    pub fn new() -> Self {
        Self {
            clauses: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn where_eq<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::Equal,
            value: Some(value.into()),
            values: None,
        };
        self.clauses.push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_ne<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::NotEqual,
            value: Some(value.into()),
            values: None,
        };
        self.clauses.push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_gt<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::GreaterThan,
            value: Some(value.into()),
            values: None,
        };
        self.clauses.push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_gte<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::GreaterThanOrEqual,
            value: Some(value.into()),
            values: None,
        };
        self.clauses.push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_lt<V: Into<Value>>(mut self, column: &str, value: V) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::LessThan,
            value: Some(value.into()),
            values: None,
        };
        self.clauses.push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_in<V: Into<Value>>(
        mut self,
        column: &str,
        values: Vec<V>,
    ) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let values: Vec<Value> = values.into_iter().map(|v| v.into()).collect();
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::In,
            value: None,
            values: Some(values),
        };
        self.clauses.push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_like<V: Into<Value>>(
        mut self,
        column: &str,
        pattern: V,
    ) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::Like,
            value: Some(pattern.into()),
            values: None,
        };
        self.clauses.push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_null(mut self, column: &str) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::IsNull,
            value: None,
            values: None,
        };
        self.clauses.push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn where_not_null(mut self, column: &str) -> Result<Self, ApiError> {
        self.validate_column(column)?;
        let condition = WhereCondition {
            column: column.to_string(),
            operator: ComparisonOperator::IsNotNull,
            value: None,
            values: None,
        };
        self.clauses.push((WhereClause::Condition(condition), None));
        Ok(self)
    }

    pub fn and(mut self) -> Self {
        if let Some(last) = self.clauses.last_mut() {
            last.1 = Some(LogicalOperator::And);
        }
        self
    }

    pub fn or(mut self) -> Self {
        if let Some(last) = self.clauses.last_mut() {
            last.1 = Some(LogicalOperator::Or);
        }
        self
    }

    fn validate_column(&self, column: &str) -> Result<(), ApiError> {
        if !T::columns().contains(&column) {
            return Err(ApiError::InvalidColumn(column.to_string()));
        }
        Ok(())
    }
}
