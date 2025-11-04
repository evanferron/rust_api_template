use serde_json::Value;
use uuid::Uuid;

#[derive(Debug)]
#[derive(Clone)]
pub enum BindValue {
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
    String(String),
    Uuid(Uuid),
    Json(Value),
}
