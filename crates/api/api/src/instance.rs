use actix_web::web::{Data, Json};
use api_utils::{
    context::WyrmContext,
    responses::instance::{LocalInstanceConfigResponse, LocalInstanceStatusResponse},
};
use database::models::instance::LocalInstance;
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_utils::result::WyrmResult;

/// Public, so the frontend can read the setup and sign-in configuration.
pub async fn status(ctx: Data<WyrmContext>) -> WyrmResult<Json<LocalInstanceStatusResponse>> {
    let instance = LocalInstance::get(&ctx.db_pool).await?;
    Ok(Json(instance.into()))
}

/// Admin-only. Returns the instance-wide configuration.
pub async fn get(
    user: AuthenticatedUser,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<LocalInstanceConfigResponse>> {
    user.require_admin()?;

    let instance = LocalInstance::get(&ctx.db_pool).await?;
    Ok(Json(instance.into()))
}
