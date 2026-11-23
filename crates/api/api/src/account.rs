use actix_web::web::{Data, Json};
use api_utils::context::WyrmContext;
use database::{models::local_user::LocalUser, newtypes::UserId};
use serde::Serialize;
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_utils::result::WyrmResult;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AccountResponse {
    id: UserId,
    username: String,
    has_password: bool,
    oidc_linked: bool,
    is_admin: bool,
}

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
