use actix_web::web;
use api_api::auth;

/// Reachable without signing in: the sign-in flow, and logout so an expired session can clear its
/// cookie.
pub fn public_config(cfg: &mut web::ServiceConfig) {
    cfg.route("/auth/status", web::get().to(auth::status::status))
        .route("/auth/login", web::post().to(api_crud::auth::login))
        .route("/auth/logout", web::post().to(api_crud::auth::logout));

    #[cfg(feature = "oidc")]
    cfg.route("/auth/oidc/login", web::get().to(auth::oidc::login))
        .route("/auth/oidc/callback", web::get().to(auth::oidc::callback));
}
