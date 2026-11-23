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

#[derive(AsChangeset)]
#[diesel(table_name = schema::local_users)]
pub struct LocalUserUpdateForm {
    pub username: Option<String>,
    /// Argon2 hash.
    pub password_hash: Option<String>,
    /// The OIDC provider's `sub`. Replaces any subject already linked.
    pub oidc_subject: Option<String>,
}
