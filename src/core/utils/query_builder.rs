use sqlx::Error as SqlxError;

// Définition des filtres pour les requêtes avancées
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
    Like,
    ILike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: Option<serde_json::Value>,
}

// Options pour les requêtes
pub struct QueryOptions {
    pub filters: Vec<Filter>,
    pub sort: Vec<(String, bool)>, // (field, is_ascending)
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub relations: Vec<String>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            filters: vec![],
            sort: vec![],
            limit: None,
            offset: None,
            relations: vec![],
        }
    }
}

// Builder pour QueryOptions
pub struct QueryBuilder {
    options: QueryOptions,
    table_name: String,
}

impl QueryBuilder {
    pub fn new(table_name: String) -> Self {
        Self {
            options: QueryOptions::default(),
            table_name,
        }
    }

    pub fn new_with_option(table_name: String, option: QueryOptions) -> Self {
        Self {
            options: option,
            table_name,
        }
    }

    pub fn filter(
        mut self,
        field: &str,
        operator: FilterOperator,
        value: Option<serde_json::Value>,
    ) -> Self {
        self.options.filters.push(Filter {
            field: field.to_string(),
            operator,
            value,
        });
        self
    }

    pub fn sort(mut self, field: &str, ascending: bool) -> Self {
        self.options.sort.push((field.to_string(), ascending));
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.options.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.options.offset = Some(offset);
        self
    }

    pub fn with_relation(mut self, relation: &str) -> Self {
        self.options.relations.push(relation.to_string());
        self
    }

    pub fn build(self) -> QueryOptions {
        self.options
    }

    pub fn build_query(&self) -> Result<(String, Vec<serde_json::Value>), SqlxError> {
        let mut query = format!("SELECT * FROM {}", self.table_name);
        let mut params: Vec<serde_json::Value> = vec![];
        let mut param_index = 1;

        if !self.options.filters.is_empty() {
            query.push_str(" WHERE ");
            let mut first = true;

            for filter in &self.options.filters {
                if !first {
                    query.push_str(" AND ");
                }
                first = false;

                match filter.operator {
                    FilterOperator::Eq => {
                        query.push_str(&format!("{} = ${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                    }
                    FilterOperator::Ne => {
                        query.push_str(&format!("{} != ${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                    }
                    FilterOperator::Gt => {
                        query.push_str(&format!("{} > ${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                    }
                    FilterOperator::Lt => {
                        query.push_str(&format!("{} < ${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                    }
                    FilterOperator::Ge => {
                        query.push_str(&format!("{} >= ${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                    }
                    FilterOperator::Le => {
                        query.push_str(&format!("{} <= ${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                    }
                    FilterOperator::Like => {
                        query.push_str(&format!("{} LIKE ${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                    }
                    FilterOperator::ILike => {
                        query.push_str(&format!("{} ILIKE ${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                    }
                    FilterOperator::In => {
                        query.push_str(&format!("{} IN (${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                        query.push_str(")");
                    }
                    FilterOperator::NotIn => {
                        query.push_str(&format!("{} NOT IN (${}", filter.field, param_index));
                        params.push(filter.value.clone().unwrap_or(serde_json::Value::Null));
                        param_index += 1;
                        query.push_str(")");
                    }
                    FilterOperator::IsNull => {
                        query.push_str(&format!("{} IS NULL", filter.field));
                    }
                    FilterOperator::IsNotNull => {
                        query.push_str(&format!("{} IS NOT NULL", filter.field));
                    }
                }
            }
        }

        if !self.options.sort.is_empty() {
            query.push_str(" ORDER BY ");
            let mut first = true;

            for (field, is_ascending) in &self.options.sort {
                if !first {
                    query.push_str(", ");
                }
                first = false;

                query.push_str(&format!(
                    "{} {}",
                    field,
                    if *is_ascending { "ASC" } else { "DESC" }
                ));
            }
        }

        if let Some(limit) = self.options.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.options.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        Ok((query, params))
    }
}
