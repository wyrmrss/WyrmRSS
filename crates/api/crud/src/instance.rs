use actix_web::{
    HttpResponse,
    http::header,
    web::{Data, Json},
};
use api_utils::context::WyrmContext;
use database::models::instance::LocalInstance;
use serde::Deserialize;
use wyrm_auth::{
    Auth,
    password::{hash_password, validate_password},
    username::normalize_username,
};
use wyrm_utils::{
    error::{
        AuthError::{self, InvalidSetupToken},
        DatabaseError::Conflict,
        WyrmError,
    },
    result::WyrmResult,
};

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct InstanceSetup {
    /// The setup token retrieved from the server logs.
    token: String,
    /// The username for the admin/owner user.
    username: String,
    /// The password for the admin/owner user.
    password: String,
}

/// Sets up a new instance: checks the setup token from the server log, gives the
/// admin (user 1) a username and password, and signs them in with a session cookie.
pub async fn setup(
    Json(data): Json<InstanceSetup>,
    auth: Data<Auth>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<HttpResponse> {
    if LocalInstance::get(&ctx.db_pool).await?.is_setup {
        return Err(WyrmError::Database(Conflict(
            "instance already set up".into(),
        )));
    }

    if !auth.setup_token_matches(&data.token) {
        return Err(WyrmError::Auth(InvalidSetupToken));
    }

    let sessions = auth
        .sessions()
        .ok_or_else(|| AuthError::InvalidAuthConfig("public_url is required".into()))?;

    let username = normalize_username(&data.username)?;

    validate_password(&data.password)?;

    let password_hash = hash_password(data.password).await?;

    let admin = LocalInstance::complete_setup(&ctx.db_pool, username, password_hash).await?;

    let authenticated = sessions.create(&ctx.db_pool, admin.id).await?;

    let mut response = HttpResponse::NoContent();
    response.insert_header((header::CACHE_CONTROL, "no-store"));

    authenticated.set_cookie(&mut response);

    Ok(response.finish())
}
