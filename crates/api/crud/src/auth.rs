use actix_web::{
    HttpRequest,
    HttpResponse,
    http::header,
    web::{Data, Json},
};
use api_utils::context::WyrmContext;
use database::models::local_user::LocalUser;
use serde::Deserialize;
use wyrm_auth::{
    Auth,
    password::{hash_password, verify_password},
    username::normalize_username,
};
use wyrm_utils::{error::AuthError, result::WyrmResult};

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct LoginRequest {
    /// The username the user is trying to log in as.
    username: String,
    /// The user's password.
    password: String,
}

pub async fn login(
    req: HttpRequest,
    Json(data): Json<LoginRequest>,
    ctx: Data<WyrmContext>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    let sessions = auth.sessions()?;

    sessions.check_csrf(&req)?;

    let username = normalize_username(&data.username)?;
    let password = data.password;

    let local_user = LocalUser::get_by_username(&ctx.db_pool, &username).await?;

    // Hash anyway, so an unknown username or a user without a password
    // takes as long to reject as a wrong password.
    let Some(user) = local_user else {
        let _ = hash_password(password).await;
        return Err(AuthError::InvalidCredentials.into());
    };

    let Some(hash) = user.password_hash else {
        let _ = hash_password(password).await;
        return Err(AuthError::InvalidCredentials.into());
    };

    if !verify_password(password, hash.to_string()).await {
        return Err(AuthError::InvalidCredentials.into());
    }

    let authenticated = sessions.create(&ctx.db_pool, user.id).await?;

    let mut response = HttpResponse::NoContent();
    authenticated.set_cookie(&mut response);

    Ok(response.finish())
}

pub async fn logout(
    req: HttpRequest,
    ctx: Data<WyrmContext>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    let sessions = auth.sessions()?;

    sessions.check_csrf(&req)?;

    let cookie = sessions.revoke(&ctx.db_pool, &req).await?;

    Ok(HttpResponse::NoContent()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .cookie(cookie)
        .finish())
}
