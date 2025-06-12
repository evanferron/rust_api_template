use crate::config::config::Config;
use crate::core::base::generic_middleware::GenericMiddleware;
use crate::modules::auth::auth_models::Claims;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpMessage};
use futures::future::LocalBoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode};
use std::rc::Rc;

pub fn auth_middleware<S, B>() -> GenericMiddleware<
    impl Fn(ServiceRequest, Rc<S>) -> LocalBoxFuture<'static, Result<ServiceResponse<B>, Error>> + Clone,
>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    GenericMiddleware::new(
        |req: ServiceRequest,
         service: Rc<S>|
         -> LocalBoxFuture<'static, Result<ServiceResponse<B>, Error>> {
            Box::pin(async move {
                let auth_header = req
                    .headers()
                    .get("Authorization")
                    .and_then(|h| h.to_str().ok());
                let token = match auth_header {
                    Some(h) if h.starts_with("Bearer ") => h[7..].to_string(),
                    _ => {
                        return Err(actix_web::error::ErrorUnauthorized(
                            "Missing or invalid Authorization header",
                        ));
                    }
                };

                let config = req.app_data::<actix_web::web::Data<Config>>();
                let secret = match config {
                    Some(cfg) => &cfg.jwt.secret,
                    None => {
                        return Err(actix_web::error::ErrorInternalServerError(
                            "Missing configuration for JWT secret",
                        ));
                    }
                };

                let token_data: Result<TokenData<Claims>, _> = decode::<Claims>(
                    &token,
                    &DecodingKey::from_secret(secret.as_bytes()),
                    &Validation::new(Algorithm::HS256),
                );
                let claims = match token_data {
                    Ok(data) => data.claims,
                    Err(_) => {
                        return Err(actix_web::error::ErrorUnauthorized(
                            "Invalid token or expired",
                        ));
                    }
                };

                req.extensions_mut().insert(claims);

                service.call(req).await
            })
        },
    )
}
