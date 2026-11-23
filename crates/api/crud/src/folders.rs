use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::folder::{Folder, FolderUpdateForm},
    newtypes::FolderId,
};
use serde::Deserialize;
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct CreateFolder {
    name: String,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdateFolder {
    name: String,
}

pub async fn create(
    user: AuthenticatedUser,
    Json(data): Json<CreateFolder>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Folder>> {
    ctx.deny_in_demo()?;

    let folder = Folder::resolve_or_create(&ctx.db_pool, user.id, &data.name).await?;
    Ok(Json(folder))
}

pub async fn update(
    user: AuthenticatedUser,
    path: Path<FolderId>,
    Json(data): Json<UpdateFolder>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Folder>> {
    ctx.deny_in_demo()?;

    let folder = Folder::update(
        &ctx.db_pool,
        FolderUpdateForm {
            id: path.into_inner(),
            user_id: user.id,
            name: data.name,
        },
    )
    .await?;
    Ok(Json(folder))
}

pub async fn delete(
    user: AuthenticatedUser,
    path: Path<FolderId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Folder>> {
    ctx.deny_in_demo()?;

    let folder = Folder::delete(&ctx.db_pool, user.id, path.into_inner()).await?;
    Ok(Json(folder))
}
