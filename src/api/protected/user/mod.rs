use actix_web::web;
pub mod user_controller;

pub fn routes_config(cfg: &mut web::ServiceConfig) {
    cfg.service(user_controller::get_users)
        .service(user_controller::get_user_by_id)
        .service(user_controller::create_user)
        .service(user_controller::update_user)
        .service(user_controller::delete_user);
}
