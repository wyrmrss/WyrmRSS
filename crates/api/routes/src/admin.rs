use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin").route("/instance", web::patch().to(api_crud::instance::update)),
    );
}
