use actix_web::web::{Data, Json};
use api_utils::{context::WyrmContext, responses::instance::InstanceStatusResponse};
use database::models::instance::LocalInstance;
use wyrm_utils::result::WyrmResult;

pub async fn get(ctx: Data<WyrmContext>) -> WyrmResult<Json<InstanceStatusResponse>> {
    let instance = LocalInstance::get(&ctx.db_pool).await?;
    Ok(Json(instance.into()))
}
