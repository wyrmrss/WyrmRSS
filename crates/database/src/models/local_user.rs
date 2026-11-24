use crate::{
    DatabaseConn,
    DatabasePool,
    newtypes::UserId,
    schema::{self, local_users},
};
use chrono::{DateTime, Utc};
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

    pub async fn create(pool: &DatabasePool, form: LocalUserInsertForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::insert_into(local_users::table)
            .values(&form)
            .returning(Self::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(WyrmError::from)
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
}
