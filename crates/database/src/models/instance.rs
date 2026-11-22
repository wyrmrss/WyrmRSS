use crate::{
    DatabaseConn, DatabasePool,
    models::local_user::{LocalUser, LocalUserUpdateForm},
    newtypes::UserId,
    schema::{self, local_instance},
};
use diesel::{Selectable, deserialize::Queryable, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl};
use wyrm_utils::{
    error::{DatabaseError::Conflict, WyrmError},
    result::WyrmResult,
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::local_instance)]
#[diesel(check_for_backend(crate::Backend))]
pub struct LocalInstance {
    /// Whether this instance has been set up yet.
    /// True from the start on desktop, which is single-user and skips setup.
    pub is_setup: bool,
}

impl LocalInstance {
    pub async fn get(pool: &DatabasePool) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        local_instance::table
            .select(LocalInstance::as_select())
            .first(&mut conn)
            .await
            .map_err(WyrmError::from)
    }
    /// Marks the instance as set up; false if it already was.
    pub async fn claim_setup(conn: &mut DatabaseConn) -> WyrmResult<bool> {
        let updated =
            diesel::update(local_instance::table.filter(local_instance::is_setup.eq(false)))
                .set(local_instance::is_setup.eq(true))
                .execute(conn)
                .await?;
        Ok(updated == 1)
    }

    /// Marks the instance as set up and gives the admin (user 1) their username and password,
    /// both or neither. Fails with a conflict if the instance is already set up.
    pub async fn complete_setup(
        pool: &DatabasePool,
        username: String,
        password_hash: String,
    ) -> WyrmResult<LocalUser> {
        let mut pool = pool.get().await?;
        let conn = &mut *pool;

        conn.transaction(async |conn| {
            if !Self::claim_setup(conn).await? {
                return Err(WyrmError::Database(Conflict(
                    "instance already set up".to_string(),
                )));
            }
            let form = LocalUserUpdateForm {
                username: Some(username),
                password_hash: Some(password_hash),
            };
            LocalUser::update(conn, UserId(1), form).await
        })
        .await
    }
}
