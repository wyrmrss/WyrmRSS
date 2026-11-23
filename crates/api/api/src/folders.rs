use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{models::folder::Folder, newtypes::FolderId};
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_utils::result::WyrmResult;

pub async fn get(
    user: AuthenticatedUser,
    path: Path<FolderId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Folder>> {
    let folder = Folder::get(&ctx.db_pool, user.id, path.into_inner()).await?;
    Ok(Json(folder))
}

pub async fn list(
    user: AuthenticatedUser,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Vec<Folder>>> {
    let folders = Folder::get_all(&ctx.db_pool, user.id).await?;
    Ok(Json(folders))
}
