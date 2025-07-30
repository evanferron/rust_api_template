use crate::config::config::Config;
use crate::core::errors::errors::ApiError;
use crate::modules::auth::auth_helpers::verify_token;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{Error, HttpMessage};

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());
    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => h[7..].to_string(),
        _ => {
            return Err(ApiError::BadRequest(
                "Missing or invalid Authorization header".to_string(),
            )
            .into());
        }
    };

    let config = req.app_data::<actix_web::web::Data<Config>>();
    let secret = match config {
        Some(cfg) => &cfg.jwt.secret,
        None => {
            return Err(
                ApiError::BadRequest("Missing configuration for JWT secret".to_string()).into(),
            );
        }
    };

    let claims =
        verify_token(&token, secret).map_err(|e| ApiError::Authentication(e.to_string()))?;

    req.extensions_mut().insert(claims);

    next.call(req).await
}
