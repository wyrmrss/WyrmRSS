use crate::{DatabaseConn, DatabasePool, newtypes::UserId, schema::settings};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use wyrm_utils::{error::WyrmError, result::WyrmResult};

/// Controls when posts are marked as read.
/// - `OnOpen`: automatically marked read when opened in the reader.
/// - `OnScroll`: marked read when scrolled past in the post list, or when opened.
/// - `Manually`: only marked read via the toggle button.
/// - `Disabled`: read state is never updated; all posts appear as read.
#[derive(Clone, Debug, DbEnum, Serialize, Deserialize, ts_rs::TS)]
// Postgres has a real `read_mode` enum type declared in schema.rs. SQLite does
// not, and diesel-derive-enum only emits its sqlite impls against a mapping type
// it generates itself (`ReadModeMapping`). Supplying `ExistingTypePath` there
// leaves those impls referring to a type that was never defined.
#[cfg_attr(
    feature = "postgres",
    ExistingTypePath = "crate::schema::sql_types::ReadMode"
)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export)]
pub enum ReadMode {
    OnOpen,
    OnScroll,
    Manually,
    Disabled,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(crate::Backend))]
#[derive(ts_rs::TS)]
#[ts(optional_fields, export)]
pub struct Settings {
    #[serde(skip)]
    #[ts(skip)]
    pub user_id: UserId,
    /// Number of posts returned per page when listing.
    pub page_size: i32,
    /// When posts get marked as read (see [`ReadMode`]).
    pub read_mode: ReadMode,
    /// Delete read posts after N days.
    /// Skips bookmarked (read later) and archived posts.
    pub expire_read_after_days: Option<i32>,
    /// Delete unread posts after N days.
    /// Skips bookmarked (read later) and archived posts.
    pub expire_unread_after_days: Option<i32>,
    /// Delay before marking post as read.
    pub read_delay_secs: i32,
}

impl Settings {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub(crate) fn owned(user_id: UserId) -> _ {
        settings::table.filter(settings::user_id.eq(user_id))
    }

    pub async fn get(pool: &DatabasePool, user_id: UserId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        Self::owned(user_id)
            .select(Settings::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    /// Every user's settings. Unscoped, for background jobs only.
    pub async fn get_all(pool: &DatabasePool) -> WyrmResult<Vec<Self>> {
        let mut conn = pool.get().await?;
        settings::table
            .select(Settings::as_select())
            .load(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    pub async fn update(pool: &DatabasePool, form: SettingsUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(Self::owned(form.user_id))
            .set(form)
            .returning(Settings::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    /// Gives a new user a settings row with the column defaults.
    pub async fn create_default(conn: &mut DatabaseConn, user_id: UserId) -> WyrmResult<()> {
        diesel::insert_into(settings::table)
            .values(settings::user_id.eq(user_id))
            .execute(conn)
            .await?;
        Ok(())
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::settings)]
#[diesel(check_for_backend(crate::Backend))]
pub struct SettingsUpdateForm {
    #[diesel(skip_update)]
    pub user_id: UserId,
    pub page_size: Option<i32>,
    pub read_mode: Option<ReadMode>,
    pub expire_read_after_days: Option<Option<i32>>,
    pub expire_unread_after_days: Option<Option<i32>>,
    pub read_delay_secs: Option<i32>,
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;
    use crate::setup_test_db;

    /// Snapshots the row, applies updates, restores the original and only then
    /// asserts, so a failed assertion can't leave the dev database dirty.
    #[tokio::test]
    async fn update_changes_only_given_fields() {
        let pool = setup_test_db().await;

        let original = Settings::get(&pool, UserId(1))
            .await
            .expect("should read settings");

        // Change a few fields, leave the rest as `None`.
        let updated = Settings::update(
            &pool,
            SettingsUpdateForm {
                user_id: UserId(1),
                page_size: Some(original.page_size + 1),
                read_mode: Some(ReadMode::Disabled),
                expire_read_after_days: Some(Some(30)),
                expire_unread_after_days: None,
                read_delay_secs: None,
            },
        )
        .await
        .expect("update should succeed");

        // An explicit inner `None` clears the nullable expiry.
        let cleared = Settings::update(
            &pool,
            SettingsUpdateForm {
                user_id: UserId(1),
                page_size: None,
                read_mode: None,
                expire_read_after_days: Some(None),
                expire_unread_after_days: None,
                read_delay_secs: None,
            },
        )
        .await
        .expect("update should succeed");

        // Restore before asserting
        Settings::update(
            &pool,
            SettingsUpdateForm {
                user_id: UserId(1),
                page_size: Some(original.page_size),
                read_mode: Some(original.read_mode.clone()),
                expire_read_after_days: Some(original.expire_read_after_days),
                expire_unread_after_days: None,
                read_delay_secs: None,
            },
        )
        .await
        .expect("restore should succeed");

        // Given fields changed...
        assert_eq!(updated.page_size, original.page_size + 1);
        assert!(matches!(updated.read_mode, ReadMode::Disabled));
        assert_eq!(updated.expire_read_after_days, Some(30));
        // ...while `None` fields were left untouched.
        assert_eq!(updated.read_delay_secs, original.read_delay_secs);
        // An explicit inner `None` writes NULL...
        assert_eq!(cleared.expire_read_after_days, None);
        // ...but a plain `None` field is still left untouched.
        assert_eq!(cleared.page_size, original.page_size + 1);
    }
}
