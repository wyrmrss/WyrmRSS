use actix_web::web;

pub fn public_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/instance/status", web::get().to(api_api::instance::status))
        .route("/instance/setup", web::post().to(api_crud::instance::setup));
}
