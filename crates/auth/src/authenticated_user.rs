use actix_web::{
    Error,
    FromRequest,
    HttpMessage,
    HttpRequest,
    dev::Payload,
    error::ErrorUnauthorized,
};
use database::{models::local_user::LocalUser, newtypes::UserId};
use std::future::{Ready, ready};
use wyrm_utils::{error::AuthError, result::WyrmResult};

/// The signed-in user. This is what API handlers get from middleware.
#[derive(Clone)]
pub struct AuthenticatedUser {
    pub id: UserId,
    pub username: String,
    pub is_admin: bool,
}

impl From<LocalUser> for AuthenticatedUser {
    fn from(user: LocalUser) -> Self {
        Self {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
        }
    }
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    /// Reads the user the auth middleware put into the request.
    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let result = req
            .extensions()
            .get::<AuthenticatedUser>()
            .cloned()
            .ok_or_else(|| ErrorUnauthorized("Authentication required"));

        ready(result)
    }
}

impl AuthenticatedUser {
    /// Errors with a 403 unless the user is an admin.
    pub fn require_admin(&self) -> WyrmResult<()> {
        if !self.is_admin {
            return Err(AuthError::AdminRequired.into());
        }
        Ok(())
    }
}
