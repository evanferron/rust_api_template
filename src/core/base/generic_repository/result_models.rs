#[derive(sqlx::FromRow)]
pub struct CountResult {
    pub count: i64,
}

#[derive(sqlx::FromRow)]
pub struct ExistResult {
    pub exist: bool,
}
