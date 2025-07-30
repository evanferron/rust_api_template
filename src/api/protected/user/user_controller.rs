use crate::config::models::Services;
use crate::core::errors::errors::{ApiError, ErrorResponse};
use crate::modules::user::user_models::{
    CreateUserRequest, UpdateUserRequest, UserIdPath, UserResponse,
};
use actix_web::{HttpResponse, Responder, delete, get, post, put, web};
use validator::Validate;

#[utoipa::path(
    get,
    path = "/api/protected/user",
    tag = "users",
    responses(
        (status = 200, description = "Liste des utilisateurs", body = Vec<UserResponse>),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[get("")]
pub async fn get_users(services: web::Data<Services>) -> Result<impl Responder, ApiError> {
    let users = services.user_service.get_users().await?;

    let user_responses: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();

    Ok(web::Json(user_responses))
}

#[utoipa::path(
    get,
    path = "/api/protected/user/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "ID de l'utilisateur")
    ),
    responses(
        (status = 200, description = "Utilisateur trouvé", body = UserResponse),
        (status = 404, description = "Utilisateur non trouvé", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[get("/{id}")]
pub async fn get_user_by_id(
    path: web::Path<UserIdPath>,
    services: web::Data<Services>,
) -> Result<impl Responder, ApiError> {
    let user = services.user_service.get_user_by_id(path.id).await?;
    Ok(web::Json(UserResponse::from(user)))
}

#[utoipa::path(
    post,
    path = "/api/protected/user",
    tag = "users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Utilisateur créé", body = UserResponse),
        (status = 400, description = "Données invalides", body = ErrorResponse),
        (status = 409, description = "Email déjà utilisé", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[post("")]
pub async fn create_user(
    services: web::Data<Services>,
    user: web::Json<CreateUserRequest>,
) -> Result<impl Responder, ApiError> {
    // Validation des données
    if let Err(e) = user.validate() {
        return Err(ApiError::BadRequest(format!("{}", e)));
    }

    let user = services.user_service.create_user(user.into_inner()).await?;

    Ok(HttpResponse::Created().json(UserResponse::from(user)))
}

#[utoipa::path(
    put,
    path = "/api/protected/user/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "ID de l'utilisateur")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Utilisateur mis à jour", body = UserResponse),
        (status = 400, description = "Données invalides", body = ErrorResponse),
        (status = 404, description = "Utilisateur non trouvé", body = ErrorResponse),
        (status = 409, description = "Email déjà utilisé", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[put("/{id}")]
pub async fn update_user(
    services: web::Data<Services>,
    path: web::Path<UserIdPath>,
    req: web::Json<UpdateUserRequest>,
) -> Result<impl Responder, ApiError> {
    // Validation des données
    if let Err(e) = req.validate() {
        return Err(ApiError::BadRequest(format!("{}", e)));
    }

    let user = services
        .user_service
        .update_user(
            path.id,
            req.username.clone(),
            req.email.clone(),
            req.password.clone(),
        )
        .await?;

    Ok(web::Json(UserResponse::from(user)))
}

#[utoipa::path(
    delete,
    path = "/api/protected/user/{id}",
    tag = "users",
    params(
        ("id" = Uuid, Path, description = "ID de l'utilisateur")
    ),
    responses(
        (status = 204, description = "Utilisateur supprimé"),
        (status = 404, description = "Utilisateur non trouvé", body = ErrorResponse),
        (status = 500, description = "Erreur interne du serveur", body = ErrorResponse)
    )
)]
#[delete("/{id}")]
pub async fn delete_user(
    services: web::Data<Services>,
    path: web::Path<UserIdPath>,
) -> Result<impl Responder, ApiError> {
    services.user_service.delete_user(path.id).await?;

    Ok(HttpResponse::NoContent().finish())
}
