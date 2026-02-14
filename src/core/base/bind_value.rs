use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{Postgres, query_builder::Separated};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum BindValue {
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Uuid(Uuid),
    Json(Value),
    DateTime(DateTime<Utc>),
    Null,
}

impl From<i32> for BindValue {
    fn from(value: i32) -> Self {
        BindValue::I32(value)
    }
}

impl From<i64> for BindValue {
    fn from(value: i64) -> Self {
        BindValue::I64(value)
    }
}

impl From<f64> for BindValue {
    fn from(value: f64) -> Self {
        BindValue::F64(value)
    }
}

impl From<bool> for BindValue {
    fn from(value: bool) -> Self {
        BindValue::Bool(value)
    }
}

impl From<String> for BindValue {
    fn from(value: String) -> Self {
        BindValue::String(value)
    }
}

impl From<&str> for BindValue {
    fn from(value: &str) -> Self {
        BindValue::String(value.to_string())
    }
}

impl From<Uuid> for BindValue {
    fn from(value: Uuid) -> Self {
        BindValue::Uuid(value)
    }
}

impl From<Value> for BindValue {
    fn from(value: Value) -> Self {
        BindValue::Json(value)
    }
}

impl From<DateTime<Utc>> for BindValue {
    fn from(value: DateTime<Utc>) -> Self {
        BindValue::DateTime(value)
    }
}

impl From<Option<String>> for BindValue {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(v) => BindValue::String(v),
            None => BindValue::Null,
        }
    }
}

impl From<Option<&str>> for BindValue {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(v) => BindValue::String(v.to_string()),
            None => BindValue::Null,
        }
    }
}

impl From<Option<Uuid>> for BindValue {
    fn from(value: Option<Uuid>) -> Self {
        match value {
            Some(v) => BindValue::Uuid(v),
            None => BindValue::Null,
        }
    }
}

impl From<Option<DateTime<Utc>>> for BindValue {
    fn from(value: Option<DateTime<Utc>>) -> Self {
        match value {
            Some(v) => BindValue::DateTime(v),
            None => BindValue::Null,
        }
    }
}

impl From<Option<Value>> for BindValue {
    fn from(value: Option<Value>) -> Self {
        match value {
            Some(v) => BindValue::Json(v),
            None => BindValue::Null,
        }
    }
}

impl Default for BindValue {
    fn default() -> Self {
        BindValue::Null
    }
}

impl BindValue {
    pub fn push_bind_unseparated<'qb, 'args, Sep>(
        value: impl Into<BindValue>,
        mut separated: Separated<'qb, 'args, Postgres, Sep>,
    ) -> Separated<'qb, 'args, Postgres, Sep>
    where
        'args: 'qb,
        Sep: Display,
    {
        match value.into() {
            BindValue::I32(v) => separated.push_bind_unseparated(v),
            BindValue::I64(v) => separated.push_bind_unseparated(v),
            BindValue::F64(v) => separated.push_bind_unseparated(v),
            BindValue::String(v) => separated.push_bind_unseparated(v),
            BindValue::Bool(v) => separated.push_bind_unseparated(v),
            BindValue::Uuid(v) => separated.push_bind_unseparated(v),
            BindValue::Json(v) => separated.push_bind_unseparated(v),
            BindValue::DateTime(v) => separated.push_bind_unseparated(v),
            BindValue::Null => separated.push("NULL"),
        };
        separated
    }
}
