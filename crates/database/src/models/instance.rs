use crate::{
    DatabaseConn,
    DatabasePool,
    models::local_user::{LocalUser, LocalUserUpdateForm},
    newtypes::UserId,
    schema::{self, local_instance},
};
use diesel::{Selectable, deserialize::Queryable, prelude::*};
use diesel_async::{AsyncConnection, RunQueryDsl};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use wyrm_utils::{
    error::{DatabaseError::Conflict, WyrmError},
    result::WyrmResult,
};

#[derive(Clone, Copy, Debug, PartialEq, DbEnum, Serialize, Deserialize, ts_rs::TS)]
#[cfg_attr(
    feature = "postgres",
    ExistingTypePath = "crate::schema::sql_types::RegistrationMode"
)]
#[serde(rename_all = "snake_case")]
#[ts(rename_all = "snake_case", export)]
pub enum RegistrationMode {
    /// Account registration is not allowed.
    Closed,
    /// Anyone can register.
    Open,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::local_instance)]
#[diesel(check_for_backend(crate::Backend))]
pub struct LocalInstance {
    /// Whether this instance has been set up yet.
    /// True from the start on desktop, which is single-user and skips setup.
    pub is_setup: bool,
    /// Who can create an account. Closed by default.
    pub registration_mode: RegistrationMode,
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

    pub async fn update(pool: &DatabasePool, form: LocalInstanceUpdateForm) -> WyrmResult<Self> {
        let mut conn = pool.get().await?;
        diesel::update(local_instance::table)
            .set(form)
            .returning(Self::as_returning())
            .get_result(&mut conn)
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
                oidc_subject: None,
            };
            LocalUser::update(conn, UserId(1), form).await
        })
        .await
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = schema::local_instance)]
pub struct LocalInstanceUpdateForm {
    pub registration_mode: RegistrationMode,
}

#[cfg(all(test, feature = "postgres"))]
mod tests {
    use super::*;
    use crate::setup_test_db;

    #[tokio::test]
    async fn update_round_trips_registration_mode() {
        let pool = setup_test_db().await;
        let original = LocalInstance::get(&pool)
            .await
            .expect("get should succeed")
            .registration_mode;

        for mode in [RegistrationMode::Open, RegistrationMode::Closed] {
            let form = LocalInstanceUpdateForm {
                registration_mode: mode,
            };
            LocalInstance::update(&pool, form)
                .await
                .expect("update should succeed");
            let stored = LocalInstance::get(&pool).await.expect("get should succeed");
            assert_eq!(stored.registration_mode, mode);
        }

        let form = LocalInstanceUpdateForm {
            registration_mode: original,
        };
        LocalInstance::update(&pool, form)
            .await
            .expect("should restore the original mode");
    }
}
