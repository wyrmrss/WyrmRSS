use actix_web::web::{Data, Json};
use api_utils::context::WyrmContext;
use database::models::host_credential::HostCredential;
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_utils::result::WyrmResult;

pub async fn list(
    user: AuthenticatedUser,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Vec<HostCredential>>> {
    let host_credentials = HostCredential::get_all_for_user(&ctx.db_pool, user.id).await?;
    Ok(Json(host_credentials))
}
