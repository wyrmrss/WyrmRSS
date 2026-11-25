use actix_web::{HttpRequest, HttpResponse, http::header, web::Data};
use api_utils::{context::WyrmContext, responses::instance::LocalInstanceResponse};
use database::models::instance::LocalInstance;
use wyrm_auth::Auth;
use wyrm_utils::result::WyrmResult;

/// Public, so setup and sign-in can read it. Some fields are hidden until
/// signed in, see [`LocalInstanceResponse`].
pub async fn get(
    req: HttpRequest,
    ctx: Data<WyrmContext>,
    auth: Data<Auth>,
) -> WyrmResult<HttpResponse> {
    let signed_in = auth.is_signed_in(&ctx.db_pool, &req).await?;
    let instance = LocalInstance::get(&ctx.db_pool).await?;

    // The body depends on the session, so it must not be cached.
    Ok(HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "no-store"))
        .json(LocalInstanceResponse::new(instance, signed_in)))
}
