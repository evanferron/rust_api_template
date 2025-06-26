use actix_web::{HttpResponse, Responder, post, web};
use validator::Validate;

use crate::{
    config::{config::Config, models::Services},
    core::errors::errors::{ApiError, ErrorResponse},
    modules::auth::{
        auth_helpers::{generate_jwt, verify_token},
        auth_models::{
            AuthResponse, LoginRequest, RefreshRequest, RefreshResponse, RegisterRequest, Sub,
        },
    },
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Utilisateur créé avec succès", body = AuthResponse),
        (status = 400, description = "Erreur de validation", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[post("register")]
pub async fn register(
    services: web::Data<Services>,
    config: web::Data<Config>,
    user: web::Json<RegisterRequest>,
) -> Result<impl Responder, ApiError> {
    user.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let created_user = services.auth_service.create_user(user.into_inner()).await?;

    let sub = Sub {
        id: created_user.id,
        email: created_user.email.clone(),
        is_admin: None,
    };

    let token = generate_jwt(
        sub.clone(),
        config.jwt.secret.as_str(),
        config.jwt.expiration,
    );

    let refresh_token = generate_jwt(
        sub.clone(),
        config.jwt.refresh_secret.as_str(),
        config.jwt.refresh_expiration,
    );

    Ok(HttpResponse::Created().json(AuthResponse {
        id: created_user.id,
        username: created_user.username,
        email: created_user.email,
        token: token.map_err(|e| ApiError::InternalServer(e.to_string()))?,
        refresh_token: refresh_token.map_err(|e| ApiError::InternalServer(e.to_string()))?,
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Connexion réussie", body = AuthResponse),
        (status = 401, description = "Email ou mot de passe invalide", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[post("login")]
pub async fn login(
    services: web::Data<Services>,
    config: web::Data<Config>,
    user: web::Json<LoginRequest>,
) -> Result<impl Responder, ApiError> {
    user.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let authenticated_user = services
        .auth_service
        .authenticate_user(user.email.clone(), user.password.clone())
        .await?;

    let sub = Sub {
        id: authenticated_user.id,
        email: authenticated_user.email.clone(),
        is_admin: None,
    };

    let token = generate_jwt(
        sub.clone(),
        config.jwt.secret.as_str(),
        config.jwt.expiration,
    );

    let refresh_token = generate_jwt(
        sub.clone(),
        config.jwt.refresh_secret.as_str(),
        config.jwt.refresh_expiration,
    );

    Ok(web::Json(AuthResponse {
        id: authenticated_user.id,
        username: authenticated_user.username,
        email: authenticated_user.email,
        token: token.map_err(|e| ApiError::InternalServer(e.to_string()))?,
        refresh_token: refresh_token.map_err(|e| ApiError::InternalServer(e.to_string()))?,
    }))
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token rafraîchi avec succès", body = RefreshResponse),
        (status = 401, description = "Token invalide ou expiré", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[post("refresh")]
pub async fn refresh(
    config: web::Data<Config>,
    request: web::Json<RefreshRequest>,
) -> Result<impl Responder, ApiError> {
    // Validate the refresh token
    let claims = verify_token(&request.refresh_token, &config.jwt.refresh_secret)
        .map_err(|e| ApiError::Authentication(e.to_string()))?;

    let user = claims.user;

    let new_token = generate_jwt(
        user.clone(),
        config.jwt.secret.as_str(),
        config.jwt.expiration,
    );

    let new_refresh_token = generate_jwt(
        user,
        config.jwt.refresh_secret.as_str(),
        config.jwt.refresh_expiration,
    );

    Ok(web::Json(RefreshResponse {
        token: new_token.map_err(|e| ApiError::InternalServer(e.to_string()))?,
        refresh_token: new_refresh_token.map_err(|e| ApiError::InternalServer(e.to_string()))?,
    }))
}
