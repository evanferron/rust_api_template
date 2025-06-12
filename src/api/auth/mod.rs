use actix_web::web;

pub mod auth_controller;

pub fn routes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(auth_controller::register);
}
