use actix_web::web::{Data, Json};
use api_utils::{context::WyrmContext, responses::account::AccountResponse};
use database::models::local_user::LocalUser;
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_utils::result::WyrmResult;

pub async fn get(
    user: AuthenticatedUser,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<AccountResponse>> {
    let local_user = LocalUser::get(&ctx.db_pool, user.id).await?;

    Ok(Json(AccountResponse {
        id: local_user.id,
        username: local_user.username,
        has_password: local_user.password_hash.is_some(),
        oidc_linked: local_user.oidc_subject.is_some(),
        is_admin: local_user.is_admin,
    }))
}
