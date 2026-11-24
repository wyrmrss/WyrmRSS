use actix_web::web::{Data, Json};
use api_utils::{context::WyrmContext, responses::instance::LocalInstanceStatusResponse};
use database::models::instance::LocalInstance;
use wyrm_utils::result::WyrmResult;

pub async fn get(ctx: Data<WyrmContext>) -> WyrmResult<Json<LocalInstanceStatusResponse>> {
    let instance = LocalInstance::get(&ctx.db_pool).await?;
    Ok(Json(instance.into()))
}
