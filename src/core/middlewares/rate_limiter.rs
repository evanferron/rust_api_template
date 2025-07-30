use actix_web::{
    Error,
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web,
};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::errors::errors::ApiError;

// Structure pour stocker les informations de rate limiting par IP
#[derive(Debug, Clone)]
struct RateLimitInfo {
    count: u32,
    window_start: Instant,
}

// Store global pour le rate limiting (en production, utilisez Redis)
lazy_static::lazy_static! {
    static ref RATE_LIMIT_STORE: Arc<Mutex<HashMap<String, RateLimitInfo>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

// Configuration du rate limiter
#[derive(Clone, Debug)]
pub struct RateLimiterConfig {
    pub max_requests: u32,
    pub window_duration: Duration,
    pub identifier_header: Option<String>, // Header personnalisé pour identifier l'utilisateur
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_duration: Duration::from_secs(60),
            identifier_header: None,
        }
    }
}

// Fonction pour extraire l'identifiant du client
fn get_client_identifier(req: &ServiceRequest, config: &RateLimiterConfig) -> String {
    // Si un header personnalisé est défini, l'utiliser en priorité
    if let Some(header_name) = &config.identifier_header {
        if let Some(header_value) = req.headers().get(header_name) {
            if let Ok(value) = header_value.to_str() {
                return format!("header:{}", value);
            }
        }
    }

    // Sinon, utiliser l'IP
    let remote_addr = req
        .connection_info()
        .peer_addr()
        .unwrap_or("unknown")
        .to_string();

    format!("ip:{}", remote_addr)
}

// Middleware de rate limiting
pub async fn rate_limiter_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let config = req
        .app_data::<web::Data<RateLimiterConfig>>()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("RateLimiterConfig not found"))?;
    let method = req.method().clone();
    let path = req.path().to_owned();
    let client_id = get_client_identifier(&req, &config);

    let now = Instant::now();
    let should_allow = {
        let mut store = RATE_LIMIT_STORE.lock().unwrap();

        // Nettoyer les entrées expirées périodiquement
        if store.len() > 1000 {
            // Nettoyage quand le store devient trop grand
            store.retain(|_, info| now.duration_since(info.window_start) < config.window_duration);
        }

        match store.get_mut(&client_id) {
            Some(info) => {
                // Vérifier si la fenêtre a expiré
                if now.duration_since(info.window_start) >= config.window_duration {
                    info.count = 1;
                    info.window_start = now;
                    debug!(
                        client_id = %client_id,
                        count = 1,
                        max_requests = config.max_requests,
                        "Rate_limit_window_reset"
                    );
                    true
                } else if info.count < config.max_requests {
                    info.count += 1;
                    debug!(
                        client_id = %client_id,
                        count = info.count,
                        max_requests = config.max_requests,
                        "Rate_limit_request_allowed"
                    );
                    true
                } else {
                    false
                }
            }
            None => {
                store.insert(
                    client_id.clone(),
                    RateLimitInfo {
                        count: 1,
                        window_start: now,
                    },
                );
                debug!(
                    client_id = %client_id,
                    count = 1,
                    max_requests = config.max_requests,
                    "Rate_limit_new_client"
                );
                true
            }
        }
    };

    if should_allow {
        next.call(req).await
    } else {
        warn!(
            method = %method,
            path = %path,
            client_id = %client_id,
            max_requests = config.max_requests,
            window_seconds = config.window_duration.as_secs(),
            "Rate_limit_exceeded"
        );

        Err(ApiError::RateLimitExceeded {
            client_id,
            max_requests: config.max_requests,
            window_duration: config.window_duration,
        }
        .into())
    }
}
