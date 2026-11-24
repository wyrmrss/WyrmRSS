use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use api_utils::{context::WyrmContext, responses::instance::InstanceConfigResponse};
use database::models::instance::{LocalInstance, LocalInstanceUpdateForm, RegistrationMode};
use serde::Deserialize;
use wyrm_auth::{
    Auth,
    authenticated_user::AuthenticatedUser,
    password::{hash_password, validate_password},
    username::normalize_username,
};
use wyrm_utils::{
    error::{AuthError::InvalidSetupToken, DatabaseError::Conflict, WyrmError},
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

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateInstance {
    /// Who can create an account.
    registration_mode: RegistrationMode,
}

/// Updates instance-level configuration. Admins only.
pub async fn update(
    user: AuthenticatedUser,
    Json(data): Json<UpdateInstance>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<InstanceConfigResponse>> {
    ctx.deny_in_demo()?;

    user.require_admin()?;

    let form = LocalInstanceUpdateForm {
        registration_mode: data.registration_mode,
    };

    let instance = LocalInstance::update(&ctx.db_pool, form).await?;

    Ok(Json(instance.into()))
}

/// Sets up a new instance: checks the setup token from the server log, gives the
/// admin (user 1) a username and password, and signs them in with a session cookie.
pub async fn setup(
    req: HttpRequest,
    Json(data): Json<InstanceSetup>,
    auth: Data<Auth>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<HttpResponse> {
    let sessions = auth.sessions()?;

    sessions.check_csrf(&req)?;

    if LocalInstance::get(&ctx.db_pool).await?.is_setup {
        return Err(WyrmError::Database(Conflict(
            "instance already set up".into(),
        )));
    }

    if !auth.setup_token_matches(&data.token) {
        return Err(WyrmError::Auth(InvalidSetupToken));
    }

    let username = normalize_username(&data.username)?;

    validate_password(&data.password)?;

    let password_hash = hash_password(data.password).await?;

    let admin = LocalInstance::complete_setup(&ctx.db_pool, username, password_hash).await?;

    let authenticated = sessions.create(&ctx.db_pool, admin).await?;

    let mut response = HttpResponse::NoContent();
    authenticated.set_cookie(&mut response);

    Ok(response.finish())
}
