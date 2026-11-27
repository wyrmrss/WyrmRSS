use actix_web::web;
use api_api::account;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/account")
            .route("", web::get().to(account::get))
            .route("", web::delete().to(api_crud::account::delete))
            .route(
                "/password",
                web::post().to(api_crud::account::update_password),
            ),
    );
}
