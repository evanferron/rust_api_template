use crate::modules::auth::auth_middleware::auth_middleware;
use actix_web::{middleware, web};

mod auth;
mod health;
mod protected;
pub mod swagger;

pub fn routes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .service(web::scope("/health").configure(health::routes_config))
            .service(web::scope("/auth").configure(auth::routes_config))
            .service(
                web::scope("/protected")
                    .wrap(middleware::from_fn(auth_middleware))
                    .configure(protected::routes_config),
            ),
    );
}
