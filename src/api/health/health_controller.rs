use crate::{core::errors::errors::ErrorResponse, db::connection::check_connection};
use actix_web::{Responder, get, web};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize, utoipa::ToSchema)]
pub struct HealthResponse {
    status: String,
    timestamp: String,
    version: String,
    database: bool,
}

#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "API en service", body = HealthResponse),
        (status = 500, description = "Problème de connexion à la base de données", body = ErrorResponse)
    )
)]
#[get("")]
pub async fn health_check(pool: web::Data<PgPool>) -> impl Responder {
    let db_status = check_connection(&pool).await.is_ok();
    let status = if db_status { "ok" } else { "error" };

    web::Json(HealthResponse {
        status: status.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        database: db_status,
    })
}
