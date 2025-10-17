use utoipa::OpenApi;
use crate::api;
use crate::api::health::health_controller::{HealthResponse};
use crate::modules::user::user_models::{
    UserResponse, 
    CreateUserRequest, 
    UpdateUserRequest
};
use crate::core::errors::errors::ErrorResponse;

// OpenAPI configuration for the API
#[derive(OpenApi)]
#[openapi(
    paths(
        api::health::health_controller::health_check,
        api::protected::user::user_controller::get_users,
        api::protected::user::user_controller::get_user_by_id,
        api::protected::user::user_controller::create_user,
        api::protected::user::user_controller::update_user,
        api::protected::user::user_controller::delete_user,
    ),
    components(
        schemas(
            UserResponse, 
            CreateUserRequest,
            UpdateUserRequest,
            HealthResponse,
            ErrorResponse
        )
    ),
    tags(
        (name = "health", description = "Endpoints de vérification de santé"),
        (name = "users", description = "API de gestion des utilisateurs")
    ),
    info(
        title = "API Template",
        version = env!("CARGO_PKG_VERSION"),
        description = "Documentation de l'API Template en Rust avec Actix-Web",
        contact(
            name = "Evan",
            email = "evan.ferron53@gmail.com"
        ),
    )
)]
pub struct ApiDoc;