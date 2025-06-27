use actix_web::web;

pub mod health_controller;

pub fn routes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(health_controller::health_check);
}
