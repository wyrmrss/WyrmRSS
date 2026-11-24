use database::newtypes::UserId;
use serde::Serialize;

#[derive(Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AccountResponse {
    pub id: UserId,
    pub username: String,
    pub has_password: bool,
    pub oidc_linked: bool,
    pub is_admin: bool,
}
