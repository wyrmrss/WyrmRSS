use actix_web::web;
use api_api::auth;
#[cfg(feature = "oidc")]
use api_api::oidc;

/// Reachable without signing in: sign-in, sign-up, and logout so an expired session can clear its
/// cookie.
pub fn public_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/auth/status", web::get().to(auth::status))
        .route(
            "/auth/register",
            web::post().to(api_crud::account::register),
        )
        .route("/auth/login", web::post().to(api_crud::auth::login))
        .route("/auth/logout", web::post().to(api_crud::auth::logout));

    #[cfg(feature = "oidc")]
    cfg.route("/auth/oidc/login", web::get().to(oidc::login))
        .route("/auth/oidc/callback", web::get().to(oidc::callback));
}
