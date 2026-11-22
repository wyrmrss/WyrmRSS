use crate::{
    DatabaseConn,
    DatabasePool,
    newtypes::UserId,
    schema::{self, local_users},
};
use chrono::{DateTime, Utc};
use diesel::{Selectable, deserialize::Queryable, prelude::*};
use diesel_async::RunQueryDsl;
use wyrm_utils::{error::WyrmError, result::WyrmResult};

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
