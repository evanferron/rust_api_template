use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use sqlx::error::Error as SqlxError;
use std::fmt;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Erreur d'authentification: {0}")]
    Authentication(String),

    #[error("Erreur d'autorisation: {0}")]
    Authorization(String),

    #[error("Erreur lors de la validation: {0}")]
    Validation(String),

    #[error("Ressource non trouvée: {0}")]
    NotFound(String),

    #[error("Erreur de conflit: {0}")]
    Conflict(String),

    #[error("Erreur interne du serveur: {0}")]
    InternalServer(String),

    #[error("Erreur de base de données: {0}")]
    Database(SqlxError),

    #[error("Database error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid column name: {0}")]
    InvalidColumn(String),
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub status: u16,
    pub message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Authentication(message) => {
                let error_response = ErrorResponse {
                    status: 401,
                    message: message.to_string(),
                };
                HttpResponse::Unauthorized().json(error_response)
            }
            ApiError::Authorization(message) => {
                let error_response = ErrorResponse {
                    status: 403,
                    message: message.to_string(),
                };
                HttpResponse::Forbidden().json(error_response)
            }
            ApiError::Validation(message) => {
                let error_response = ErrorResponse {
                    status: 400,
                    message: message.to_string(),
                };
                HttpResponse::BadRequest().json(error_response)
            }
            ApiError::NotFound(message) => {
                let error_response = ErrorResponse {
                    status: 404,
                    message: message.to_string(),
                };
                HttpResponse::NotFound().json(error_response)
            }
            ApiError::Conflict(message) => {
                let error_response = ErrorResponse {
                    status: 409,
                    message: message.to_string(),
                };
                HttpResponse::Conflict().json(error_response)
            }
            ApiError::InternalServer(message) => {
                let error_response = ErrorResponse {
                    status: 500,
                    message: message.to_string(),
                };
                HttpResponse::InternalServerError().json(error_response)
            }
            ApiError::Database(err) => {
                let error_response = ErrorResponse {
                    status: 500,
                    message: format!("Erreur de base de données: {}", err),
                };
                HttpResponse::InternalServerError().json(error_response)
            }
            ApiError::Serialization(err) => {
                let error_response = ErrorResponse {
                    status: 500,
                    message: format!("Erreur de sérialisation: {}", err),
                };
                HttpResponse::InternalServerError().json(error_response)
            }
            ApiError::InvalidColumn(column) => {
                let error_response = ErrorResponse {
                    status: 400,
                    message: format!("Nom de colonne invalide: {}", column),
                };
                HttpResponse::BadRequest().json(error_response)
            }
        }
    }
}
