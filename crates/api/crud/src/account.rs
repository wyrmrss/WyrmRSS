use actix_web::{
    HttpResponse,
    web::{Data, Json},
};
use api_utils::context::WyrmContext;
use database::models::local_user::{LocalUser, LocalUserUpdateForm};
use serde::Deserialize;
use wyrm_auth::{
    authenticated_user::AuthenticatedUser,
    password::{hash_password, validate_password, verify_password},
};
use wyrm_utils::{error::WyrmError, result::WyrmResult};

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdatePassword {
    current_password: Option<String>,
    new_password: String,
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
