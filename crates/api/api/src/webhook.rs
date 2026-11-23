use actix_web::web::{Data, Json, Path};
use api_utils::context::WyrmContext;
use database::{
    models::webhook::Webhook,
    newtypes::FeedId,
    views::{self, webhook::FeedWebhookView},
};
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_utils::result::WyrmResult;

pub async fn list(
    user: AuthenticatedUser,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Vec<Webhook>>> {
    let webhooks = Webhook::get_all(&ctx.db_pool, user.id).await?;
    Ok(Json(webhooks))
}

pub async fn list_for_feed(
    user: AuthenticatedUser,
    path: Path<FeedId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Vec<FeedWebhookView>>> {
    let webhooks = views::webhook::list_for_feed(&ctx.db_pool, user.id, path.into_inner()).await?;
    Ok(Json(webhooks))
}
