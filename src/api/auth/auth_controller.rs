use actix_web::{HttpResponse, Responder, post, web};
use validator::Validate;

use crate::{
    config::{config::Config, models::Services},
    core::errors::errors::{ApiError, ErrorResponse},
    modules::{
        auth::{auth_helpers::generate_jwt, auth_models::AuthResponse},
        user::user_models::CreateUserRequest,
    },
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = CreateUserRequest,
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
    user: web::Json<CreateUserRequest>,
) -> Result<impl Responder, ApiError> {
    user.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let created_user = services.auth_service.create_user(user.into_inner()).await?;

    let token = generate_jwt(
        created_user.id,
        config.jwt.secret.as_str(),
        config.jwt.expiration,
    );

    let refresh_token = generate_jwt(
        created_user.id,
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
    request_body = CreateUserRequest,
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
    user: web::Json<CreateUserRequest>,
) -> Result<impl Responder, ApiError> {
    user.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let authenticated_user = services
        .auth_service
        .authenticate_user(user.email.clone(), user.password.clone())
        .await?;

    let token = generate_jwt(
        authenticated_user.id,
        config.jwt.secret.as_str(),
        config.jwt.expiration,
    );

    let refresh_token = generate_jwt(
        authenticated_user.id,
        config.jwt.refresh_secret.as_str(),
        config.jwt.refresh_expiration,
    );

    Ok(HttpResponse::Ok().json(AuthResponse {
        id: authenticated_user.id,
        username: authenticated_user.username,
        email: authenticated_user.email,
        token: token.map_err(|e| ApiError::InternalServer(e.to_string()))?,
        refresh_token: refresh_token.map_err(|e| ApiError::InternalServer(e.to_string()))?,
    }))
}
