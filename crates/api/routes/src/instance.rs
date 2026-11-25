use actix_web::web;

pub fn public_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/instance", web::get().to(api_api::instance::get))
        .route("/instance/setup", web::post().to(api_crud::instance::setup));
}
