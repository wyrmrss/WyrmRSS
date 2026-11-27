use crate::{
    DatabaseConn,
    DatabasePool,
    models::settings::Settings,
    newtypes::UserId,
    schema::{self, local_users},
};
use chrono::{DateTime, Duration, Utc};
use diesel::{Selectable, deserialize::Queryable, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl};
use wyrm_utils::{
    error::{DatabaseError, WyrmError},
    result::WyrmResult,
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::local_users)]
#[diesel(check_for_backend(crate::Backend))]
pub struct LocalUser {
    pub id: UserId,
    /// User's handle. Lowercase and URL-safe.
    pub username: String,
    /// Argon2 hash; None for users who only sign in through OIDC.
    pub password_hash: Option<String>,
    /// The OIDC provider's `sub` for this user, once linked.
    pub oidc_subject: Option<String>,
    /// Whether user is admin.
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    /// When the user last started a manual refresh, for the rate limit.
    pub last_manual_poll_at: Option<DateTime<Utc>>,
}

impl LocalUser {
    pub async fn get(pool: &DatabasePool, id: UserId) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        local_users::table
            .find(id)
            .select(Self::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }

    /// For password login. None when no user has that username.
    pub async fn get_by_username(pool: &DatabasePool, username: &str) -> WyrmResult<Option<Self>> {
        let mut conn = pool.get().await?;
        local_users::table
            .filter(local_users::username.eq(username))
            .select(Self::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(WyrmError::from)
    }

    /// For OIDC login. None when no user is linked to that subject.
    pub async fn get_by_oidc_subject(
        pool: &DatabasePool,
        subject: &str,
    ) -> WyrmResult<Option<Self>> {
        let mut conn = pool.get().await?;
        local_users::table
            .filter(local_users::oidc_subject.eq(subject))
            .select(Self::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(WyrmError::from)
    }

    /// Takes a connection so callers can run it inside a transaction, e.g. setup with
    /// `claim_setup`.
    pub async fn update(
        conn: &mut DatabaseConn,
        id: UserId,
        form: LocalUserUpdateForm,
    ) -> WyrmResult<Self> {
        diesel::update(local_users::table.find(id))
            .set(form)
            .returning(Self::as_returning())
            .get_result(conn)
            .await
            .map_err(WyrmError::from)
    }

    /// Creates the user and their default settings row in one transaction,
    /// so a user can't exist without settings.
    pub async fn create(pool: &DatabasePool, form: LocalUserInsertForm) -> WyrmResult<Self> {
        let mut pool = pool.get().await?;
        let conn = &mut *pool;

        conn.transaction(async |conn| {
            let user = diesel::insert_into(local_users::table)
                .values(&form)
                .returning(Self::as_returning())
                .get_result(conn)
                .await?;
            Settings::create_default(conn, user.id).await?;
            Ok(user)
        })
        .await
    }

    /// Deletes the user and, by cascade, everything they own.
    /// Fails with a conflict if they're the instance's only admin.
    pub async fn delete(pool: &DatabasePool, id: UserId) -> WyrmResult<Self> {
        let mut pool = pool.get().await?;
        let conn = &mut *pool;

        conn.transaction(async |conn| {
            let admins = local_users::table
                .filter(local_users::is_admin.eq(true))
                .select(local_users::id);

            // Locks the admin rows, so two admins deleting themselves at once can't both pass.
            // SQLite has no row locks but only allows one writer at a time.
            #[cfg(feature = "postgres")]
            let admins = admins.for_update();
            let admin_ids: Vec<UserId> = admins.load(conn).await?;

            if admin_ids == [id] {
                return Err(WyrmError::Database(DatabaseError::Conflict(
                    "an instance needs at least one admin".into(),
                )));
            }

            diesel::delete(local_users::table.find(id))
                .returning(Self::as_returning())
                .get_result(conn)
                .await
                .map_err(WyrmError::from)
        })
        .await
    }

    pub async fn try_start_manual_poll(
        pool: &DatabasePool,
        user_id: UserId,
        interval_secs: i32,
    ) -> WyrmResult<bool> {
        let mut conn = pool.get().await?;
        let cutoff = Utc::now() - Duration::seconds(interval_secs as i64);

        let target = local_users::table.find(user_id).filter(
            local_users::last_manual_poll_at
                .is_null()
                .or(local_users::last_manual_poll_at.lt(cutoff)),
        );

        let updated = diesel::update(target)
            .set(local_users::last_manual_poll_at.eq(Utc::now()))
            .execute(&mut conn)
            .await?;

        Ok(updated == 1)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::local_users)]
#[diesel(check_for_backend(crate::Backend))]
pub struct LocalUserInsertForm {
    pub username: String,
    pub password_hash: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = schema::local_users)]
pub struct LocalUserUpdateForm {
    pub username: Option<String>,
    /// Argon2 hash.
    pub password_hash: Option<String>,
    /// The OIDC provider's `sub`. Replaces any subject already linked.
    pub oidc_subject: Option<String>,
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;
    use crate::setup_test_db;

    /// Unique per run because the test database is shared across concurrent tests.
    fn unique_username() -> String {
        format!(
            "user_{}",
            Utc::now()
                .timestamp_nanos_opt()
                .expect("timestamp in range")
        )
    }

    async fn create_user(pool: &DatabasePool, username: String) -> WyrmResult<LocalUser> {
        LocalUser::create(
            pool,
            LocalUserInsertForm {
                username,
                password_hash: "hash".into(),
            },
        )
        .await
    }

    /// User 1 is the only admin in the test database.
    #[tokio::test]
    async fn delete_refuses_the_only_admin() {
        let pool = setup_test_db().await;
        assert!(
            matches!(
                LocalUser::delete(&pool, UserId(1)).await,
                Err(WyrmError::Database(DatabaseError::Conflict(_)))
            ),
            "deleting the only admin should conflict"
        );
    }

    #[tokio::test]
    async fn delete_removes_a_regular_user() {
        let pool = setup_test_db().await;
        let user = create_user(&pool, unique_username())
            .await
            .expect("create should succeed");

        LocalUser::delete(&pool, user.id)
            .await
            .expect("delete should succeed");
        let found = LocalUser::get_by_username(&pool, &user.username)
            .await
            .expect("lookup should succeed");
        assert!(found.is_none(), "user should be gone");
    }

    #[tokio::test]
    async fn create_refuses_a_taken_username() {
        let pool = setup_test_db().await;
        let username = unique_username();
        let user = create_user(&pool, username.clone())
            .await
            .expect("create should succeed");

        assert!(
            matches!(
                create_user(&pool, username).await,
                Err(WyrmError::Database(DatabaseError::UniqueViolation(_)))
            ),
            "second user with the same username should be refused"
        );

        LocalUser::delete(&pool, user.id)
            .await
            .expect("should delete user");
    }

    #[tokio::test]
    async fn manual_poll_is_refused_within_the_interval() {
        let pool = setup_test_db().await;
        let user = create_user(&pool, unique_username())
            .await
            .expect("create should succeed");

        let first = LocalUser::try_start_manual_poll(&pool, user.id, 60)
            .await
            .expect("first poll should run the query");
        assert!(first, "first manual poll should be allowed");

        let second = LocalUser::try_start_manual_poll(&pool, user.id, 60)
            .await
            .expect("second poll should run the query");
        assert!(!second, "second manual poll within 60s should be refused");

        let last = LocalUser::get(&pool, user.id)
            .await
            .expect("get should succeed")
            .last_manual_poll_at;
        assert!(last.is_some(), "allowed poll should record its time");

        LocalUser::delete(&pool, user.id)
            .await
            .expect("should delete user");
    }

    #[tokio::test]
    async fn manual_poll_with_zero_interval_is_always_allowed() {
        let pool = setup_test_db().await;
        let user = create_user(&pool, unique_username())
            .await
            .expect("create should succeed");

        for attempt in 1..=3 {
            let allowed = LocalUser::try_start_manual_poll(&pool, user.id, 0)
                .await
                .expect("poll should run the query");
            assert!(
                allowed,
                "manual poll {attempt} should be allowed with no limit"
            );
        }

        LocalUser::delete(&pool, user.id)
            .await
            .expect("should delete user");
    }
}
