use crate::config::config::Config;
use crate::core::base::generic_middleware::GenericMiddleware;
use crate::core::errors::errors::ApiError;
use crate::modules::auth::auth_helpers::verify_token;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpMessage};
use futures::future::LocalBoxFuture;
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
                        return Err(ApiError::Validation(
                            "Missing or invalid Authorization header".to_string(),
                        )
                        .into());
                    }
                };

                let config = req.app_data::<actix_web::web::Data<Config>>();
                let secret = match config {
                    Some(cfg) => &cfg.jwt.secret,
                    None => {
                        return Err(ApiError::Validation(
                            "Missing configuration for JWT secret".to_string(),
                        )
                        .into());
                    }
                };

                let claims = verify_token(&token, secret)
                    .map_err(|e| ApiError::Authentication(e.to_string()))?;

                req.extensions_mut().insert(claims);

                service.call(req).await
            })
        },
    )
}
