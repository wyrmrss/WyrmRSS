use actix_web::web::{Data, Json};
use api_utils::context::WyrmContext;
use database::models::instance::{LocalInstance, RegistrationMode};
use serde::Serialize;
use wyrm_utils::result::WyrmResult;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct LocalInstanceResponse {
    is_setup: bool,
    registration_mode: RegistrationMode,
}

pub async fn get(ctx: Data<WyrmContext>) -> WyrmResult<Json<LocalInstanceResponse>> {
    let instance = LocalInstance::get(&ctx.db_pool).await?;
    Ok(Json(LocalInstanceResponse {
        is_setup: instance.is_setup,
        registration_mode: instance.registration_mode,
    }))
}
