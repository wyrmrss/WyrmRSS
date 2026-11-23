use actix_web::{
    HttpResponse,
    web::{Data, Json, Path, Query},
};
use api_utils::context::WyrmContext;
use database::{
    models::{
        archive::PostArchive,
        post::{MarkReadTarget, Post},
    },
    newtypes::{FeedId, FolderId, PostId},
    utils::pagination::{PagedResponse, PaginationCursor},
    views::{archive::get_post_archive_insert_form, post::PostQuery},
};
use serde::Deserialize;
use wyrm_auth::authenticated_user::AuthenticatedUser;
use wyrm_utils::result::WyrmResult;

#[derive(Deserialize, ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct ListPosts {
    pub page: Option<PaginationCursor>,
    pub feed_id: Option<FeedId>,
    pub folder_id: Option<FolderId>,
    pub bookmarked: Option<bool>,
    pub unread_only: Option<bool>,
    pub search: Option<String>,
}

pub async fn get(
    user: AuthenticatedUser,
    path: Path<PostId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<Post>> {
    let post_id = path.into_inner();
    let post = Post::get(&ctx.db_pool, user.id, post_id).await?;
    Ok(Json(post))
}

pub async fn archive(
    user: AuthenticatedUser,
    path: Path<PostId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PostArchive>> {
    let post_id = path.into_inner();
    let form = get_post_archive_insert_form(&ctx.db_pool, user.id, post_id).await?;
    let archived = PostArchive::create(&ctx.db_pool, form).await?;
    Ok(Json(archived))
}

pub async fn unarchive(
    user: AuthenticatedUser,
    path: Path<PostId>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<HttpResponse> {
    let post_id = path.into_inner();
    PostArchive::delete(&ctx.db_pool, user.id, post_id).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn mark_as_read(
    user: AuthenticatedUser,
    Json(target): Json<MarkReadTarget>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<HttpResponse> {
    let _ = Post::mark_read(&ctx.db_pool, user.id, target).await?;
    Ok(HttpResponse::NoContent().finish())
}

pub async fn list(
    user: AuthenticatedUser,
    query: Query<ListPosts>,
    ctx: Data<WyrmContext>,
) -> WyrmResult<Json<PagedResponse<Vec<Post>>>> {
    let query = query.into_inner();
    let page_size = ctx.runtime_settings.read()?.page_size;
    let page = PostQuery {
        cursor: query.page,
        feed_id: query.feed_id,
        folder_id: query.folder_id,
        bookmarked: query.bookmarked,
        unread_only: query.unread_only.or(Some(true)),
        search: query.search,
    }
    .list(&ctx.db_pool, user.id, page_size)
    .await?;
    Ok(Json(page))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::web::Query;

    /// Parse a query string through the same extractor production uses.
    fn parse(qs: &str) -> ListPosts {
        Query::<ListPosts>::from_query(qs)
            .expect("query should parse")
            .into_inner()
    }

    #[test]
    fn parses_scalar_fields() {
        let q = parse("search=rust");
        assert_eq!(q.search.as_deref(), Some("rust"));
    }
}
