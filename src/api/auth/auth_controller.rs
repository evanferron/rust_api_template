use actix_web::{HttpResponse, Responder, post, web};
use validator::Validate;

use crate::{
    config::models::Services,
    core::errors::errors::{ApiError, ErrorResponse},
    modules::user::user_models::{CreateUserRequest, UserResponse},
};

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Utilisateur créé avec succès", body = UserResponse),
        (status = 400, description = "Erreur de validation", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[post("register")]
pub async fn register(
    services: web::Data<Services>,
    user: web::Json<CreateUserRequest>,
) -> Result<impl Responder, ApiError> {
    user.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    // todo : change this to call auth service
    let created_user = services.user_service.create_user(user.into_inner()).await?;

    Ok(HttpResponse::Created().json(UserResponse::from(created_user)))
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "Connexion réussie", body = UserResponse),
        (status = 401, description = "Email ou mot de passe invalide", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[post("login")]
pub async fn login(
    services: web::Data<Services>,
    user: web::Json<CreateUserRequest>,
) -> Result<impl Responder, ApiError> {
    user.validate()
        .map_err(|e| ApiError::Validation(e.to_string()))?;

    let authenticated_user = services
        .auth_service
        .authenticate_user(user.email.clone(), user.password.clone())
        .await?;

    Ok(HttpResponse::Ok().json(UserResponse::from(authenticated_user)))
}
