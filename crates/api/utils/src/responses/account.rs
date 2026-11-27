use chrono::{DateTime, Utc};
use database::newtypes::UserId;
use serde::Serialize;

/// The signed-in user's own account.
#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AccountResponse {
    pub id: UserId,
    /// Lowercase, can't be changed.
    pub username: String,
    /// Whether the user can sign in with a password.
    pub has_password: bool,
    /// Whether an OIDC identity is linked to the account.
    pub oidc_linked: bool,
    /// Whether the user can change instance settings.
    pub is_admin: bool,
    /// When the user last refreshed their feeds manually.
    pub last_manual_poll_at: Option<DateTime<Utc>>,
}
