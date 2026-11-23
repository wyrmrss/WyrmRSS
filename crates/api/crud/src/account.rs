use actix_web::{
    HttpRequest,
    HttpResponse,
    http::header,
    web::{Data, Json},
};
use api_utils::context::WyrmContext;
use database::models::{
    instance::{LocalInstance, RegistrationMode},
    local_user::{LocalUser, LocalUserInsertForm, LocalUserUpdateForm},
};
use serde::Deserialize;
use wyrm_auth::{
    Auth,
    authenticated_user::AuthenticatedUser,
    password::{hash_password, validate_password, verify_password},
    username::normalize_username,
};
use wyrm_utils::{
    error::{
        AuthError::RegistrationClosed,
        DatabaseError::{Conflict, UniqueViolation},
        WyrmError,
    },
    result::WyrmResult,
};

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct RegisterAccount {
    /// New user's username.
    username: String,
    /// New user's password.
    password: String,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, optional_fields)]
pub struct DeleteAccount {
    /// Required when the user has a password.
    password: Option<String>,
    /// Required for users without a password (OIDC-only), as a typed confirmation.
    username: Option<String>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, optional_fields)]
pub struct UpdatePassword {
    current_password: Option<String>,
    new_password: String,
}

pub async fn register(
    req: HttpRequest,
    Json(data): Json<RegisterAccount>,
    ctx: Data<WyrmContext>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    ctx.deny_in_demo()?;

    let sessions = auth.sessions()?;

    sessions.check_csrf(&req)?;

    let registration_mode = LocalInstance::get(&ctx.db_pool).await?.registration_mode;

    if registration_mode == RegistrationMode::Closed {
        return Err(WyrmError::Auth(RegistrationClosed));
    }

    let username = normalize_username(&data.username)?;

    validate_password(&data.password)?;

    let password_hash = hash_password(data.password).await?;

    let form = LocalUserInsertForm {
        username,
        password_hash,
    };

    let local_user = LocalUser::create(&ctx.db_pool, form)
        .await
        .map_err(|e| match e {
            WyrmError::Database(UniqueViolation(_)) => {
                WyrmError::Database(Conflict("username is already taken".into()))
            }
            e => e,
        })?;

    let authenticated = sessions.create(&ctx.db_pool, local_user).await?;

    let mut response = HttpResponse::Created();

    authenticated.set_cookie(&mut response);

    Ok(response.finish())
}

pub async fn update_password(
    user: AuthenticatedUser,
    Json(data): Json<UpdatePassword>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<HttpResponse> {
    ctx.deny_in_demo()?;

    let local_user = LocalUser::get(&ctx.db_pool, user.id).await?;

    validate_password(&data.new_password)?;

    if data.current_password.as_deref() == Some(data.new_password.as_str()) {
        return Err(WyrmError::InvalidInput(
            "new password must differ from the current one".into(),
        ));
    }

    // OIDC-only users have no password yet, so this sets their first one.
    if let Some(hash) = &local_user.password_hash {
        let current = data
            .current_password
            .ok_or_else(|| WyrmError::InvalidInput("current password is required".into()))?;

        if !verify_password(current, hash.clone()).await {
            return Err(WyrmError::InvalidInput(
                "current password is incorrect".into(),
            ));
        }
    }

    let password_hash = hash_password(data.new_password).await?;

    let mut conn = ctx.db_pool.get().await?;

    LocalUser::update(
        &mut conn,
        user.id,
        LocalUserUpdateForm {
            username: None,
            password_hash: Some(password_hash),
            oidc_subject: None,
        },
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Deletes the signed-in user's account and everything they own,
/// after they confirm with their password, or their username if
/// they have no password.
pub async fn delete(
    req: HttpRequest,
    user: AuthenticatedUser,
    Json(data): Json<DeleteAccount>,
    ctx: Data<WyrmContext>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    ctx.deny_in_demo()?;

    let sessions = auth.sessions()?;

    let local_user = LocalUser::get(&ctx.db_pool, user.id).await?;

    match &local_user.password_hash {
        Some(hash) => {
            let password = data
                .password
                .ok_or_else(|| WyrmError::InvalidInput("password is required".into()))?;

            if !verify_password(password, hash.clone()).await {
                return Err(WyrmError::InvalidInput("password is incorrect".into()));
            }
        }
        None => {
            let confirmed = data
                .username
                .is_some_and(|typed| typed.trim().eq_ignore_ascii_case(&local_user.username));

            if !confirmed {
                return Err(WyrmError::InvalidInput(
                    "type your username to confirm".into(),
                ));
            }
        }
    }

    LocalUser::delete(&ctx.db_pool, user.id).await?;

    // Clear the browser's cookie.
    let cookie = sessions.revoke(&ctx.db_pool, &req).await?;

    Ok(HttpResponse::NoContent()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .cookie(cookie)
        .finish())
}
