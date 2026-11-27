use actix_web::{
    HttpResponse,
    http::header,
    web::{Data, Json, Path},
};
use api_utils::context::WyrmContext;
use chrono::{TimeDelta, Utc};
use database::{
    models::{feed::Feed, feed_icon::FeedIcon, local_user::LocalUser},
    newtypes::FeedId,
    views::{self, feed::FeedView},
};
use std::time::Duration;
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_rss::worker::WorkerCommand;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

pub async fn get(
    user: AuthenticatedUser,
    path: Path<FeedId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Feed>> {
    let feed = Feed::get(&ctx.db_pool, user.id, path.into_inner()).await?;
    Ok(Json(feed))
}

pub async fn list(
    user: AuthenticatedUser,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Vec<FeedView>>> {
    let feeds = views::feed::list(&ctx.db_pool, user.id).await?;
    Ok(Json(feeds))
}

pub async fn icon(
    user: AuthenticatedUser,
    path: Path<FeedId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<HttpResponse> {
    let icon = FeedIcon::get(&ctx.db_pool, user.id, path.into_inner()).await?;
    let Some(FeedIcon {
        data: Some(data),
        content_type,
        ..
    }) = icon
    else {
        return Ok(HttpResponse::NotFound().finish());
    };
    Ok(HttpResponse::Ok()
        .content_type(content_type.unwrap_or_else(|| "image/x-icon".to_string()))
        // Icons rarely change; spare the repeat requests as post lists render.
        .insert_header((header::CACHE_CONTROL, "max-age=86400"))
        .body(data))
}

/// Polls the user's feeds now. Replies 200 when they're done, or 202 if they're
/// still running when the HTTP timeout runs out.
pub async fn poll(user: AuthenticatedUser, ctx: Data<WyrmContext>) -> WyrmResult<HttpResponse> {
    let interval = ctx.runtime_settings.read()?.manual_poll_interval_secs;

    // Refused when the user refreshed less than `manual_poll_interval_secs` ago.
    if !LocalUser::try_start_manual_poll(&ctx.db_pool, user.id, interval).await? {
        let last = LocalUser::get(&ctx.db_pool, user.id)
            .await?
            .last_manual_poll_at;

        let next_allowed_at = last.unwrap_or_else(Utc::now) + TimeDelta::seconds(interval as i64);
        let secs_left = (next_allowed_at - Utc::now()).num_seconds().max(0) as u64;
        let mins_left = secs_left.div_ceil(60);

        return Err(WyrmError::TooManyRequests(format!(
            "feeds were refreshed recently, try again in {mins_left} min"
        )));
    }

    // channel through which worker signals completion
    let (tx, rx) = tokio::sync::oneshot::channel();
    ctx.worker_tx
        .send(WorkerCommand::PollUserFeeds {
            reply: tx,
            user_id: user.id,
        })
        .await
        .map_err(|_| WyrmError::WorkerError("feed worker channel closed".into()))?;

    // Waits up to the HTTP timeout, then the poll keeps running in the background.
    let http_timeout = ctx.runtime_settings.read()?.http_timeout_secs;
    match tokio::time::timeout(Duration::from_secs(http_timeout as u64), rx).await {
        Ok(Ok(())) => Ok(HttpResponse::Ok().finish()),
        // The worker task ended without replying.
        Ok(Err(_)) => Err(WyrmError::WorkerError(
            "feed poll ended without replying".into(),
        )),
        Err(_) => Ok(HttpResponse::Accepted().finish()),
    }
}
