use database::models::post::Post;
use serde::Serialize;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct UpdatePostResponse {
    pub post: Post,
    pub feed_unread_count: i64,
}
