pub mod user;

use actix_web::web;

pub fn routes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/user").configure(user::routes_config));
}
